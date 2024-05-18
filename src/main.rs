use core::fmt;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

pub struct Timer {
    start: Instant,
    remaining: u64,
}

impl Timer {
    pub fn new(t: u64) -> Self {
        Timer {
            start: Instant::now(),
            remaining: t,
        }
    }

    pub fn countdown(&self) {
        while self.start.elapsed().as_secs() <= self.remaining {
            print!("\r{}", self);
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(1000));
        }
    }
}

impl fmt::Display for Timer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let remaining = self.remaining - self.start.elapsed().as_secs();
        let hours = remaining / 3600;
        let minutes = (remaining % 3600) / 60;
        let seconds = remaining % 60;
        write!(f, "{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

fn main() {
    let timer = Timer {
        start: Instant::now(),
        remaining: 1000,
    };

    timer.countdown();
}
