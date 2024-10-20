use crate::timer::TimerAction;

use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;
use tracing::debug;

type Sender = flume::Sender<TimerMessage>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Sender>>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimerMessage {
    pub action: TimerAction,
    pub sender: SocketAddr,
}

#[derive(Clone)]
pub struct WebSocketHandler {
    pub peer_map: PeerMap,
    pub ws_to_app_sender: flume::Sender<TimerMessage>,
    pub ws_to_app_receiver: flume::Receiver<TimerMessage>,
    pub app_to_ws_sender: flume::Sender<TimerMessage>,
    pub app_to_ws_receiver: flume::Receiver<TimerMessage>,
    pub local_addr: Arc<Mutex<Option<SocketAddr>>>,
}

impl WebSocketHandler {
    pub fn new() -> Self {
        let (app_to_ws_sender, app_to_ws_receiver) = flume::unbounded();
        let (ws_to_app_sender, ws_to_app_receiver) = flume::unbounded();

        WebSocketHandler {
            peer_map: Arc::new(Mutex::new(HashMap::new())),
            ws_to_app_sender,
            ws_to_app_receiver,
            app_to_ws_sender,
            app_to_ws_receiver,
            local_addr: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn host(self, addr: &SocketAddr) {
        let listener = TcpListener::bind(addr).await.unwrap();
        *self.local_addr.lock().await = Some(*addr);

        loop {
            tokio::select! {
                Ok((socket, peer_addr)) = listener.accept() => {

                    let ws_stream = tokio_tungstenite::accept_async(socket)
                        .await
                        .expect("Error during the websocket handshake occurred");

                    let handler_clone = self.clone();
                    tokio::spawn(async move {
                        handler_clone.handle_connection(peer_addr, ws_stream).await;
                    });
                }
            }
        }
    }

    async fn handle_connection(&self, addr: SocketAddr, ws_stream: WebSocketStream<TcpStream>) {
        let (mut outgoing, mut incoming) = ws_stream.split();
        let (pre_outgoing_sender, pre_outgoing_receiver) = flume::unbounded::<TimerMessage>();

        {
            let mut peer_map = self.peer_map.lock().await;
            peer_map.insert(addr, pre_outgoing_sender.clone());
            debug!("HOST(PEER_MAP): Inserted peer: {:?}", peer_map);
        }

        loop {
            tokio::select! {
                Some(message) = incoming.next() => {
                    if let Ok(message) = message {
                        let timer_message: TimerMessage = serde_json::from_str(&message.to_string()).unwrap();
                        debug!("HOST(INCOMING): Message RECEIVED FROM client: {:?}", timer_message);
                        self.ws_to_app_sender.send_async(timer_message.clone()).await.unwrap();
                        debug!("HOST(WS_TO_APP): Message SENT TO app");

                        self.broadcast(timer_message).await;
                    }
                }
                Ok(timer_message) = self.app_to_ws_receiver.recv_async() => {
                    debug!("HOST(APP_TO_WS): Message RECEIVED FROM app: {:?}", timer_message);
                    self.broadcast(timer_message).await;
                }
                Ok(timer_message) = pre_outgoing_receiver.recv_async() => {
                    debug!("HOST(PRE_OUTGOING): Message RECEIVED FROM Broadcast: {:?}", timer_message);
                    let msg = serde_json::to_string(&timer_message).unwrap();
                    if outgoing.send(Message::text(msg)).await.is_err() {
                        debug!("HOST(OUTGOING): Failed to send message to WS");
                        break;
                    }
                    debug!("HOST(OUTGOING): Staged Message SENT TO WS");
                }
                else => break,
            }
        }

        let mut peer_map = self.peer_map.lock().await;
        peer_map.remove(&addr);
    }

    async fn broadcast(&self, timer_message: TimerMessage) {
        let peer_map = self.peer_map.lock().await;

        for (addr, pre_outoging_sender) in peer_map
            .iter()
            .filter(|(&addr, _)| addr != timer_message.sender)
        {
            if pre_outoging_sender
                .send_async(timer_message.clone())
                .await
                .is_err()
            {
                debug!(
                    "HOST(PRE_OUTGOING): Failed to stage message for client({:?})",
                    addr
                );
            }
            debug!(
                "HOST(PRE_OUTGOING): Stage message for CLIENT({:?}): {:?}",
                addr, timer_message
            );
        }
    }

    pub async fn join(self, addr: &SocketAddr) {
        let ws_addr = format!("ws://{}", addr).into_client_request().unwrap();

        let (ws_stream, _) = tokio_tungstenite::connect_async(ws_addr)
            .await
            .expect("Failed to connect");

        let tcp_strem = match ws_stream.get_ref() {
            MaybeTlsStream::Plain(tcp_stream) => tcp_stream,
            _ => panic!("Expected Plain stream"),
        };

        let local_addr = tcp_strem.local_addr().unwrap();
        debug!("{:?} joined the session", local_addr);
        *self.local_addr.lock().await = Some(local_addr);

        let (mut outgoing, mut incoming) = ws_stream.split();

        loop {
            tokio::select! {
                Some(Ok(message)) = incoming.next() => {
                    let timer_message = serde_json::from_str(&message.to_string()).unwrap();
                    debug!("{:?} - JOIN(INCOMING): Message RECEIVED FROM client: {:?}", local_addr, timer_message);
                    self.ws_to_app_sender.send_async(timer_message).await.unwrap();
                    debug!("{:?} - JOIN(WS_TO_APP): Message SENT TO app", local_addr);
                }
                Ok(timer_message) = self.app_to_ws_receiver.recv_async() => {
                    let message = serde_json::to_string(&timer_message).unwrap();
                    debug!("{:?} - JOIN(APP_TO_WS): Message FROM app: {:?}", local_addr, timer_message);
                    if outgoing.send(Message::text(message)).await.is_err() {
                        debug!("{:?} - JOIN(OUTGOING): Failed to send message to WS", local_addr);
                        break;
                    }
                    debug!("{:?} - JOIN(OUTGOING): Message SENT TO WS", local_addr);
                }
                else => break,
            }
        }
    }
}
