use std::{error, fmt::Display, io, net::SocketAddr};
use tokio::task::JoinError;

/// Represents a message that can be sent to the caller.
/// It can also be used as an error.
#[derive(Debug)]
pub enum Message {
    /// Non-error. The client connected to the server.
    ClientConnection(SocketAddr),
    /// Non-error. The client successfully completed the task.
    ClientSuccess(SocketAddr),
    /// Error: the server failed to bind to a socket.
    ServerConnectionError(String),
    /// Client error: the client sent an erroneous packet.
    ClientError(String),
    /// Client failure: the client sent the wrong output.
    ClientFailureError {
        error_message: String,
        cipher: String,
        message: String,
        expected: String,
    },
    /// Future joining error: a future could not be completed.
    FutureJoinError(String),
    /// IO Error: an IO operation failed.
    IOError(io::Error),
    /// General error: contains a boxed error.
    Error(Box<dyn error::Error + Send>),
}

impl From<io::Error> for Message {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<Box<dyn error::Error + Send>> for Message {
    fn from(value: Box<dyn error::Error + Send>) -> Self {
        Self::Error(value)
    }
}

impl From<JoinError> for Message {
    fn from(value: JoinError) -> Self {
        Self::FutureJoinError(value.to_string())
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::ServerConnectionError(error_message) => {
                write!(f, "Server connection error: {error_message}")
            }
            Message::ClientError(error_message) => write!(f, "Client error: {error_message}"),
            Message::ClientFailureError { error_message, .. } => {
                write!(f, "Client failure: {error_message}")
            }
            Message::IOError(error) => write!(f, "IOError: {error}"),
            Message::Error(error) => write!(f, "Error: {error}"),
            Message::ClientConnection(socket) => write!(f, "Client connected: {socket}"),
            Message::ClientSuccess(socket) => write!(f, "Client success: {socket}"),
            Message::FutureJoinError(error_message) => write!(f, "Join error: {error_message}"),
        }
    }
}

impl error::Error for Message {}
