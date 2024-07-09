mod parser;
mod pomodoro;
mod timer;
mod tui;
mod ui;

use crate::pomodoro::Pomodoro;
use crate::timer::Timer;
use clap::{Parser, Subcommand};
use parser::parse_duration;
use std::io;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "Pomodoro Timer")]
#[command(version = "0.1")]
#[command(about = "Pomodoro Timer", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Start a timer", visible_alias = "t")]
    Timer {
        #[arg(value_parser = parse_duration, short, long)]
        duration: Duration,
    },

    // TODO: handle default values
    #[command(about = "Start a pomodoro session", visible_alias = "p")]
    Pomodoro {
        #[arg(short, long)]
        sessions: Option<usize>,
        #[arg(value_parser = parse_duration)]
        focus_duration: Option<Duration>,
        #[arg(value_parser = parse_duration)]
        break_duration: Option<Duration>,
    },

    Start {
        #[arg(value_parser = parse_duration, short, long)]
        duration: Option<Duration>,
    },
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let tick_rate = Duration::from_millis(250);

    match &cli.command {
        Some(Commands::Timer { duration }) => {
            let mut timer = Timer::new(*duration);
            timer.run(&mut tui::init()?, tick_rate)?;
            tui::restore()?;

            Ok(())
        }
        Some(Commands::Pomodoro {
            sessions,
            focus_duration,
            break_duration,
        }) => {
            let sessions = sessions.unwrap_or(4);
            let focus_duration = focus_duration.unwrap_or(Duration::from_secs(25));
            let break_duration = break_duration.unwrap_or(Duration::from_secs(5));

            let mut pomodoro = Pomodoro::new(sessions, focus_duration, break_duration);
            pomodoro.run(&mut tui::init()?, tick_rate)?;
            tui::restore()?;

            Ok(())
        }
        _ => {
            let mut timer = Timer::new(Duration::from_secs(60));
            timer.run(&mut tui::init()?, tick_rate)?;
            tui::restore()?;

            Ok(())
        }
    }
}
