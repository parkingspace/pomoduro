mod parser;
mod timer;
mod tui;
mod ui;

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
    // TODO: short and long commands
    #[command(about = "Start a timer")]
    Timer {
        #[arg(value_parser = parse_duration, short, long)]
        duration: Duration,
    },

    // TODO: handle default values
    // TODO: short and long commands
    #[command(about = "Start a pomodoro session", visible_alias = "p")]
    Pomodoro {
        #[arg(value_parser = parse_duration, short, long)]
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

    match &cli.command {
        Some(Commands::Timer { duration }) => {
            let mut timer = Timer::new(*duration);
            timer.run(&mut tui::init()?)?;
            tui::restore()?;

            Ok(())
        }
        _ => {
            // TODO: replace timer with pomodoro
            let mut timer = Timer::new(Duration::from_secs(60));
            timer.run(&mut tui::init()?)?;
            tui::restore()?;

            Ok(())
        }
    }
}
