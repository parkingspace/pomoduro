use std::fmt;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Timer {
    status: TimerStatus,
    start: Instant,
    duration: Duration,
    name: String,
}

#[derive(PartialEq, Copy, Clone)]
pub enum TimerStatus {
    Running,
    Paused,
    Done, // when single timer is done
    Exit, // when the program is exited
}

pub enum TimerAction {
    Start,
    Pause,
    Quit,
}

impl Timer {
    pub fn new(t: Duration, name: String) -> Self {
        Timer {
            start: Instant::now(),
            duration: t,
            status: TimerStatus::Running,
            name,
        }
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

    pub fn set_status(&mut self, status: TimerStatus) {
        self.status = status
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

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn format_duration(&self, total_seconds: Duration) -> String {
        let total_seconds = total_seconds.as_secs();
        match total_seconds {
            0..=3599 => {
                let minutes = (total_seconds % 3600) / 60;
                let seconds = total_seconds % 60;

                match minutes {
                    0 => format!("{}s", seconds),
                    _ => format!("{}m {}s", minutes, seconds),
                }
            }
            3600..=86399 => {
                let hours = total_seconds / 3600;
                let minutes = (total_seconds % 3600) / 60;
                let seconds = total_seconds % 60;

                match hours {
                    0 => format!("{}m {}s", minutes, seconds),
                    _ => format!("{}h {}m {}s", hours, minutes, seconds),
                }
            }
            _ => {
                let days = total_seconds / 86400;
                let hours = (total_seconds % 86400) / 3600;
                let minutes = (total_seconds % 3600) / 60;
                let seconds = total_seconds % 60;

                format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
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
