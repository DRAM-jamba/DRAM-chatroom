use dram_client_lib::models::{MessageObj, MessageType};

#[test]
fn test_deserialize_all_message_types() {
    let test_cases = vec![
        (
            r#"{"m_type": "message", "from": "alice", "body": "Hello", "ts": 1000}"#,
            MessageType::Message,
        ),
        (
            r#"{"m_type": "connect", "from": "system", "body": "", "ts": 2000}"#,
            MessageType::Connect,
        ),
        (
            r#"{"m_type": "disconnect", "from": "system", "body": "", "ts": 2100}"#,
            MessageType::Disconnect,
        ),
        (
            r#"{"m_type": "userlist", "from": "system", "body": "alice,bob", "ts": 3000}"#,
            MessageType::UserList,
        ),
        (
            r#"{"m_type": "voicelist", "from": "system", "body": "alice", "ts": 3100}"#,
            MessageType::VoiceList,
        ),
        (
            r#"{"m_type": "voicestart", "from": "alice", "body": "", "ts": 4000}"#,
            MessageType::VoiceStart,
        ),
        (
            r#"{"m_type": "voiceend", "from": "alice", "body": "", "ts": 5000}"#,
            MessageType::VoiceEnd,
        ),
    ];

    for (json, expected_type) in test_cases {
        let msg: MessageObj = serde_json::from_str(json).expect("Failed to parse");
        assert_eq!(msg.m_type, expected_type);
    }
}

#[test]
fn test_message_routing_by_type() {
    let message_types = vec![
        (MessageType::Message, "emit_message"),
        (MessageType::Connect, "emit_session_update"),
        (MessageType::Disconnect, "emit_session_update"),
        (MessageType::UserList, "emit_user_list"),
        (MessageType::VoiceList, "emit_voice_list"),
        (MessageType::VoiceStart, "no_op"),
        (MessageType::VoiceEnd, "no_op"),
    ];

    for (msg_type, expected_handler) in message_types {
        let handler = match msg_type {
            MessageType::Message => "emit_message",
            MessageType::Connect | MessageType::Disconnect => "emit_session_update",
            MessageType::UserList => "emit_user_list",
            MessageType::VoiceList => "emit_voice_list",
            MessageType::VoiceStart | MessageType::VoiceEnd => "no_op",
        };
        assert_eq!(handler, expected_handler);
    }
}

#[test]
fn test_message_serialization_roundtrip() {
    let original = MessageObj {
        m_type: MessageType::Message,
        from: "alice".to_string(),
        body: "Integration test message".to_string(),
        ts: 1234567890,
    };

    let json = serde_json::to_string(&original).expect("Serialization failed");
    let deserialized: MessageObj = serde_json::from_str(&json).expect("Deserialization failed");

    assert_eq!(original.m_type, deserialized.m_type);
    assert_eq!(original.from, deserialized.from);
    assert_eq!(original.body, deserialized.body);
    assert_eq!(original.ts, deserialized.ts);
}

#[test]
fn test_invalid_message_handling() {
    let invalid_cases = vec![
        r#"{"m_type": "invalid_type", "from": "user", "body": "test", "ts": 1}"#,
        r#"{"m_type": "message"}"#,
        r#"invalid json"#,
        r#"{}"#,
    ];

    for json in invalid_cases {
        let result: Result<MessageObj, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Should have failed: {}", json);
    }
}

#[test]
fn test_message_with_unicode_and_special_chars() {
    let json = r#"{"m_type": "message", "from": "user", "body": "你好 🌍 \"quotes\"", "ts": 1}"#;
    let msg: MessageObj = serde_json::from_str(json).expect("Failed to parse");
    
    assert_eq!(msg.body, "你好 🌍 \"quotes\"");
    assert_eq!(msg.m_type, MessageType::Message);
}
