mod actor;
mod app;
mod cli;
mod event;
mod parser;
mod pomodoro;
mod timer;
mod tui;
mod ui;

use app::App;
use futures::{SinkExt, StreamExt};
use std::{error::Error, net::SocketAddr};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

use crate::cli::Commands;
use std::time::Duration;

const FOCUS_DURATION: u64 = 25;
const BREAK_DURATION: u64 = 5;
const LONG_BREAK_DURATION: u64 = 15;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = cli::parse();
    let tick_rate = Duration::from_millis(250);

    match &cli.command {
        Some(Commands::Timer { duration, name }) => {
            let name = name.as_ref().unwrap_or(&String::from("Timer")).to_string();
            App::new_timer(*duration, name, tick_rate)
                .run(&mut tui::init()?)
                .await?;
            tui::restore()?;
        }
        Some(Commands::Pomodoro {
            sessions,
            focus_duration,
            break_duration,
            long_break_duration,
        }) => {
            let total_sessions = sessions.unwrap_or(4);
            let focus_duration = focus_duration.unwrap_or(Duration::from_secs(FOCUS_DURATION));
            let break_duration = break_duration.unwrap_or(Duration::from_secs(BREAK_DURATION));
            let long_break_duration =
                long_break_duration.unwrap_or(Duration::from_secs(LONG_BREAK_DURATION));

            App::new_pomodoro(
                total_sessions,
                focus_duration,
                break_duration,
                long_break_duration,
                tick_rate,
            )
            .run(&mut tui::init()?)
            .await?;
            tui::restore()?;
        }
        Some(Commands::Host { port }) => {
            let port = port.unwrap_or(8080);
            let addr = SocketAddr::from(([127, 0, 0, 1], port));

            host(addr).await;

            tui::restore()?;
        }
        Some(Commands::Join { address, port }) => {
            use tokio::net;

            let port = port.unwrap_or(8080);
            let address = address.clone().unwrap_or("127.0.0.1".to_string());
            let addr = format!("{}:{}", address, port);

            if let Err(e) = net::lookup_host(&addr).await {
                println!("Host not found: {:?}", e);
            }

            let ws_addr = format!("ws://{}", addr).into_client_request().unwrap();
            let (ws_stream, _) = tokio_tungstenite::connect_async(ws_addr)
                .await
                .expect("Failed to connect");

            let (mut outgoing, incoming) = ws_stream.split();

            let (sender, receiver) = flume::unbounded();

            tokio::spawn(read_stdin(sender.clone()));

            let ws_to_stdout = incoming.for_each(|msg| async move {
                println!("Received message: {:?}", msg);

                let data = msg.unwrap().into_data();
                tokio::io::stdout().write_all(&data).await.unwrap();
            });

            let stdin_to_ws = async move {
                while let Ok(msg) = receiver.recv_async().await {
                    outgoing.send(msg).await.unwrap();
                }
            };

            tokio::pin!(ws_to_stdout, stdin_to_ws);
            tokio::select! {
                _ = &mut ws_to_stdout => {}
                _ = &mut stdin_to_ws => {}
            }
        }
        _ => (),
    };

    Ok(())
}

async fn host(addr: SocketAddr) {
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Listening on {}", addr);

    while let Ok((socket, addr)) = listener.accept().await {
        tokio::spawn(async move {
            handle_connection(addr, socket).await;
        });
    }
}

async fn handle_connection(client_addr: SocketAddr, socket: TcpStream) {
    let ws_stream = tokio_tungstenite::accept_async(socket)
        .await
        .expect("Error during the websocket handshake occurred");

    let (outgoing, mut incoming) = ws_stream.split();

    println!("WebSocket connection established: {}", client_addr);

    loop {
        let msg = match incoming.next().await {
            Some(msg) => msg,
            None => {
                println!("WebSocket connection closed");
                break;
            }
        };

        println!("Received message: {:?}", msg);
    }
}

async fn read_stdin(sender: flume::Sender<Message>) {
    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut buffer = String::new();

    loop {
        buffer.clear();
        if stdin.read_line(&mut buffer).await.unwrap() == 0 {
            break;
        }
        sender
            .send_async(Message::Text(buffer.trim().to_string()))
            .await
            .unwrap();
    }
}
