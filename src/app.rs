use crossterm::event::KeyModifiers;
use crossterm::event::{self, Event, KeyCode};

use crate::pomodoro::{Pomodoro, PomodoroState};
use crate::timer::{Timer, TimerAction, TimerSession, TimerStatus};
use crate::tui;
use crate::ui;
use std::io;
use std::time::{Duration, Instant};

pub trait Session {
    fn tick(&mut self);
    fn is_finished(&self) -> bool;
    fn toggle_pause(&mut self);
    fn get_timer(&mut self) -> Option<&mut Timer>;
}

pub struct PomodoroSession {
    pomodoro: Pomodoro,
    current_timer: Option<Timer>,
}

impl PomodoroSession {
    fn new(
        total_sessions: usize,
        focus_duration: Duration,
        break_duration: Duration,
        long_break_duration: Duration,
    ) -> Self {
        let first_timer = Timer::new(focus_duration, "Focus".to_string());
        let current_timer = first_timer.clone();

        let pomodoro = Pomodoro::new(
            total_sessions,
            focus_duration,
            break_duration,
            long_break_duration,
            first_timer,
        );

        PomodoroSession {
            pomodoro,
            current_timer: Some(current_timer),
        }
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
        if let Some(timer) = &mut self.current_timer {
            if timer.get_status() == TimerStatus::Running {
                timer.set_status(TimerStatus::Paused)
            } else if timer.get_status() == TimerStatus::Paused {
                timer.set_status(TimerStatus::Running)
            }
        }
    }

    fn get_timer(&mut self) -> Option<&mut Timer> {
        self.current_timer.as_mut()
    }
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

    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        let mut last_tick = Instant::now();

        loop {
            terminal.draw(|f| ui::render(f, self))?;

            let timeout = self.tick_rate.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == event::KeyEventKind::Press {
                        if let Some(action) = Self::key_to_action(key.code, key.modifiers) {
                            self.handle_action(action)
                        }
                    }
                }
            }

            if last_tick.elapsed() >= self.tick_rate {
                self.session.tick();
                last_tick = Instant::now();
            }

            if self.should_quit() {
                break;
            }
        }

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
}
