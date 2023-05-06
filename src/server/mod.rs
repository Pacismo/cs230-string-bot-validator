mod connection_handler;
mod error_messages;
mod message_channel;
use connection_handler::handle_connection;
pub use error_messages::Message;

use futures::executor::block_on;
use std::{
    io::{self, ErrorKind},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener},
    sync::mpsc::{channel, Receiver, Sender},
};
use tokio::task::JoinHandle;

/// Represents information about how to run the test.
#[derive(Clone, Debug)]
pub struct TestParams {
    pub message_count: usize,
    pub max_message_len: usize,
    pub stop_at_first: bool,
    pub expected_email: Option<String>,
}

/// The meat of the server.
/// Handles connections and errors.
async fn server_main(
    tx: Sender<Message>,
    listener: TcpListener,
    params: TestParams,
) -> Result<(), Message> {
    if params.stop_at_first {
        // Quit at the first client completion.
        match listener.accept() {
            Ok(cx) => handle_connection(
                cx,
                params.message_count,
                params.max_message_len,
                params.expected_email,
            )
            .await
            .map(|_| ()),
            Err(e) => Err(Message::ServerConnectionError(e.to_string())),
        }
    } else {
        // A list of handles. Handles are run asynchronymously to the server.
        // Yields at the end of every loop.
        let mut handles = vec![];

        // The listener shouldn't block this thread.
        listener.set_nonblocking(true)?;
        loop {
            'incoming_loop: loop {
                match listener.accept() {
                    Ok(cx) => {
                        // Tell the main thread that a client has connected to the server.
                        tx.send(Message::ClientConnection(cx.1.clone()))
                            .expect("Receiver end is disconnected");

                        handles.push(tokio::spawn(handle_connection(
                            cx,
                            params.message_count,
                            params.max_message_len,
                            None,
                        )));
                    }
                    Err(e) => {
                        if e.kind() != ErrorKind::WouldBlock {
                            // Tell the main thread that an error was encountered.
                            handles.into_iter().for_each(|h| h.abort());
                            return Err(Message::ServerConnectionError(
                                "Error encountered during listener loop".into(),
                            ));
                        } else {
                            break 'incoming_loop;
                        }
                    }
                }
            }

            // Search for and join green threads that have finished handling their connections.
            for (message, index) in handles
                .iter_mut()
                .zip(0..)
                .filter(|(h, _)| h.is_finished())
                .map(|(h, i)| match block_on(async { h.await }) {
                    Ok(Err(e)) => (e, i),
                    Err(e) => (Message::Error(Box::new(e)), i),
                    Ok(Ok(socket)) => (Message::ClientSuccess(socket), i),
                })
                .collect::<Vec<_>>()
                .into_iter()
            {
                tx.send(message).expect("Receiver end is disconnected");
                handles.remove(index);
            }

            // Yield this thread.
            tokio::task::yield_now().await;
        }
    }
}

pub fn create_server(
    params: TestParams,
) -> io::Result<(
    JoinHandle<Result<(), Message>>,
    SocketAddr,
    Receiver<Message>,
)> {
    let (tx, rx) = channel();

    let listener = TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::new(127, 0, 0, 1),
        0,
    )))?;

    let address = listener.local_addr()?;

    let handle = tokio::spawn(server_main(tx, listener, params));

    Ok((handle, address, rx))
}
