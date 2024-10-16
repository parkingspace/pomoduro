use crate::event::{Event, Events};
use crate::pomodoro::{Pomodoro, PomodoroSession, PomodoroState};
use crate::timer::{Timer, TimerAction, TimerSession, TimerStatus};
use crate::tui;
use crate::ui;

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyModifiers};
use futures::{SinkExt, StreamExt};
use std::time::Duration;
use std::{io, net::SocketAddr};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

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

#[derive(Clone)]
pub struct WebSocketHandler {
    ws_to_app_sender: flume::Sender<TimerAction>,
    ws_to_app_receiver: flume::Receiver<TimerAction>,
    app_to_ws_sender: flume::Sender<TimerAction>,
    app_to_ws_receiver: flume::Receiver<TimerAction>,
}

impl WebSocketHandler {
    fn new() -> Self {
        let (app_to_ws_sender, app_to_ws_receiver) = flume::unbounded();
        let (ws_to_app_sender, ws_to_app_receiver) = flume::unbounded();

        WebSocketHandler {
            ws_to_app_sender,
            ws_to_app_receiver,
            app_to_ws_sender,
            app_to_ws_receiver,
        }
    }

    pub async fn host(self, addr: SocketAddr) {
        let listener = TcpListener::bind(addr).await.unwrap();
        println!("Listening on {}", addr);

        while let Ok((socket, _addr)) = listener.accept().await {
            let ws_stream = tokio_tungstenite::accept_async(socket)
                .await
                .expect("Error during the websocket handshake occurred");

            // NOTE: incoming: read from client
            // outgoing: write to client
            // how to read outgone? use ws_to_app_receiver
            // Problems:
            // outgoing is not being used -> where should I use then?
            // outgoing must be used when the client(from Join) triggers an event
            // how do I know when the user presses a key?
            // -> when user presses a key, pass it via app_to_ws_sender.send()
            let (mut outgoing, mut incoming) = ws_stream.split();

            loop {
                tokio::select! {
                    Some(message) = incoming.next() => {
                        if let Ok(message) = message {
                            let msg = serde_json::from_str(&message.to_string()).unwrap();
                            println!("HOST: Message from client: {:?}", message);
                            self.ws_to_app_sender.send_async(msg).await.unwrap();
                        }
                    }
                    Ok(action) = self.app_to_ws_receiver.recv_async() => {
                        println!("Received via app_to_ws_receiver: {:?}", action);
                        let msg = serde_json::to_string(&action).unwrap();
                        if outgoing.send(Message::text(msg)).await.is_err() {
                            break; // Failed to send, assume connection is closed
                        }
                    }
                }
            }
        }
    }

    pub async fn join(self, addr: &str) {
        let ws_addr = format!("ws://{}", addr).into_client_request().unwrap();

        let (ws_stream, _) = tokio_tungstenite::connect_async(ws_addr)
            .await
            .expect("Failed to connect");

        let (mut _outgoing, mut incoming) = ws_stream.split();

        tokio::spawn(async move {
            while let Some(Ok(message)) = incoming.next().await {
                let message = serde_json::from_str(&message.to_string()).unwrap();
                println!("JOIN: Message from client: {:?}", message);
                self.ws_to_app_sender.send_async(message).await.unwrap();
            }
        });
    }
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
                                        ws_handler.app_to_ws_sender.send_async(action).await.unwrap();
                                        self.handle_action(action);
                                    }
                                }
                                _ => ()
                            }
                        }
                        Ok(app_message) = ws_handler.ws_to_app_receiver.recv_async() => {
                            println!("YAY!!! Received via ws_to_app_receiver: {:?}", app_message);
                            self.handle_ws_message(app_message);
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
        // TODO: refactor this
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
