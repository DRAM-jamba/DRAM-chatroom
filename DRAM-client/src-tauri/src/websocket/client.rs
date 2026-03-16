use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tokio::sync::Mutex;
use std::sync::Arc;
use tauri::AppHandle;
//use crate::{error::AppError, events};
use crate::error::AppError;
use crate::events;            

type WsSink = futures_util::stream::SplitSink<
    WebSocketStream<MaybeTlsStream<TcpStream>>,
    Message,
>;

#[derive(Debug)]
pub struct WsClient {
    pub sink: Arc<Mutex<WsSink>>,
}

impl WsClient {
    pub async fn connect(ip: String, app: AppHandle) -> Result<Self, AppError> {
        let url = format!("ws://{}", ip);
        let (ws_stream, _) = connect_async(url).await
        .map_err(|e| AppError::Network(e.to_string()))?;
    
        let (sink, mut stream) = ws_stream.split();

        // Spawn a task to handle incoming messages and forward them to the frontend
        let app_clone = app.clone();
        tokio::spawn(async move {
            while let Some(Ok(msg)) = stream.next().await {
                match msg {
                    Message::Text(text) => {
                        events::emit_message(&app_clone, &text);
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
        self.sink.lock().await
            .send(Message::Text(msg.to_string().into())).await
            .map_err(|e| AppError::Network(e.to_string()))
    }

    pub async fn ping(&self) -> Result<(), AppError> {
        self.sink.lock().await
            .send(Message::Ping(vec![].into())).await
            .map_err(|e| AppError::Network(e.to_string()))
    }
}