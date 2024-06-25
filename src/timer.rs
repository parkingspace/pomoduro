use crate::tui;
use crate::ui;
use crossterm::event::{self, Event, KeyCode};
use std::fmt;
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
            if self.is_done() {
                self.status = TimerStatus::Done;

                // TODO: decide what to show when timer is done
                println!("Timer is done");
                break;
            }
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

    pub fn get_elapsed_time(&self) -> Duration {
        self.start.elapsed()
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

impl fmt::Display for Timer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let duration = self.get_remaining_time().as_secs();
        match duration {
            0..=3599 => {
                let minutes = (duration % 3600) / 60;
                let seconds = duration % 60;
                write!(f, "{:02}:{:02}", minutes, seconds)
            }
            3600..=86399 => {
                let hours = duration / 3600;
                let minutes = (duration % 3600) / 60;
                let seconds = duration % 60;
                write!(f, "{:02}:{:02}:{:02}", hours, minutes, seconds)
            }
            _ => {
                let days = duration / 86400;
                let hours = (duration % 86400) / 3600;
                let minutes = (duration % 3600) / 60;
                let seconds = duration % 60;
                let day_str = if days == 1 { "day" } else { "days" };
                write!(
                    f,
                    "{} {}, {:02}:{:02}:{:02}",
                    days, day_str, hours, minutes, seconds
                )
            }
        }
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
