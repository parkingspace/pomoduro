mod app;
mod cli;
mod event;
mod host;
mod parser;
mod pomodoro;
mod timer;
mod tui;
mod ui;

use app::{App, SessionInfo};
use timer::TimerAction;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};

use crate::cli::Commands;
use std::io;
use std::time::Duration;

const FOCUS_DURATION: u64 = 25;
const BREAK_DURATION: u64 = 5;
const LONG_BREAK_DURATION: u64 = 15;

#[tokio::main]
async fn main() -> io::Result<()> {
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
        // Some(Commands::Host { port }) => {
        //     // host
        //     // 1. start up TCP server
        //     let port = port.unwrap_or(8080);
        //     let addr = format!("127.0.0.0:{}", port);
        //     let listener = TcpListener::bind(addr).await?;
        //     let (state_tx, _) = broadcast::channel(16);
        //     let (command_tx, command_rx) = mpsc::channel(100);
        //
        //     // 2. create new host
        //
        //     // 3. listen for new connections in loop
        //     while let Ok((stream, addr)) = listener.accept().await {
        //         println!("New client connected: {}", addr);
        //         let state_rx = state_tx.subscribe();
        //         // 4. listen for commands sent from participants
        //         // 5. broadcast new states to the participants
        //         let command_tx = command_tx.clone();
        //         tokio::spawn(handle_client(stream, state_rx, command_tx));
        //     }
        // }
        // Some(Commands::Join { address, port }) => {
        //     // join
        //     use tokio::net::TcpStream;
        //     // 1. connect to the server using ip address
        //     let address = address.clone().unwrap_or("127.0.0.0".to_string());
        //     let port = port.unwrap_or(8080);
        //     let addr = format!("{}:{}", address, port);
        //     let mut stream = TcpStream::connect(addr).await?;
        //     // 2. receive state updates in loop
        //     // 3. send commands to the host
        //     // 4. update local UI based on received state
        // }
        _ => (),
    };
    // app.run(&mut tui::init()?)?;
    // tui::restore()?;

    Ok(())
}
