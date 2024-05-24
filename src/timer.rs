use core::fmt;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

pub struct Timer {
    start: Instant,
    duration: Duration,
}

impl Timer {
    pub fn new(t: Duration) -> Self {
        Timer {
            start: Instant::now(),
            duration: t,
        }
    }

    pub fn countdown(&self) {
        while self.start.elapsed() <= self.duration {
            /* NOTE:
            this does clear the line but not sure it's generic enough to use it.
            I'm going to use ratatui so maybe it's better to not worry.
            */
            print!("\r\x1B[2K{}", self);
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(1000));
        }
    }
}

impl fmt::Display for Timer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let duration = (self.duration - self.start.elapsed()).as_secs();
        match duration {
            0..=3599 => {
                let minutes = (duration % 3600) / 60;
                let seconds = duration % 60;
                write!(f, "{:02}:{:02}", minutes, seconds)
            }
            3600..=86399 => {
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
                let day_str = if days == 1 { "day" } else { "days" };
                write!(
                    f,
                    "{} {}, {:02}:{:02}:{:02}",
                    days, day_str, hours, minutes, seconds
                )
            }
        }
    }
}
