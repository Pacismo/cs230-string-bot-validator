use super::Message;
use std::{
    io::{self, BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
};

pub enum ClientMessage {
    /// Client `HELLO` message. Contains an email.
    Hello(String),
    /// Client decryption message. Contains the resulting string.
    Decrypt(String),
}

/// Wraps around the TCP stream to read from and write to the socket.
pub struct MessageChannel {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl MessageChannel {
    /// Create a new message channel.
    pub fn new(socket: TcpStream) -> io::Result<Self> {
        Ok(Self {
            reader: BufReader::new(socket.try_clone()?),
            writer: BufWriter::new(socket),
        })
    }

    /// Reads a message from the stream. Parses the input into a `ClientMessage`.
    pub fn read_message(&mut self) -> Result<ClientMessage, Message> {
        let mut line = String::new();

        self.reader.read_line(&mut line)?;

        let message: Vec<&str> = line.trim().split_whitespace().collect();

        if message.len() < 1 {
            Err(Message::ClientError("Client sent an empty message".into()))
        } else if message[0] != "cs230" || message.len() > 3 || message.len() == 1 {
            Err(Message::ClientError(
                "Could not parse client message".into(),
            ))
        } else if message.len() == 3 {
            if message[1] == "HELLO" {
                Ok(ClientMessage::Hello(message[2].into()))
            } else {
                Err(Message::ClientError(
                    "Could not parse client message".into(),
                ))
            }
        } else {
            Ok(ClientMessage::Decrypt(message[1].into()))
        }
    }

    /// Writes some data to the stream.
    fn write_message(&mut self, data: &str) -> io::Result<()> {
        let mut bytes = data.trim().as_bytes().to_owned();
        bytes.extend_from_slice(&[b'\n', b'\0']);

        self.writer.write(&bytes)?;
        self.writer.flush()?;

        Ok(())
    }

    /// Send the client a problem. This represents a `STATUS` message.
    pub fn write_problem(&mut self, cypher: &[char; 26], message: &str) -> io::Result<()> {
        let data = format!(
            "cs230 STATUS {} {}",
            &cypher.iter().collect::<String>(),
            message
        );
        self.write_message(&data)
    }

	/// Send the client a `BYE` message.
    pub fn write_bye(&mut self, hash: String) -> io::Result<()> {
        let data = format!("cs230 {hash} BYE");
        self.write_message(&data)
    }
}
