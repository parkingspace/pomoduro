/*
TODO:
- add unit tests
- Ratatui
*/
mod parser;
mod timer;

use clap::Parser;
use parser::parse_duration;
use timer::Timer;

#[derive(Parser)]
#[command(name = "Pomodoro Timer")]
#[command(version = "0.1")]
#[command(about = "Time", long_about = None)]
struct Cli {
    #[arg(value_parser = parse_duration, short, long)]
    duration: u64,
}

fn main() {
    let cli = Cli::parse();

    let time_in_minutes = cli.duration * 60;

    let timer = Timer::new(time_in_minutes);
    timer.countdown();
}
