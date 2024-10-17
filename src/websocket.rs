use crate::timer::TimerAction;

use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

#[derive(Clone)]
pub struct WebSocketHandler {
    pub ws_to_app_sender: flume::Sender<TimerAction>,
    pub ws_to_app_receiver: flume::Receiver<TimerAction>,
    pub app_to_ws_sender: flume::Sender<TimerAction>,
    pub app_to_ws_receiver: flume::Receiver<TimerAction>,
}

impl WebSocketHandler {
    pub fn new() -> Self {
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
                            break;
                        }
                    }
                    else => break,
                }
            }
        }
    }

    pub async fn join(self, addr: &str) {
        let ws_addr = format!("ws://{}", addr).into_client_request().unwrap();

        let (ws_stream, _) = tokio_tungstenite::connect_async(ws_addr)
            .await
            .expect("Failed to connect");

        let (mut outgoing, mut incoming) = ws_stream.split();

        loop {
            tokio::select! {
                Some(Ok(message)) = incoming.next() => {
                    let message = serde_json::from_str(&message.to_string()).unwrap();
                    println!("JOIN: Message from client: {:?}", message);
                    self.ws_to_app_sender.send_async(message).await.unwrap();
                }
                Ok(action) = self.app_to_ws_receiver.recv_async() => {
                    let msg = serde_json::to_string(&action).unwrap();
                    if outgoing.send(Message::text(msg)).await.is_err() {
                        break;
                    }
                }
                else => break,
            }
        }
    }
}
