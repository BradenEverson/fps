//! The engine for several different game sessions running at the same time

use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use futures::{lock::Mutex, stream::FuturesUnordered, StreamExt};

/// The future responsible for a game's session. Return type is the game's ID as it comes out
type GameFuture<ID> = Pin<Box<dyn Future<Output = ID> + Send>>;

/// The engine holds all currently executing games. It is not aware of any internal state of these
/// games aside from registered name
#[derive(Default)]
pub struct SessionEngine {
    /// All running games
    games: Arc<Mutex<FuturesUnordered<GameFuture<usize>>>>,
    /// Id - name mappings for games
    game_refs: HashMap<usize, String>
}

impl SessionEngine {
    /// Registers a new game session with the engine. This locks the current execution for a second
    /// until it's been added in
    pub async fn register<S: Into<String>>(&mut self, game: GameFuture<usize>, name: S, id: usize) {
        let name = name.into();
        self.games.lock().await.push(game);
        self.game_refs.insert(id, name);
    }

    /// Runs the server until all games are done executing
    pub async fn run(&mut self) {
        while let Some(complete) = self.games.lock().await.next().await {
            tracing::info!("Completed Game with ID {complete}");
            self.game_refs.remove(&complete);
        }
    }

    /// Wraps self in a thread safe Arc<Mutex<_>>
    pub fn arc(self) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(self))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::FutureExt;

    use crate::engine::{GameFuture, SessionEngine};

    #[tokio::test]
    async fn session_engine_can_be_added_to_while_running() {
        let _ = tracing_subscriber::fmt::try_init();
        fn generate_future<const TIME: usize>() -> GameFuture<usize> {
            async {
                tracing::info!("Entering future {TIME}");
                std::thread::sleep(Duration::from_millis(TIME as u64));
                tracing::info!("Ending future {TIME}");

                TIME
            }.boxed()
        }

        let fut1 = generate_future::<500>();
        let fut2 = generate_future::<200>();
        let fut3 = generate_future::<100>();

        let engine = SessionEngine::default().arc();

        engine.lock().await.register(fut1, "future1", 0).await;

        let engine_clone = engine.clone();
        let start = tokio::spawn(async move {
            engine_clone.lock().await.run().await;
        });

        let add = tokio::spawn(async move {
            tracing::warn!("Attempting to add new future 1");
            engine.lock().await.register(fut2, "future2", 1).await;
            tracing::warn!("Attempting to add new future 2");
            engine.lock().await.register(fut3, "future3", 2).await;
        });


        tokio::join!(start, add).1.expect("Failed");
    }
}
