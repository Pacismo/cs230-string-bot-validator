use super::{
    message_channel::{ClientMessage, MessageChannel},
    Message,
};

use rand::{distributions::Uniform, prelude::*, random};
use std::net::{SocketAddr, TcpStream};

fn create_message<G: rand::RngCore>(rng: &mut G, max_len: usize) -> ([char; 26], String, String) {
    const ALPHABET: [char; 26] = [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ];

    let mut cipher: [char; 26] = ALPHABET.clone();
    cipher.shuffle(rng);

    let len = rng.sample(Uniform::new(1, max_len));

    let (expect, message) = (0..len).map(|_| rng.sample(Uniform::new(0, 26))).fold(
        (String::new(), String::new()),
        |(mut e, mut m), i| {
            m.push(ALPHABET[i]);
            e.push(cipher[i]);
            (e, m)
        },
    );

    (cipher, message, expect)
}

pub async fn handle_connection(
    (stream, client_socket): (TcpStream, SocketAddr),
    num_msg: usize,
    len_msg: usize,
    expected_email: Option<String>,
) -> Result<SocketAddr, Message> {
    let mut channel = MessageChannel::new(stream)?;

    match channel.read_message()? {
        ClientMessage::Hello(email) => {
            if let Some(expected) = expected_email {
                if email != expected {
                    return Err(Message::ClientError(format!("Client sent the wrong email address in HELLO message (expected: {expected}; got {email})")));
                }
            }
        }
        ClientMessage::Decrypt(_) => {
            return Err(Message::ClientError(
                "Client did not send a HELLO message".into(),
            ));
        }
    }

    let mut generator = StdRng::from_seed(random());

    for _ in 0..num_msg {
        let (cipher, message, expected) = create_message(&mut generator, len_msg);
        channel.write_problem(&cipher, &message)?;

        if let ClientMessage::Decrypt(result) = channel.read_message()? {
            if result != expected {
                return Err(Message::ClientFailureError {
                    error_message: "Client failed to decrypt a message".into(),
                    cipher: cipher.into_iter().collect(),
                    message: message,
                    expected: expected,
                });
            }
        } else {
            return Err(Message::ClientError("Client re-sent HELLO message".into()));
        }
    }

    channel.write_bye("GOODJOB".into())?;
    Ok(client_socket)
}
