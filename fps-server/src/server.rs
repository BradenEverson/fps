//! The actual server networking components and communication channels

use std::future::Future;
use std::pin::Pin;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::service::Service;
use hyper::upgrade::Upgraded;
use hyper::{body, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::WebSocketStream;

/// Everything necessary to add a new game session to the engine
pub struct GameManager(pub Sender<ClientStream>);

/// A Client WebSocket read and write stream
pub type ClientStream = WebSocketStream<TokioIo<Upgraded>>;

impl Service<Request<body::Incoming>> for GameManager {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, mut req: Request<body::Incoming>) -> Self::Future {
        let sender = self.0.clone();
        if hyper_tungstenite::is_upgrade_request(&req) {
            let (response, websocket) = hyper_tungstenite::upgrade(&mut req, None).expect("Error upgrading to websocket");

            tokio::spawn(async move {
                match websocket.await {
                    Ok(ws) => {
                        sender.send(ws).await.expect("Failed to transfer client websocket connection");
                    }
                    Err(e) => tracing::error!("Failed to construct websocket, {}", e)
                }
            });

            Box::pin(async { Ok(response) })
        } else {
            todo!()
        }
    }
}
