use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind, KeyModifiers};

use crate::event::{Event, Events};
use crate::pomodoro::{Pomodoro, PomodoroSession};
use crate::timer::{Timer, TimerAction, TimerSession, TimerStatus};
use crate::tui;
use crate::ui;
use std::io;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub timer: TimerInfo,
    pub pomodoro: Option<PomodoroInfo>,
}

#[derive(Debug, Clone)]
pub struct TimerInfo {
    pub name: String,
    pub remaining: Duration,
}

#[derive(Debug, Clone)]
pub struct PomodoroInfo {
    pub total_sessions: usize,
    pub current_session: usize,
}

pub trait Session {
    fn tick(&mut self);
    fn is_finished(&self) -> bool;
    fn toggle_pause(&mut self);
    fn get_timer(&mut self) -> Option<&mut Timer>;
    fn get_pomodoro(&mut self) -> Option<&mut Pomodoro>;
}

pub struct App {
    session: Box<dyn Session>,
    tick_rate: Duration,
    mode: Mode,
}

pub enum Mode {
    Timer,
    Pomodoro,
}

impl App {
    pub fn new_timer(duration: Duration, name: String, tick_rate: Duration) -> Self {
        App {
            session: Box::new(TimerSession::new(duration, name)),
            tick_rate,
            mode: Mode::Timer,
        }
    }

    pub fn new_pomodoro(
        total_sessions: usize,
        focus_duration: Duration,
        break_duration: Duration,
        long_break_duration: Duration,
        tick_rate: Duration,
    ) -> Self {
        App {
            session: Box::new(PomodoroSession::new(
                total_sessions,
                focus_duration,
                break_duration,
                long_break_duration,
            )),
            tick_rate,
            mode: Mode::Pomodoro,
        }
    }

    pub async fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        let mut events = Events::new();

        loop {
            if let Some(event) = events.next().await {
                match event {
                    Event::Tick => {
                        self.session.tick();
                    }
                    Event::Render => {
                        terminal.draw(|f| ui::render(f, self))?;
                    }
                    _ => {
                        self.handle_event(event)?;
                    }
                }
            }

            if self.should_quit() {
                break;
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> io::Result<()> {
        if let Event::Crossterm(CrosstermEvent::Key(key)) = event {
            if let Some(action) = Self::key_to_action(key.code, key.modifiers) {
                self.handle_action(action);
            }
        };

        Ok(())
    }

    fn should_quit(&self) -> bool {
        // NOTE: timer: this returns true if the TimerStatus is Exit
        // pomodoro: this returns true if the PomodoroState is Completed
        // Exit and Completed are different things so they must be handled separately
        // TODO: refactor this
        self.session.is_finished()
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
        let timer = self.get_timer().unwrap();

        match action {
            TimerAction::Quit => timer.set_status(TimerStatus::Exit),
            TimerAction::Pause => {
                self.session.toggle_pause();
            }
        }
    }

    pub fn get_timer(&mut self) -> Option<&mut Timer> {
        self.session.get_timer()
    }

    pub fn get_session_info(&mut self) -> SessionInfo {
        match self.mode {
            Mode::Timer => SessionInfo {
                timer: self.get_timer_info(),
                pomodoro: None,
            },
            Mode::Pomodoro => SessionInfo {
                timer: self.get_timer_info(),
                pomodoro: self.get_pomodoro_info(),
            },
        }
    }

    fn get_timer_info(&mut self) -> TimerInfo {
        self.session
            .get_timer()
            .map(|timer| TimerInfo {
                name: timer.get_name().to_string(),
                remaining: timer.remaining_time(),
            })
            .unwrap()
    }

    fn get_pomodoro_info(&mut self) -> Option<PomodoroInfo> {
        self.session.get_pomodoro().map(|pomodoro| PomodoroInfo {
            total_sessions: pomodoro.get_total_sessions(),
            current_session: pomodoro.get_current_session(),
        })
    }
}
