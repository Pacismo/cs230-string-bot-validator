use std::{error, fmt::Display, io, net::SocketAddr};
use tokio::task::JoinError;

#[derive(Debug)]
pub enum Message {
    ClientConnection(SocketAddr),
    ClientSuccess(SocketAddr),
    ServerConnectionError(String),
    ClientError(String),
    ClientFailureError {
        error_message: String,
        cipher: String,
        message: String,
        expected: String,
    },
    FutureJoinError(String),
    IOError(io::Error),
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
