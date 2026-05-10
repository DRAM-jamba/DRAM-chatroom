use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use crate::error::AppError;
use crate::events::{self, emit_message, emit_member_list};
use crate::models::{MessageObj, MessageType, MessagePayload};

type WsSink = futures_util::stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

#[derive(Debug, Clone)]
pub struct WsClient {
    pub sink: Arc<Mutex<WsSink>>,
}

impl WsClient {
    pub async fn connect(url: &str, app: AppHandle) -> Result<Self, AppError> {
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;

        let (sink, mut stream) = ws_stream.split();

        // Spawn a task to handle incoming messages and forward them to the frontend
        let app_clone = app.clone();
        tokio::spawn(async move {
            while let Some(Ok(msg)) = stream.next().await {
                match msg {
                Message::Text(text) => {
                    match serde_json::from_str::<MessageObj>(&text) {
                        Ok(obj) => {
                            match obj.m_type {
                                MessageType::Message => {
                                    println!("Emitting message: {:?}", obj);
                                    emit_message(&app_clone, MessagePayload {
                                        from: obj.from,
                                        body: obj.body,
                                        ts: obj.ts,
                                    });
                                }
                                MessageType::Connect | MessageType::Disconnect | MessageType::UserList => {
                                    if obj.body.is_empty() {
                                        println!("Emitting user list: {:?}", obj);
                                        return; 
                                    }
                                    println!("Emitting user list: {:?}", obj);
                                    match serde_json::from_str::<Vec<String>>(&obj.body) {
                                        Ok(participants) => {
                                            emit_member_list(&app_clone, participants);
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to parse member list: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Parse error: {} | Raw: {}", e, text);
                        }
                    }
                }
                Message::Close(_) => {
                    events::emit_disconnected(&app_clone);
                    break;
                }
                _ => {}
            }
                }    
        });

        Ok(Self {
            sink: Arc::new(Mutex::new(sink)),
        })
    }

    pub async fn send(&self, msg: &str) -> Result<(), AppError> {
        println!("Sending message: {}", msg);
        self.sink
            .lock()
            .await
            .send(Message::Text(msg.to_string().into()))
            .await
            .map_err(|e| AppError::Network(e.to_string()))
    }

    pub async fn close(&self) -> Result<(), AppError> {
        self.sink
            .lock()
            .await
            .send(Message::Close(None))
            .await
            .map_err(|e| AppError::Network(e.to_string()))
    }
}
