mod server;

use clap::{Parser, Subcommand};

use std::{
    error::Error,
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::TryRecvError,
};

use crate::server::{create_server, TestParams};

#[derive(Subcommand, Debug, Clone)]
#[command()]
enum Mode {
    /// Runs once only.
    /// Takes, as an optional argument, the path to the client executable.
    /// If no client executable is specified, then the socket address will be printed.
    #[command()]
    Once {
        /// The number of messages to send before declaring victory.
        #[arg(default_value_t = 256, short = 'n', long = "n-msg")]
        message_count: usize,
        /// The maximum length of a message.
        #[arg(default_value_t = 512, short = 'l', long = "max-len")]
        max_message_len: usize,
        /// The client executable to run.
        /// Arguments are automatically generated and passed to the runtime.
        #[arg()]
        client_exec: Option<PathBuf>,
        /// The netid to use for the test.
        /// By default, it is set to example@umass.edu.
        /// This has no effect if no client was specified.
        #[arg(short = 'i', long = "netid")]
        netid: Option<String>,
        /// Hides the standard error output from the client.
        /// Has no effect if no client was specified.
        #[arg(short = 'e')]
        hide_stderr: bool,
    },
    /// Runs until an interrupt gets received.
    /// This creates a server that will continually receive incoming connections until an interrupt is received.
    #[command()]
    UntilInterrupt {
        /// The number of messages to send before declaring victory.
        #[arg(default_value_t = 256, short = 'n', long = "n-msg")]
        message_count: usize,
        /// The maximum length of a message.
        #[arg(default_value_t = 512, short = 'l', long = "max-len")]
        max_message_len: usize,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    /// How to run the test.
    #[command(subcommand)]
    mode: Mode,
}

async fn do_once(
    message_count: usize,
    max_message_len: usize,
    client: Option<PathBuf>,
    email: String,
    hide_stderr: bool,
) -> Result<(), Box<dyn Error>> {
    let test_params = TestParams {
        message_count,
        max_message_len,
        stop_at_first: true,
        expected_email: if client.is_some() {
            Some(email.clone())
        } else {
            None
        },
    };

    let (server_handle, address, _) = create_server(test_params)?;

    if let Some(client) = client {
        let mut child = Command::new(client)
            .args([email, address.port().to_string(), address.ip().to_string()])
            .stdin(Stdio::null())
            .stderr(if hide_stderr {
                Stdio::null()
            } else {
                Stdio::inherit()
            })
            .spawn()?;

        let code: Result<ExitStatus, Box<dyn Error>> = loop {
            if let Some(code) = child.try_wait()? {
                server_handle.await??;
                break Ok(code);
            }
        };

        match code {
            Ok(code) => {
                println!("Child process exited with code {code}");
                Ok(())
            }
            Err(e) => {
                eprintln!("Test failed!\n{e}");
                Err(e)
            }
        }
    } else {
        println!("Server address at {address}");
        server_handle.await?.map(|_| ())?;
        Ok(())
    }
}

async fn do_until_interrupt(message_count: usize, max_message_len: usize) -> ! {
    let test_params = TestParams {
        message_count,
        max_message_len,
        stop_at_first: false,
        expected_email: None,
    };

    let (server_handle, address, rx) = create_server(test_params).expect("Failed to create server");

    println!("Server address at {address}");

    loop {
        match rx.try_recv() {
            Ok(svr_error) => {
                eprintln!("{}", svr_error);
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                server_handle.abort();
                panic!("MPSC channel got disconnected");
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::parse();

    match args.mode {
        Mode::Once {
            message_count,
            max_message_len,
            client_exec,
            netid,
            hide_stderr,
        } => {
            do_once(
                message_count,
                max_message_len,
                client_exec,
                netid.unwrap_or("example@umass.edu".into()),
                hide_stderr,
            )
            .await
        }
        Mode::UntilInterrupt {
            message_count,
            max_message_len,
        } => do_until_interrupt(message_count, max_message_len).await,
    }
}
