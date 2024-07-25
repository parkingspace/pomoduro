mod app;
mod cli;
mod parser;
mod pomodoro;
mod timer;
mod tui;
mod ui;

use app::App;

use crate::cli::Commands;
use std::io;
use std::time::Duration;

const FOCUS_DURATION: u64 = 25;
const BREAK_DURATION: u64 = 5;
const LONG_BREAK_DURATION: u64 = 15;

fn main() -> io::Result<()> {
    let cli = cli::parse();
    let tick_rate = Duration::from_millis(250);

    let mut app = match &cli.command {
        Some(Commands::Timer { duration, name }) => {
            let name = name.as_ref().unwrap_or(&String::from("Timer")).to_string();
            App::new_timer(*duration, name, tick_rate)
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
        }
        _ => App::new_timer(Duration::from_secs(60), String::from("Timer"), tick_rate),
    };
    app.run(&mut tui::init()?)?;
    tui::restore()?;

    Ok(())
}
