/*
TODO:
- add unit tests
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
#[command(about = "Time", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Start the timer")]
    Timer {
        #[arg(value_parser = parse_duration, short, long)]
        time: Duration,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Timer { time }) => {
            let timer = Timer::new(*time);
            timer.countdown();
        }
        None => {
            println!("No command provided");
        }
    }
}
