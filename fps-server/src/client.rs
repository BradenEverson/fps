//! A client and its relation to any games

use std::marker::PhantomData;

use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

use crate::server::ClientStream;

/// A connected state
#[derive(Default)]
pub struct Connected;
/// A disconnected state
#[derive(Default)]
pub struct Disconnected;

/// A client's current socket stream and any additional positional information
#[derive(Default)]
pub struct ClientSession<ID: Default, STATE> {
    /// Client name
    name: String,
    /// If client is in a game, what game are they in
    in_game: Option<ID>,
    /// If client is in a game, the stream they are communicating through
    stream: Option<ClientStream>,
    /// Whether the client is connected or not
    _state: PhantomData<STATE>,
}

impl<ID: Default, STATE> ClientSession<ID, STATE> {
    /// Creates a new disconnected client
    pub fn new() -> ClientSession<ID, Disconnected> {
        ClientSession::default()
    }

    /// Connects a client to a ClientStream
    pub fn connect(self, stream: ClientStream) -> ClientSession<ID, Connected> {
        ClientSession {
            name: self.name,
            in_game: self.in_game,
            stream: Some(stream),
            _state: PhantomData,
        }
    }

    /// Returns a client's name
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// If client is in a game, returns a reference to that game's ID
    pub fn get_game_id(&self) -> Option<&ID> {
        self.in_game.as_ref()
    }
}

impl<ID: Default> ClientSession<ID, Connected> {
    /// Sends a message back to the client. Returns none if the stream no longer exists and client
    /// should disconnect. Returns Some(true) if successfully sent and Some(false) if not
    pub async fn send_message(&mut self, message: Message) -> Option<bool> {
        if let Some(stream) = &mut self.stream {
            Some(stream.send(message).await.is_ok())
        } else {
            None
        }
    }

    /// Hooks up messages to a handling function
    pub async fn receive_msg_callback<F>(&mut self, mut callback: F) -> Option<bool>
    where
        F: FnMut(Message) -> Option<bool>,
    {
        if let Some(stream) = &mut self.stream {
            while let Some(Ok(message)) = stream.next().await {
                callback(message)?;
            }
            Some(true)
        } else {
            None
        }
    }

    /// Disconnects client from the stream
    pub fn disconnect(self) -> ClientSession<ID, Disconnected> {
        ClientSession {
            name: self.name,
            in_game: self.in_game,
            stream: None,
            _state: PhantomData,
        }
    }
}
