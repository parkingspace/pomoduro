use crate::tui;
use crate::ui;
use crossterm::event::{self, Event, KeyCode};
use std::fmt;
use std::io;
use std::time::{Duration, Instant};

pub struct Timer {
    status: TimerStatus,
    start: Instant,
    duration: Duration,
    display: TimerDisplay,
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
    ToggleDisplay,
    Quit,
}

#[derive(Copy, Clone)]
pub enum TimerDisplay {
    Remaining,
    Elapsed,
    Percentage,
}

impl Timer {
    pub fn new(t: Duration) -> Self {
        Timer {
            start: Instant::now(),
            duration: t,
            status: TimerStatus::Running,
            display: TimerDisplay::Remaining,
        }
    }

    fn key_to_action(key: KeyCode) -> Option<TimerAction> {
        match key {
            KeyCode::Char('q') => Some(TimerAction::Quit),
            KeyCode::Char('p') => Some(TimerAction::Pause),
            KeyCode::Char('t') => Some(TimerAction::ToggleDisplay),
            _ => None,
        }
    }

    fn handle_action(&mut self, action: TimerAction) {
        match action {
            TimerAction::Quit => self.status = TimerStatus::Exit,
            // TODO: add pause functionality
            TimerAction::Pause => {
                if self.status == TimerStatus::Running {
                    self.status = TimerStatus::Paused;
                }
            }
            TimerAction::Start => {
                if self.status == TimerStatus::Paused {
                    self.status = TimerStatus::Running;
                }
            }
            TimerAction::ToggleDisplay => {
                self.display = match self.display {
                    TimerDisplay::Remaining => TimerDisplay::Elapsed,
                    TimerDisplay::Elapsed => TimerDisplay::Percentage,
                    TimerDisplay::Percentage => TimerDisplay::Remaining,
                };
            }
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
                        if let Some(action) = Self::key_to_action(key.code) {
                            self.handle_action(action)
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn elapsed_time(&self) -> Duration {
        Instant::now().saturating_duration_since(self.start)
    }

    pub fn is_done(&self) -> bool {
        self.elapsed_time() >= self.duration
    }

    pub fn get_status(&self) -> TimerStatus {
        self.status
    }

    pub fn remaining_time(&self) -> Duration {
        let precise_remaining = self.duration.saturating_sub(self.elapsed_time());
        if precise_remaining.is_zero() {
            return Duration::ZERO;
        }

        // Round up for display: this is necessary because Duration includes fractional seconds
        Duration::from_secs(precise_remaining.as_secs().saturating_add(1))
    }

    pub fn get_duration(&self) -> Duration {
        self.duration
    }

    pub fn get_display(&self) -> TimerDisplay {
        self.display
    }

    fn format_duration(&self, total_seconds: Duration) -> String {
        let total_seconds = total_seconds.as_secs();
        match total_seconds {
            0..=3599 => {
                let minutes = (total_seconds % 3600) / 60;
                let seconds = total_seconds % 60;
                format!("{:02}:{:02}", minutes, seconds)
            }
            3600..=86399 => {
                let hours = total_seconds / 3600;
                let minutes = (total_seconds % 3600) / 60;
                let seconds = total_seconds % 60;
                format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
            }
            _ => {
                let days = total_seconds / 86400;
                let hours = (total_seconds % 86400) / 3600;
                let minutes = (total_seconds % 3600) / 60;
                let seconds = total_seconds % 60;
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
        write!(f, "{}", self.format_duration(self.remaining_time()))
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
