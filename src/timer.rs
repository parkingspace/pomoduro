use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use std::fmt;
use std::time::{Duration, Instant};

#[derive(PartialEq, Copy, Clone)]
pub enum TimerType {
    Single,
    Pomodoro,
}

#[derive(Clone)]
pub struct Timer {
    status: TimerStatus,
    start: Instant,
    duration: Duration,
    name: String,
    timer_type: TimerType,
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
    pub fn new(t: Duration, name: String, timer_type: TimerType) -> Self {
        Timer {
            start: Instant::now(),
            duration: t,
            status: TimerStatus::Running,
            name,
            timer_type,
        }
    }

    fn key_to_action(key: KeyCode, modifiers: KeyModifiers) -> Option<TimerAction> {
        match key {
            KeyCode::Char('c') | KeyCode::Char('C') if modifiers == KeyModifiers::CONTROL => {
                Some(TimerAction::Quit)
            }
            KeyCode::Char('q') => Some(TimerAction::Quit),
            KeyCode::Char('p') => Some(TimerAction::Pause),
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
        }
    }

    // pub fn run(&mut self, terminal: &mut tui::Tui, tick_rate: Duration) -> io::Result<()> {
    //     let mut last_tick = Instant::now();
    //
    //     while self.get_status() != TimerStatus::Exit {
    //         terminal.draw(|f| ui::render(f, self))?;
    //
    //         let timeout = tick_rate.saturating_sub(last_tick.elapsed());
    //         if event::poll(timeout)? {
    //             if let Event::Key(key) = event::read()? {
    //                 if key.kind == event::KeyEventKind::Press {
    //                     if let Some(action) = Self::key_to_action(key.code, key.modifiers) {
    //                         self.handle_action(action)
    //                     }
    //                 }
    //             }
    //         }
    //
    //         if last_tick.elapsed() >= tick_rate {
    //             // update the status: check if timer is done
    //             if self.status == TimerStatus::Running && self.is_done() {
    //                 self.status = TimerStatus::Done;
    //                 // if self.timer_type == TimerType::Pomodoro {
    //                 break;
    //                 // }
    //             }
    //
    //             last_tick = Instant::now();
    //         }
    //     }
    //
    //     Ok(())
    // }

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

    pub fn get_timer_type(&self) -> TimerType {
        self.timer_type
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
