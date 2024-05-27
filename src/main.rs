/*
TODO:
- Ratatui
*/
mod parser;
mod timer;

use clap::{Parser, Subcommand};
use parser::parse_duration;
use std::time::Duration;
use timer::Timer;

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
    #[command(about = "Start the timer")]
    Timer {
        #[arg(value_parser = parse_duration, short, long)]
        duration: Duration,
    },

    Start {
        #[arg(value_parser = parse_duration, short, long)]
        duration: Option<Duration>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Timer { duration }) => {
            let timer = Timer::new(*duration);
            timer.countdown();
        }
        Some(Commands::Start { duration }) => {
            let focus_duration = duration.unwrap_or(Duration::from_secs(5));
            let focus_timer = Timer::new(focus_duration);
            focus_timer.countdown();

            let break_duration = Duration::from_secs(5);
            let break_timer = Timer::new(break_duration);
            break_timer.countdown();
        }
        None => {
            println!("No command provided");
        }
    }
}
