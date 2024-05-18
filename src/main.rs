use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

pub struct Timer {
    start: Instant,
    remaining: u64,
}

impl Timer {
    pub fn new(t: u64) -> Self {
        Self {
            start: Instant::now(),
            remaining: t,
        }
    }

    pub fn countdown(&self) {
        while self.start.elapsed().as_secs() <= self.remaining {
            let remaining = self.remaining - self.start.elapsed().as_secs();
            let hours = remaining / 3600;
            let minutes = (remaining % 3600) / 60;
            let seconds = remaining % 60;
            print!("\r{:02}:{:02}:{:02}", hours, minutes, seconds);
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(1000));
        }
    }
}

fn main() {
    let timer = Timer {
        start: Instant::now(),
        remaining: 1000,
    };

    timer.countdown();
}
