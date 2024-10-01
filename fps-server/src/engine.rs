//! The engine for several different game sessions running at the same time

use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use futures::lock::Mutex;
use tokio::sync::mpsc::{Sender, Receiver};

/// The future responsible for a game's session. Return type is the game's ID as it comes out
type GameFuture<ID> = Pin<Box<dyn Future<Output = ID> + Send>>;

/// The engine holds all currently executing games. It is not aware of any internal state of these
/// games aside from registered name
pub struct SessionEngine {
    /// Id - name mappings for games
    game_refs: Arc<Mutex<HashMap<usize, String>>>,
    receiver: Receiver<GameFuture<usize>>,
}

impl SessionEngine {
    /// Creates a new SessionEngine, returns itself and a sender for new game sessions
    pub fn new() -> (Self, Sender<GameFuture<usize>>) {
        let (write, read) = tokio::sync::mpsc::channel(100);
        (SessionEngine { game_refs: Arc::new(Mutex::new(HashMap::new())), receiver: read}, write)
    }

    /// Runs until the Game Session channel closes
    pub async fn run (&mut self) {
        while let Some(game) = self.receiver.recv().await {
            //let ref_clone = self.game_refs.clone();
            tokio::spawn(async move {
                let _ = game.await;
                //ref_clone.lock().await.remove(&id);
            });
        }
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

        let (mut engine, sender) = SessionEngine::new();

        let start = tokio::spawn(async move {
            engine.run().await;
        });

        sender.send(fut1).await.expect("Failed to send");

        let add = tokio::spawn(async move {
            tracing::warn!("Attempting to add new future 1");
            sender.send(fut2).await.expect("Failed to send");
            tracing::warn!("Attempting to add new future 2");
            sender.send(fut3).await.expect("Failed to send");
        });


        tokio::join!(start, add).0.expect("Failed");
    }
}
