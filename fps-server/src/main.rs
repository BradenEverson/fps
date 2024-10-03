//! A running server instance!

use fps_server::{engine::SessionEngine, server::GameManager};
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let (mut engine, game_sender) = SessionEngine::<usize>::new();
    let (sender, mut receiver) = tokio::sync::mpsc::channel(100);
    let listener = TcpListener::bind("0.0.0.0:7878")
        .await
        .expect("Failed to start up server");

    let engine_game_ref = engine.game_refs.clone();
    tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.expect("Error accepting connection");

            let io = TokioIo::new(socket);
            let manager = GameManager::new(sender.clone(), game_sender.clone(), engine_game_ref.clone()) ;

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

    tokio::spawn(async move {
        while let Some(stream) = receiver.recv().await {
            // New client stream, add it to in lobby clients
        }
    });

    engine.run().await
}
