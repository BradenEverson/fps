//! The engine for several different game sessions running at the same time

use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use futures::{lock::Mutex, stream::FuturesUnordered};

/// The future responsible for a game's session. Return type is the game's ID as it comes out
type GameFuture<ID> = Pin<Box<dyn Future<Output = ID> + Send>>;

/// The engine holds all currently executing games. It is not aware of any internal state of these
/// games aside from registered name
pub struct SessionEngine {
    /// All running games
    games: Arc<Mutex<FuturesUnordered<GameFuture<usize>>>>,
    /// Id - name mappings for games
    game_refs: HashMap<usize, String>
}


