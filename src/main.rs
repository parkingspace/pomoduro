/*
TODO:
- extract Timer into its own module
- add unit tests
- Clap
- Ratatui
*/
use clap::Parser;
use core::fmt;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

/**
By default the time argument is in minutes.
It can be used in seconds, days, or hours.

For example, 300s counts 5 minutes and 24h counts 1440 minutes.
*/
#[derive(Parser)]
#[command(name = "Pomodoro Timer")]
#[command(version = "0.1")]
#[command(about = "Time", long_about = None)]
struct Cli {
    #[arg(short, long)]
    duration: u64,
}

pub struct Timer {
    start: Instant,
    duration: u64,
}

impl Timer {
    pub fn new(t: u64) -> Self {
        Timer {
            start: Instant::now(),
            duration: t,
        }
    }

    pub fn countdown(&self) {
        while self.start.elapsed().as_secs() <= self.duration {
            print!("\r{}", self);
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(1000));
        }
    }
}

// FIX: timer doesn't flush out correctly when it's more than an 1 day
// TODO: handle singular/plural cases
impl fmt::Display for Timer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let duration = self.duration - self.start.elapsed().as_secs();
        match duration {
            0..=3600 => {
                let minutes = (duration % 3600) / 60;
                let seconds = duration % 60;
                write!(f, "{:02}:{:02}", minutes, seconds)
            }
            3601..=86400 => {
                let hours = duration / 3600;
                let minutes = (duration % 3600) / 60;
                let seconds = duration % 60;
                write!(f, "{:02}:{:02}:{:02}", hours, minutes, seconds)
            }
            _ => {
                let days = duration / 86400;
                let hours = (duration % 86400) / 3600;
                let minutes = (duration % 3600) / 60;
                let seconds = duration % 60;
                write!(
                    f,
                    "{} days, {:02}:{:02}:{:02}",
                    days, hours, minutes, seconds
                )
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let time_in_minutes = cli.duration * 60;

    let timer = Timer::new(time_in_minutes);
    timer.countdown();
}
