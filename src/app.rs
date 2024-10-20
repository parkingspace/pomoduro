use crate::event::{Event, Events};
use crate::pomodoro::{Pomodoro, PomodoroSession, PomodoroState};
use crate::timer::{Timer, TimerAction, TimerSession, TimerStatus};
use crate::tui;
use crate::ui;
use crate::websocket::{TimerMessage, WebSocketHandler};

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyModifiers};
use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use tracing::debug;

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

enum SessionType {
    SingleUser,
    Shared(WebSocketHandler),
}

pub trait Session: Send {
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
    session_type: SessionType,
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
            session_type: SessionType::SingleUser,
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
            session_type: SessionType::SingleUser,
        }
    }

    pub fn new_shared_pomodoro(
        total_sessions: usize,
        focus_duration: Duration,
        break_duration: Duration,
        long_break_duration: Duration,
        tick_rate: Duration,
    ) -> (Self, WebSocketHandler) {
        let ws_handler = WebSocketHandler::new();

        let ws_handler_clone = ws_handler.clone();

        (
            App {
                session: Box::new(PomodoroSession::new(
                    total_sessions,
                    focus_duration,
                    break_duration,
                    long_break_duration,
                )),
                tick_rate,
                mode: Mode::Pomodoro,
                session_type: SessionType::Shared(ws_handler_clone),
            },
            ws_handler,
        )
    }

    pub async fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        let mut events = Events::new();

        loop {
            match &self.session_type {
                SessionType::SingleUser => {
                    if let Some(event) = events.next().await {
                        match event {
                            Event::Tick => {
                                self.session.tick();
                            }
                            Event::Render => {
                                terminal.draw(|f| ui::render(f, self))?;
                            }
                            Event::Crossterm(CrosstermEvent::Key(key)) => {
                                if let Some(action) = self.key_to_action(key.code, key.modifiers) {
                                    self.handle_action(action);
                                }
                            }
                            _ => (),
                        }
                    }
                }
                SessionType::Shared(ws_handler) => {
                    let local_addr = *ws_handler.local_addr.lock().await;
                    let local_addr =
                        local_addr.unwrap_or_else(|| SocketAddr::from(([0, 0, 0, 0], 0)));

                    tokio::select! {
                        Some(event) = events.next() => {
                            match event {
                                Event::Tick => {
                                    self.session.tick();
                                }
                                Event::Render => {
                                    terminal.draw(|f| ui::render(f, self))?;
                                }
                                Event::Crossterm(CrosstermEvent::Key(key)) => {
                                    if let Some(action) = self.key_to_action(key.code, key.modifiers) {
                                        let timer_message = TimerMessage { action, sender: local_addr };
                                        ws_handler.app_to_ws_sender.send_async(timer_message.clone()).await.unwrap();
                                        debug!("{:?} - APP(APP_TO_WS): Action({:?}) SENT TO WS", timer_message.sender, action);
                                        self.handle_action(timer_message.action);
                                    }
                                }
                                _ => ()
                            }
                        }
                        Ok(timer_message) = ws_handler.ws_to_app_receiver.recv_async() => {
                            debug!("{:?} - APP(WS_TO_APP): Message RECEIVED FROM WS: {:?}", local_addr, timer_message);
                            self.handle_ws_message(timer_message.action);
                        }
                    }
                }
            }

            if self.should_quit() {
                break;
            }
        }

        Ok(())
    }

    fn handle_ws_message(&mut self, message: TimerAction) {
        self.handle_action(message);
    }

    fn should_quit(&self) -> bool {
        // NOTE: timer: this returns true if the TimerStatus is Exit
        // pomodoro: this returns true if the PomodoroState is Completed
        // Exit and Completed are different things so they must be handled separately
        self.session.is_finished()
    }

    fn key_to_action(&self, key: KeyCode, modifiers: KeyModifiers) -> Option<TimerAction> {
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
            TimerAction::Quit => {
                timer.set_status(TimerStatus::Exit);
                if let Some(pomodoro) = self.session.get_pomodoro() {
                    pomodoro.set_state(PomodoroState::Completed)
                }
            }
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
