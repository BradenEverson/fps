//! The actual server networking components and communication channels

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::service::Service;
use hyper::upgrade::Upgraded;
use hyper::{body, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;
use tokio_tungstenite::WebSocketStream;
use crate::engine::GameInfo;

/// Everything necessary to add a new game session to the engine
pub struct GameManager {
    /// The stream state communication
    stream_sender: Sender<ClientStream>,
    /// The game creation communication
    game_sender: Sender<GameInfo<usize>>,
    /// Lobbies that exist
    lobbies: Arc<RwLock<HashMap<usize, String>>>
}

impl GameManager {
    /// Creates a new GameManager
    pub fn new(stream_sender: Sender<ClientStream>, game_sender: Sender<GameInfo<usize>>, lobbies: Arc<RwLock<HashMap<usize, String>>>) -> Self {
        Self { stream_sender, game_sender, lobbies }
    }

    /// Returns an owned vector of the lobby names that exist. Does not hold the mutex lock on
    /// return
    pub async fn lobbies(&self) -> Vec<String> {
        let mut lobbies = vec![];

        for lobby in  self.lobbies.read().await.values() {
            lobbies.push(lobby.to_owned())
        }

        lobbies
    }
}

/// A Client WebSocket read and write stream
pub type ClientStream = WebSocketStream<TokioIo<Upgraded>>;

impl Service<Request<body::Incoming>> for GameManager {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, mut req: Request<body::Incoming>) -> Self::Future {
        let sender = self.stream_sender.clone();
        if hyper_tungstenite::is_upgrade_request(&req) {
            let (response, websocket) =
                hyper_tungstenite::upgrade(&mut req, None).expect("Error upgrading to websocket");

            tokio::spawn(async move {
                match websocket.await {
                    Ok(ws) => {
                        sender
                            .send(ws)
                            .await
                            .expect("Failed to transfer client websocket connection");
                    }
                    Err(e) => tracing::error!("Failed to construct websocket, {}", e),
                }
            });

            Box::pin(async { Ok(response) })
        } else {
            todo!()
        }
    }
}
