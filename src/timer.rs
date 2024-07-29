use std::fmt;
use std::time::{Duration, Instant};

const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_DAY: u64 = 24 * SECONDS_PER_HOUR;

#[derive(Clone)]
pub struct Timer {
    status: TimerStatus,
    started_at: Instant,
    elapsed: Duration,
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
    Pause,
    Quit,
}

impl Timer {
    pub fn new(t: Duration, name: String) -> Self {
        Timer {
            started_at: Instant::now(),
            elapsed: Duration::ZERO,
            duration: t,
            status: TimerStatus::Running,
            name,
        }
    }

    pub fn tick(&mut self) {
        if self.status == TimerStatus::Running && self.is_done() {
            self.status = TimerStatus::Exit;
        }
    }

    pub fn elapsed_time(&self) -> Duration {
        match self.status {
            TimerStatus::Running => self.elapsed + self.started_at.elapsed(),
            TimerStatus::Paused | TimerStatus::Done => self.elapsed,
            _ => self.elapsed,
        }
    }

    pub fn toggle_pause(&mut self) {
        match self.status {
            TimerStatus::Running => {
                self.elapsed += self.started_at.elapsed();
                self.status = TimerStatus::Paused;
            }
            TimerStatus::Paused => {
                self.started_at = Instant::now();
                self.status = TimerStatus::Running;
            }
            _ => {}
        }
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
            0..SECONDS_PER_HOUR => {
                let minutes = (total_seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
                let seconds = total_seconds % SECONDS_PER_MINUTE;

                match minutes {
                    0 => format!("{}s", seconds),
                    _ => format!("{}m {}s", minutes, seconds),
                }
            }
            SECONDS_PER_HOUR..SECONDS_PER_DAY => {
                let hours = total_seconds / SECONDS_PER_HOUR;
                let minutes = (total_seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
                let seconds = total_seconds % SECONDS_PER_MINUTE;

                match hours {
                    0 => format!("{}m {}s", minutes, seconds),
                    _ => format!("{}h {}m {}s", hours, minutes, seconds),
                }
            }
            _ => {
                let days = total_seconds / SECONDS_PER_DAY;
                let hours = (total_seconds % SECONDS_PER_DAY) / SECONDS_PER_HOUR;
                let minutes = (total_seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
                let seconds = total_seconds % SECONDS_PER_MINUTE;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        let timer = Timer::new(Duration::from_secs(0), "Test".to_string());
        assert_eq!(timer.format_duration(Duration::ZERO), "0s");

        assert_eq!(timer.format_duration(Duration::from_secs(1)), "1s");
        assert_eq!(
            timer.format_duration(Duration::from_secs(SECONDS_PER_MINUTE - 1)),
            "59s"
        );
        assert_eq!(
            timer.format_duration(Duration::from_secs(SECONDS_PER_MINUTE)),
            "1m 0s"
        );
        assert_eq!(
            timer.format_duration(Duration::from_secs(SECONDS_PER_MINUTE + 1)),
            "1m 1s"
        );

        assert_eq!(
            timer.format_duration(Duration::from_secs(SECONDS_PER_HOUR - 1)),
            "59m 59s"
        );
        assert_eq!(
            timer.format_duration(Duration::from_secs(SECONDS_PER_HOUR)),
            "1h 0m 0s"
        );
        assert_eq!(
            timer.format_duration(Duration::from_secs(SECONDS_PER_HOUR + 1)),
            "1h 0m 1s"
        );

        assert_eq!(
            timer.format_duration(Duration::from_secs(SECONDS_PER_DAY - 1)),
            "23h 59m 59s"
        );
        assert_eq!(
            timer.format_duration(Duration::from_secs(SECONDS_PER_DAY)),
            "1d 0h 0m 0s"
        );
        assert_eq!(
            timer.format_duration(Duration::from_secs(SECONDS_PER_DAY + 1)),
            "1d 0h 0m 1s"
        );

        assert_eq!(
            timer.format_duration(Duration::from_secs(86400 * 365)),
            "365d 0h 0m 0s"
        );
    }
}
