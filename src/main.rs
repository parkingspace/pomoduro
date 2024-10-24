mod app;
mod cli;
mod event;
mod parser;
mod pomodoro;
mod timer;
mod tui;
mod ui;
mod websocket;

use crate::cli::Commands;

use app::App;
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use std::{error::Error, net::SocketAddr};
use tracing_subscriber::EnvFilter;

const FOCUS_DURATION: u64 = 25;
const BREAK_DURATION: u64 = 5;
const LONG_BREAK_DURATION: u64 = 15;

fn clear_log_file(path: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.set_len(0)?;
    file.flush()?;

    Ok(())
}

fn setup_tracing() {
    let file_appender = tracing_appender::rolling::never("./log", "pomoduro.log");

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(false)
        .with_target(true)
        .with_writer(file_appender)
        .with_thread_ids(true)
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_tracing();

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
            clear_log_file("./log/pomoduro.log")?;

            let port = port.unwrap_or(8080);
            let addr = SocketAddr::from(([127, 0, 0, 1], port));

            let (mut app, ws_handler) = App::new_shared_pomodoro(
                4,
                Duration::from_secs(FOCUS_DURATION),
                Duration::from_secs(BREAK_DURATION),
                Duration::from_secs(LONG_BREAK_DURATION),
                tick_rate,
            );

            tokio::spawn(async move { ws_handler.host(&addr).await });
            app.run(&mut tui::init()?).await?;

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

            let addr = addr.parse::<SocketAddr>().unwrap();

            let (mut app, ws_handler) = App::new_shared_pomodoro(
                4,
                Duration::from_secs(FOCUS_DURATION),
                Duration::from_secs(BREAK_DURATION),
                Duration::from_secs(LONG_BREAK_DURATION),
                tick_rate,
            );

            tokio::spawn(async move { ws_handler.join(&addr).await });

            app.run(&mut tui::init()?).await?;
            tui::restore()?;
        }
        _ => (),
    };

    Ok(())
}
