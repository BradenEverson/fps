//! The engine for several different game sessions running at the same time

use std::{collections::HashMap, fmt::Display, future::Future, hash::Hash, pin::Pin, sync::Arc};

use futures::lock::Mutex;
use tokio::sync::{mpsc::{Receiver, Sender}, Semaphore};

/// A limit on how many game sessions can run at a time
static MAX_LOBBIES: Semaphore = Semaphore::const_new(100);

/// The future responsible for a game's session. Return type is the game's ID as it comes out
type GameFuture<ID> = Pin<Box<dyn Future<Output = ID> + Send>>;

/// All relevant game metadata
pub struct GameInfo<ID> {
    /// The Game's ID
    pub id: ID,
    /// The Game's name
    pub name: String,
    /// The Game's game
    fut: GameFuture<ID>,
}

impl<ID> GameInfo<ID> {
    /// Creates a new GameInfo based on an ID, name, and arbitrary runtime future
    pub fn new<S: Into<String>>(id: ID, name: S, fut: GameFuture<ID>) -> Self {
        Self {
            id,
            name: name.into(),
            fut,
        }
    }

    /// Executes the game's internal future
    pub async fn exec(self) -> ID {
        self.fut.await
    }
}

/// The engine holds all currently executing games. It is not aware of any internal state of these
/// games aside from registered name
pub struct SessionEngine<ID> {
    /// Id - name mappings for games
    game_refs: Arc<Mutex<HashMap<ID, String>>>,
    /// The receiving end of a game sending channel
    receiver: Receiver<GameInfo<ID>>,
}

impl<ID> SessionEngine<ID>
where
    ID: Display + Send + Sync + Hash + Copy + Eq + 'static,
{
    /// Creates a new SessionEngine, returns itself and a sender for new game sessions
    pub fn new() -> (Self, Sender<GameInfo<ID>>) {
        let (write, read) = tokio::sync::mpsc::channel(100);
        (
            SessionEngine {
                game_refs: Arc::new(Mutex::new(HashMap::new())),
                receiver: read,
            },
            write,
        )
    }

    /// Runs until the Game Session channel closes
    pub async fn run(&mut self) {
        while let Some(game) = self.receiver.recv().await {
            let ref_clone = self.game_refs.clone();
            tokio::spawn(async move {
                tracing::info!("Waiting for available game slot...");

                let _ = MAX_LOBBIES.acquire().await.expect("Semaphore acquire failed");

                tracing::info!("Starting game with name `{}` and id {}", game.name, game.id);
                let id = game.id;
                {
                    ref_clone.lock().await.insert(id, game.name.clone());
                }

                game.exec().await;

                tracing::info!("Finished game {}", id);
                {
                    ref_clone.lock().await.remove(&id);
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::FutureExt;

    use crate::engine::{GameFuture, GameInfo, SessionEngine};

    #[tokio::test]
    async fn session_engine_can_be_added_to_while_running() {
        let _ = tracing_subscriber::fmt::try_init();
        fn generate_future<const TIME: usize>() -> GameFuture<usize> {
            async {
                tracing::info!("Entering future {TIME}");
                tokio::time::sleep(Duration::from_millis(TIME as u64)).await;
                tracing::info!("Ending future {TIME}");

                TIME
            }
            .boxed()
        }

        let fut1 = generate_future::<500>();
        let fut2 = generate_future::<200>();
        let fut3 = generate_future::<100>();

        let game1 = GameInfo::new(0, "Server 1", fut1);
        let game2 = GameInfo::new(1, "Server 3", fut2);
        let game3 = GameInfo::new(2, "Server 3", fut3);

        let (mut engine, sender) = SessionEngine::new();

        let start = tokio::spawn(async move {
            engine.run().await;
        });

        sender.send(game1).await.expect("Failed to send");

        let add = tokio::spawn(async move {
            tracing::debug!("Attempting to add new future 1");
            sender.send(game2).await.expect("Failed to send");
            tracing::debug!("Attempting to add new future 2");
            sender.send(game3).await.expect("Failed to send");
            tokio::time::sleep(Duration::from_millis(500)).await;
        });

        let result = tokio::join!(start, add);
        assert!(matches!(result, (Ok(_), Ok(_))))
    }
}
