use std::io;
use std::time::Duration;

use crate::{
    timer::{Timer, TimerStatus},
    tui,
};

enum PomodoroState {
    Focus(usize),
    Break(usize),
    Completed,
}

pub struct Pomodoro {
    state: PomodoroState,
    focus_duration: Duration,
    break_duration: Duration,
    total_sessions: usize,
}

impl Pomodoro {
    pub fn new(total_sessions: usize, focus_duration: Duration, break_duration: Duration) -> Self {
        Pomodoro {
            state: PomodoroState::Focus(1),
            focus_duration,
            break_duration,
            total_sessions,
        }
    }

    pub fn next_timer(&mut self) -> Option<Timer> {
        match self.state {
            PomodoroState::Focus(session) if session <= self.total_sessions => {
                self.state = PomodoroState::Break(session + 1);
                Some(Timer::new(self.break_duration, "Break".to_string()))
            }
            PomodoroState::Break(session) if session < self.total_sessions => {
                self.state = PomodoroState::Focus(session + 1);
                Some(Timer::new(self.focus_duration, "Focus".to_string()))
            }
            // TODO: this should be long break
            PomodoroState::Break(session) if session == self.total_sessions => {
                self.state = PomodoroState::Completed;
                Some(Timer::new(self.break_duration, "Long Break".to_string()))
            }
            PomodoroState::Completed => None,
            _ => None,
        }
    }

    pub fn run(&mut self, terminal: &mut tui::Tui, tick_rate: Duration) -> io::Result<()> {
        while let Some(mut timer) = self.next_timer() {
            timer.run(terminal, tick_rate)?;
            if timer.get_status() == TimerStatus::Exit {
                break;
            }
        }

        Ok(())
    }

    // pub fn get_current_session(&self) -> usize {
    //     match self.state {
    //         PomodoroState::Focus(session) | PomodoroState::Break(session) => session,
    //         PomodoroState::Completed => self.total_sessions,
    //     }
    // }
    //
    // pub fn get_total_sessions(&self) -> usize {
    //     self.total_sessions
    // }
    //
    // pub fn is_focus(&self) -> bool {
    //     todo!()
    // }
    //
    // pub fn is_completed(&self) {
    //     todo!()
    // }
}
