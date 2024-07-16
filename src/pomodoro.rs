use std::io;
use std::time::Duration;

use crate::{
    timer::{Timer, TimerStatus},
    tui,
};

pub struct Pomodoro {
    sessions: usize,
    current_session: usize,
    focus_duration: Duration,
    break_duration: Duration,
    is_focus: bool,
    timer: Timer,
}

impl Pomodoro {
    pub fn new(sessions: usize, focus_duration: Duration, break_duration: Duration) -> Self {
        Pomodoro {
            sessions,
            current_session: 1,
            focus_duration,
            break_duration,
            is_focus: true,
            timer: Timer::new(focus_duration, String::from("Focus")),
        }
    }

    pub fn run(&mut self, terminal: &mut tui::Tui, tick_rate: Duration) -> io::Result<()> {
        // loop until current session > total sessions
        while self.current_session <= self.sessions {
            self.timer.run(terminal, tick_rate)?;

            // when timer is finished,
            if self.timer.get_status() == TimerStatus::Done {
                // focus
                if self.is_focus {
                    self.is_focus = false;
                    self.timer = Timer::new(self.break_duration, String::from("Break"));
                } else {
                    // break
                    self.is_focus = true;
                    self.timer = Timer::new(self.focus_duration, String::from("Focus"));
                    self.current_session += 1;
                }
            }

            if self.timer.get_status() == TimerStatus::Exit {
                break;
            }
        }

        Ok(())
    }

    pub fn get_current_session(&self) -> usize {
        self.current_session
    }

    pub fn get_total_session(&self) -> usize {
        self.sessions
    }
}
