use crate::tui;
use crate::ui;
use crossterm::event::{self, Event, KeyCode};
use std::fmt;
use std::io;
use std::time::{Duration, Instant};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TimerError {
    #[error("Elapsed time is greater than or equal to the duration")]
    ElapsedExceedsDuration,
}

pub struct Timer {
    status: TimerStatus,
    start: Instant,
    duration: Duration,
}

#[derive(PartialEq, Copy, Clone)]
pub enum TimerStatus {
    Running,
    Paused,
    Done,
    Exit,
}

enum TimerAction {
    Start,
    Pause,
    Quit,
}

impl Timer {
    pub fn new(t: Duration) -> Self {
        Timer {
            start: Instant::now(),
            duration: t,
            status: TimerStatus::Running,
    fn handle_action(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => {
                self.status = TimerStatus::Exit;
            }
            KeyCode::Char('p') => {
                if self.status == TimerStatus::Running {
                    self.status = TimerStatus::Paused;
                } else if self.status == TimerStatus::Paused {
                    self.status = TimerStatus::Running;
                }
            }
            _ => {}
        }
    }

    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while self.get_status() != TimerStatus::Exit {
            terminal.draw(|f| ui::render(f, self))?;
            if self.is_done() {
                self.status = TimerStatus::Done;

                // TODO: decide what to show when timer is done
                println!("Timer is done");
                break;
            }
            // wait for 1 second
            if event::poll(Duration::from_millis(1000))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == event::KeyEventKind::Press {
                        self.handle_action(key.code);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_elapsed_time(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn is_done(&self) -> bool {
        self.start.elapsed() >= self.duration
    }

    pub fn get_status(&self) -> TimerStatus {
        self.status
    }

    pub fn get_remaining_time(&self) -> Result<u64, TimerError> {
        self.duration
            .as_secs()
            .checked_sub(self.start.elapsed().as_secs())
            .ok_or(TimerError::ElapsedExceedsDuration)
    }

    pub fn get_duration(&self) -> Duration {
        self.duration
    }
    fn format_duration(&self, duration: u64) -> String {
        match duration {
            0..=3599 => {
                let minutes = (duration % 3600) / 60;
                let seconds = duration % 60;
                format!("{:02}:{:02}", minutes, seconds)
            }
            3600..=86399 => {
                let hours = duration / 3600;
                let minutes = (duration % 3600) / 60;
                let seconds = duration % 60;
                format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
            }
            _ => {
                let days = duration / 86400;
                let hours = (duration % 86400) / 3600;
                let minutes = (duration % 3600) / 60;
                let seconds = duration % 60;
                let day_str = if days == 1 { "day" } else { "days" };
                format!(
                    "{} {}, {:02}:{:02}:{:02}",
                    days, day_str, hours, minutes, seconds
                )
            }
        }
    }
}

impl fmt::Display for Timer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.get_remaining_time() {
            Ok(duration) => write!(f, "{}", self.format_duration(duration)),
            Err(e) => write!(f, "{}", e),
        }
    }
}

impl fmt::Display for TimerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimerStatus::Running => write!(f, "Running"),
            TimerStatus::Paused => write!(f, "Paused"),
            TimerStatus::Done => write!(f, "Done"),
            TimerStatus::Exit => write!(f, "Exit"),
        }
    }
}
