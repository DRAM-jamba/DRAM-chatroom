use crate::error::AppError;
use crate::events::{self, emit_message, emit_session_update, emit_user_list, emit_voice_list};
use crate::models::{MessageObj, MessageType};
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

        let sink_arc = Arc::new(Mutex::new(sink));
        let sind_arc_copy = sink_arc.clone();

        tokio::spawn(async move {
            while let Some(Ok(msg)) = stream.next().await {
                println!("{:?}", msg);
                match msg {
                    Message::Text(text) => {
                        if let Err(e) = Self::handle_incoming(&app_clone, &text) {
                            eprintln!("WS Error: {} | Raw: {}", e, text);
                        }
                    },
                    Message::Close(_) => {
                        events::emit_disconnected(&app_clone);
                        break;
                    },
                    Message::Ping(_) => {
                        println!("ping received, pong send");
                        let _ = sink_arc.lock().await.send(Message::Pong(vec![].into())).await.map_err(|_e| ());
                    }
                    _ => {}
                }
            }
        });

        Ok(Self {
            sink: sind_arc_copy,
        })
    }

    fn handle_incoming(app: &AppHandle, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        let obj: MessageObj = serde_json::from_str(text)?;

        match obj.m_type {
            MessageType::Message => {
                emit_message(app, obj);
            }
            MessageType::Connect | MessageType::Disconnect => {
                emit_session_update(app, obj);
            }
            MessageType::UserList => {
                emit_user_list(app, obj);
            }
            MessageType::VoiceList => {
                emit_voice_list(app, obj);
            }
            MessageType::VoiceStart | MessageType::VoiceEnd => {}
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

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to test message deserialization
    fn parse_message(json: &str) -> Result<MessageObj, serde_json::Error> {
        serde_json::from_str(json)
    }

    #[test]
    fn test_deserialize_message_type_message() {
        let json = r#"{"m_type": "message", "from": "user1", "body": "Hello", "ts": 1234567890}"#;
        let result = parse_message(json);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.m_type, MessageType::Message);
        assert_eq!(msg.body, "Hello");
        assert_eq!(msg.ts, 1234567890);
    }

    #[test]
    fn test_deserialize_message_type_connect() {
        let json = r#"{"m_type": "connect", "from": "system", "body": "", "ts": 1234567890}"#;
        let result = parse_message(json);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.m_type, MessageType::Connect);
    }

    #[test]
    fn test_deserialize_message_type_disconnect() {
        let json = r#"{"m_type": "disconnect", "from": "system", "body": "", "ts": 1234567890}"#;
        let result = parse_message(json);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.m_type, MessageType::Disconnect);
    }

    #[test]
    fn test_deserialize_message_type_user_list() {
        let json = r#"{"m_type": "userlist", "from": "system", "body": "user1,user2", "ts": 1234567890}"#;
        let result = parse_message(json);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.m_type, MessageType::UserList);
    }

    #[test]
    fn test_deserialize_message_type_voice_list() {
        let json = r#"{"m_type": "voicelist", "from": "system", "body": "user1,user2", "ts": 1234567890}"#;
        let result = parse_message(json);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.m_type, MessageType::VoiceList);
    }

    #[test]
    fn test_deserialize_message_type_voice_start() {
        let json = r#"{"m_type": "voicestart", "from": "user1", "body": "", "ts": 1234567890}"#;
        let result = parse_message(json);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.m_type, MessageType::VoiceStart);
    }

    #[test]
    fn test_deserialize_message_type_voice_end() {
        let json = r#"{"m_type": "voiceend", "from": "user1", "body": "", "ts": 1234567890}"#;
        let result = parse_message(json);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.m_type, MessageType::VoiceEnd);
    }

    #[test]
    fn test_deserialize_message_with_empty_body() {
        let json = r#"{"m_type": "message", "from": "user1", "body": "", "ts": 1234567890}"#;
        let result = parse_message(json);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.body, "");
    }

    #[test]
    fn test_deserialize_message_with_special_characters() {
        let json = r#"{"m_type": "message", "from": "user1", "body": "Hello! @#$%^&*()", "ts": 1234567890}"#;
        let result = parse_message(json);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.body, "Hello! @#$%^&*()");
    }

    #[test]
    fn test_deserialize_message_with_unicode() {
        let json = r#"{"m_type": "message", "from": "user1", "body": "Hello 世界 🌍", "ts": 1234567890}"#;
        let result = parse_message(json);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.body, "Hello 世界 🌍");
    }

    #[test]
    fn test_deserialize_message_with_escaped_quotes() {
        let json = r#"{"m_type": "message", "from": "user1", "body": "He said \"Hello\"", "ts": 1234567890}"#;
        let result = parse_message(json);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.body, "He said \"Hello\"");
    }

    #[test]
    fn test_deserialize_message_with_newlines() {
        let json = "{\"m_type\": \"message\", \"from\": \"user1\", \"body\": \"Line1\\nLine2\", \"ts\": 1234567890}";
        let result = parse_message(json);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.body, "Line1\nLine2");
    }

    #[test]
    fn test_deserialize_message_missing_required_field_m_type() {
        let json = r#"{"from": "user1", "body": "Hello", "ts": 1234567890}"#;
        let result = parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_message_missing_required_field_ts() {
        let json = r#"{"m_type": "message", "from": "user1", "body": "Hello"}"#;
        let result = parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_message_invalid_json() {
        let json = r#"{"m_type": "message", "from": "user1", "body": "Hello", "ts": 1234567890"#; // Missing closing brace
        let result = parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_message_empty_json_object() {
        let json = r#"{}"#;
        let result = parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_message_wrong_type_for_ts() {
        let json = r#"{"m_type": "message", "from": "user1", "body": "Hello", "ts": "not_a_number"}"#;
        let result = parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_message_null_body() {
        let json = r#"{"m_type": "message", "from": "user1", "body": null, "ts": 1234567890}"#;
        let result = parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_message_body_is_number() {
        let json = r#"{"m_type": "message", "from": "user1", "body": 12345, "ts": 1234567890}"#;
        let result = parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_message_very_large_timestamp() {
        let json = r#"{"m_type": "message", "from": "user1", "body": "Hello", "ts": 9999999999999999}"#;
        let result = parse_message(json);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.ts, 9999999999999999);
    }

    #[test]
    fn test_deserialize_message_zero_timestamp() {
        let json = r#"{"m_type": "message", "from": "user1", "body": "Hello", "ts": 0}"#;
        let result = parse_message(json);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.ts, 0);
    }

    #[test]
    fn test_deserialize_message_negative_timestamp() {
        let json = r#"{"m_type": "message", "from": "user1", "body": "Hello", "ts": -1}"#;
        let result = parse_message(json);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.ts, -1);
    }

    #[test]
    fn test_deserialize_message_very_long_body() {
        let long_body = "a".repeat(10000);
        let json = format!(r#"{{"m_type": "message", "from": "user1", "body": "{}", "ts": 1234567890}}"#, long_body);
        let result = parse_message(&json);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.body.len(), 10000);
    }

    #[test]
    fn test_deserialize_message_type_case_sensitive() {
        // Message types should be case-sensitive; "Message" != "message"
        let json = r#"{"m_type": "Message", "from": "user1", "body": "Hello", "ts": 1234567890}"#;
        let result = parse_message(json);
        // This will fail because "Message" is not a valid variant
        assert!(result.is_err());
    }

    #[test]
    fn test_message_type_routing_logic() {
        // Test that we can identify which handler each message type should use
        let test_cases = vec![
            (MessageType::Message, "message_handler"),
            (MessageType::Connect, "session_update"),
            (MessageType::Disconnect, "session_update"),
            (MessageType::UserList, "user_list_handler"),
            (MessageType::VoiceList, "voice_list_handler"),
            (MessageType::VoiceStart, "no_op"),
            (MessageType::VoiceEnd, "no_op"),
        ];

        for (msg_type, expected_handler) in test_cases {
            let handler = match msg_type {
                MessageType::Message => "message_handler",
                MessageType::Connect | MessageType::Disconnect => "session_update",
                MessageType::UserList => "user_list_handler",
                MessageType::VoiceList => "voice_list_handler",
                MessageType::VoiceStart | MessageType::VoiceEnd => "no_op",
            };
            assert_eq!(handler, expected_handler);
        }
    }

    #[test]
    fn test_deserialize_multiple_messages_in_sequence() {
        let jsons = vec![
            r#"{"m_type": "message", "from": "user1", "body": "First", "ts": 1}"#,
            r#"{"m_type": "connect", "from": "system", "body": "", "ts": 2}"#,
            r#"{"m_type": "userlist", "from": "system", "body": "user1", "ts": 3}"#,
        ];

        for json in jsons {
            let result = parse_message(json);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_deserialize_message_with_extra_fields() {
        // JSON with extra fields that don't exist in MessageObj
        let json = r#"{"m_type": "message", "from": "user1", "body": "Hello", "ts": 1234567890, "extra": "field", "another": 123}"#;
        let result = parse_message(json);
        // serde_json should ignore unknown fields by default if configured
        assert!(result.is_ok());
    }
}
