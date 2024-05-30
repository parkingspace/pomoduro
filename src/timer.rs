use crate::tui;
use crate::ui;
use crossterm::event::{self, Event, KeyCode};
use std::io;
use std::time::{Duration, Instant};

pub struct Timer {
    status: TimerStatus,
    start: Instant,
    duration: Duration,
}

#[derive(PartialEq, Copy, Clone)]
pub enum TimerStatus {
    Running,
    Paused,
    Done,
    Exit,
}

enum TimerAction {
    Start,
    Pause,
    Quit,
}

impl Timer {
    pub fn new(t: Duration) -> Self {
        Timer {
            start: Instant::now(),
            duration: t,
            status: TimerStatus::Running,
        }
    }

    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while self.get_status() != TimerStatus::Exit {
            terminal.draw(|f| ui::render(f, self))?;
            // wait for 1 second
            if event::poll(Duration::from_millis(1000))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == event::KeyEventKind::Release {
                        continue;
                    }

                    match key.code {
                        KeyCode::Char('q') => {
                            self.status = TimerStatus::Exit;
                        }
                        KeyCode::Char('p') => {
                            self.status = TimerStatus::Paused;
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    pub fn is_done(&self) -> bool {
        self.start.elapsed() >= self.duration
    }

    pub fn get_status(&self) -> TimerStatus {
        self.status
    }

    pub fn get_remaining_time(&self) -> Duration {
        self.duration.saturating_sub(self.start.elapsed())
    }

    pub fn get_duration(&self) -> Duration {
        self.duration
    }
}
