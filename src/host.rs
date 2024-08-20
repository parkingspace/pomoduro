use crate::app::{App, SessionInfo};
use crate::timer::TimerAction;

use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};

struct Host {
    app: App,
    state_tx: broadcast::Sender<SessionInfo>,
    command_rx: mpsc::Receiver<TimerAction>,
}

impl Host {
    async fn run(&mut self) {
        self.app = App::new_pomodoro(
            1,
            Duration::from_secs(3),
            Duration::from_secs(3),
            Duration::from_secs(3),
            Duration::from_millis(250),
        );

        // todo
    }
}
