use crate::error::AppError;
use crate::events::{self, emit_member_list, emit_message};
use crate::models::{MessageObj, MessagePayload, MessageType};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue, Message},
    MaybeTlsStream,
    WebSocketStream,
};

type WsSink = futures_util::stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

#[derive(Debug, Clone)]
pub struct WsClient {
    pub sink: Arc<Mutex<WsSink>>,
}

impl WsClient {
    pub async fn connect(url: &str, user_key: &str, session_key: &str, app: AppHandle) -> Result<Self, AppError> {
        let mut request = url.into_client_request()
            .map_err(|e| AppError::Network(e.to_string()))?;
        
        let headers = request.headers_mut();
        
        headers.insert("user_key", HeaderValue::from_str(user_key)
            .map_err(|e| AppError::Network(e.to_string()))?);
        headers.insert("session_key", HeaderValue::from_str(session_key)
            .map_err(|e| AppError::Network(e.to_string()))?);

        let (ws_stream, _) = connect_async(request)
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;

        let (sink, mut stream) = ws_stream.split();
        let app_clone = app.clone();

        tokio::spawn(async move {
            while let Some(Ok(msg)) = stream.next().await {
                if let Message::Text(text) = msg {
                    if let Err(e) = Self::handle_incoming(&app_clone, &text) {
                        eprintln!("WS Error: {} | Raw: {}", e, text);
                    }
                } else if let Message::Close(_) = msg {
                    events::emit_disconnected(&app_clone);
                    break;
                }
            }
        });

        Ok(Self {
            sink: Arc::new(Mutex::new(sink)),
        })
    }

    fn handle_incoming(app: &AppHandle, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        let obj: MessageObj = serde_json::from_str(text)?;

        match obj.m_type {
            MessageType::Message => {
                emit_message(app, MessageObj {
                    m_type: obj.m_type,
                    from: obj.from,
                    body: obj.body,
                    ts: obj.ts,
                });
            }
            MessageType::Connect | MessageType::Disconnect => {
                emit_session_update(app, MessageObj{
                    m_type: obj.m_type,
                    from: obj.from,
                    body: obj.body,
                    ts: obj.ts,
                });
            }
        }
        Ok(())
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
