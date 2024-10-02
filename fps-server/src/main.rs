//! A running server instance!

use std::future::Future;
use std::pin::Pin;

use fps_server::engine::{GameInfo, SessionEngine};
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{body, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;

#[tokio::main]
async fn main() {
    let (mut engine, sender) = SessionEngine::<usize>::new();
    let listener = TcpListener::bind("0.0.0.0:7878")
        .await
        .expect("Failed to start up server");

    tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.expect("Error accepting connection");

            let io = TokioIo::new(socket);
            let manager = GameManger(sender.clone());

            tokio::spawn(async move {
                if let Err(e) = http1::Builder::new()
                    .serve_connection(io, manager)
                    .with_upgrades()
                    .await
                {
                    tracing::error!("Error serving connection :( {}", e)
                }
            });
        }
    });

    engine.run().await
}

/// Everything necessary to add a new game session to the engine
pub struct GameManger(pub Sender<GameInfo<usize>>);

impl Service<Request<body::Incoming>> for GameManger {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, _req: Request<body::Incoming>) -> Self::Future {
        todo!()
    }
}
