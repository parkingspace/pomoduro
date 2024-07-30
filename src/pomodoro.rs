use std::time::Duration;

use crate::app::Session;
use crate::timer::{Timer, TimerStatus};

#[derive(PartialEq, Clone, Copy)]
pub enum PomodoroState {
    Ready,
    Focus(usize),
    Break(usize),
    LongBreak(usize),
    Completed,
}

#[derive(Clone)]
pub struct Pomodoro {
    state: PomodoroState,
    focus_duration: Duration,
    break_duration: Duration,
    long_break_duration: Duration,
    total_sessions: usize,
    timer: Option<Timer>,
}

impl Pomodoro {
    pub fn new(
        total_sessions: usize,
        focus_duration: Duration,
        break_duration: Duration,
        long_break_duration: Duration,
        timer: Timer,
    ) -> Self {
        Pomodoro {
            // state: PomodoroState::Ready,
            state: PomodoroState::Focus(1),
            focus_duration,
            break_duration,
            long_break_duration,
            total_sessions,
            timer: Some(timer),
        }
    }

    pub fn tick(&mut self) {
        if let Some(timer) = &mut self.timer {
            if timer.get_status() == TimerStatus::Exit {
                self.state = PomodoroState::Completed;
            } else if timer.is_done() {
                self.timer = self.next_timer();
            }
        }
    }

    fn next_timer(&mut self) -> Option<Timer> {
        match self.state {
            PomodoroState::Ready => {
                self.state = PomodoroState::Focus(1);
                let new_timer = Timer::new(self.focus_duration, "Focus".to_string());
                self.timer = Some(new_timer.clone());
                Some(new_timer)
            }
            PomodoroState::Focus(session) if session <= self.total_sessions => {
                self.state = PomodoroState::Break(session);
                let new_timer = Timer::new(self.break_duration, "Break".to_string());
                self.timer = Some(new_timer.clone());
                Some(new_timer)
            }
            PomodoroState::Break(session) if session < self.total_sessions => {
                self.state = PomodoroState::Focus(session + 1);
                let new_timer = Timer::new(self.focus_duration, "Focus".to_string());
                self.timer = Some(new_timer.clone());
                Some(new_timer)
            }
            PomodoroState::Break(session) if session == self.total_sessions => {
                self.state = PomodoroState::LongBreak(session);
                let new_timer = Timer::new(self.long_break_duration, "Long Break".to_string());
                self.timer = Some(new_timer.clone());
                Some(new_timer)
            }
            PomodoroState::LongBreak(_) => {
                self.state = PomodoroState::Completed;
                self.timer = None;
                None
            }
            PomodoroState::Completed => {
                self.timer = None;
                None
            }
            _ => None,
        }
    }

    pub fn get_timer(&mut self) -> Option<&mut Timer> {
        self.timer.as_mut()
    }

    pub fn get_current_session(&self) -> usize {
        match self.state {
            PomodoroState::Ready => 0,
            PomodoroState::Focus(session) | PomodoroState::Break(session) => session,
            PomodoroState::Completed => self.total_sessions,
            PomodoroState::LongBreak(_) => self.total_sessions,
        }
    }

    pub fn get_total_sessions(&self) -> usize {
        self.total_sessions
    }

    pub fn is_focus(&self) -> bool {
        matches!(self.state, PomodoroState::Focus(_))
    }

    pub fn is_completed(&self) -> bool {
        self.state == PomodoroState::Completed
    }

    pub fn get_state(&self) -> PomodoroState {
        self.state
    }

    pub fn set_state(&mut self, state: PomodoroState) {
        self.state = state;
    }
}

pub struct PomodoroSession {
    pomodoro: Pomodoro,
}

impl PomodoroSession {
    pub fn new(
        total_sessions: usize,
        focus_duration: Duration,
        break_duration: Duration,
        long_break_duration: Duration,
    ) -> Self {
        let first_timer = Timer::new(focus_duration, "Focus".to_string());

        let pomodoro = Pomodoro::new(
            total_sessions,
            focus_duration,
            break_duration,
            long_break_duration,
            first_timer,
        );

        PomodoroSession { pomodoro }
    }
}

impl Session for PomodoroSession {
    fn tick(&mut self) {
        self.pomodoro.tick()
    }

    fn is_finished(&self) -> bool {
        self.pomodoro.is_completed()
    }

    fn toggle_pause(&mut self) {
        if let Some(timer) = &mut self.pomodoro.get_timer() {
            if timer.get_status() == TimerStatus::Running {
                timer.set_status(TimerStatus::Paused)
            } else if timer.get_status() == TimerStatus::Paused {
                timer.set_status(TimerStatus::Running)
            }
        }
    }

    fn get_timer(&mut self) -> Option<&mut Timer> {
        self.pomodoro.get_timer()
    }
}
