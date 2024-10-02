//! The actual server networking components and communication channels

use std::future::Future;
use std::pin::Pin;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::service::Service;
use hyper::{body, Request, Response};
use tokio::sync::mpsc::Sender;

use crate::engine::GameInfo;

/// Everything necessary to add a new game session to the engine
pub struct GameManager(pub Sender<GameInfo<usize>>);

impl Service<Request<body::Incoming>> for GameManager {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, _req: Request<body::Incoming>) -> Self::Future {
        todo!()
    }
}
