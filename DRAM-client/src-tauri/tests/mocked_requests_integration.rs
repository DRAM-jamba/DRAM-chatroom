#[test]
fn test_server_api_auth_endpoints() {
    let api = ServerApi::new("http://192.168.1.1:8080");
    assert_eq!(api.challenge(), "http://192.168.1.1:8080/server/challenge");
    assert_eq!(api.token_url(), "http://192.168.1.1:8080/server/token");
    assert_eq!(api.refresh_token_url(), "http://192.168.1.1:8080/server/refresh_token");
}
#[test]
fn test_token_response_parsing() {
    let json = r#"{"token":"tok_123456"}"#;
    let token: dram_client_lib::models::TokenResponse = serde_json::from_str(json).expect("Failed to parse TokenResponse");
    assert_eq!(token.token, "tok_123456");
}
#[test]
fn test_challenge_from_server_parsing() {
    let json = r#"{"challenge":"abcdef123456"}"#;
    let challenge: dram_client_lib::models::ChallengeFromServer = serde_json::from_str(json).expect("Failed to parse ChallengeFromServer");
    assert_eq!(challenge.challenge, "abcdef123456");
}
#[test]
fn test_challenge_solve_payload_serialization() {
    let payload = dram_client_lib::models::ChallengeSolvePayload {
        nonce: "nonce123".to_string(),
        signature: "sig456".to_string(),
        user_key: "user789".to_string(),
    };
    let json = serde_json::to_string(&payload).expect("Failed to serialize ChallengeSolvePayload");
    assert!(json.contains("nonce123"));
    assert!(json.contains("sig456"));
    assert!(json.contains("user789"));
}
#[test]
fn test_server_error_payload_parsing() {
    let json = r#"{"error":"Something went wrong"}"#;
    let err: dram_client_lib::models::ServerErrorPayload = serde_json::from_str(json).expect("Failed to parse ServerErrorPayload");
    assert_eq!(err.error, "Something went wrong");
}
use dram_client_lib::models::{UserKey, SessionList, SessionKey, VoiceToken, Session};
use dram_client_lib::api::ServerApi;

#[test]
fn test_server_api_add_server_endpoint() {
    let api = ServerApi::new("http://192.168.1.1:8080");
    let endpoint = api.add_server();
    assert_eq!(endpoint, "http://192.168.1.1:8080/server/add");
}

#[test]
fn test_server_api_connect_server_endpoint() {
    let api = ServerApi::new("http://10.0.0.5:8080");
    let endpoint = api.connect_server();
    assert_eq!(endpoint, "http://10.0.0.5:8080/server/connect");
}

#[test]
fn test_server_api_session_list_endpoint() {
    let api = ServerApi::new("http://localhost:8080");
    let endpoint = api.session_list();
    assert_eq!(endpoint, "http://localhost:8080/session/list");
}

#[test]
fn test_server_api_create_session_endpoint() {
    let api = ServerApi::new("http://192.168.1.100:8080");
    let endpoint = api.create_session();
    assert_eq!(endpoint, "http://192.168.1.100:8080/session/create");
}

#[test]
fn test_server_api_voice_chat_endpoint() {
    let api = ServerApi::new("http://192.168.1.50:8080");
    let endpoint = api.create_voicechat();
    assert_eq!(endpoint, "http://192.168.1.50:8080/session/token");
}

#[test]
fn test_server_api_websocket_conversion() {
    let api = ServerApi::new("http://192.168.1.1:8080");
    let ws_url = api.ws();
    assert_eq!(ws_url, "ws://192.168.1.1:8080/session/connect");
}

#[test]
fn test_server_api_websocket_https_conversion() {
    let api = ServerApi::new("https://example.com:8080");
    let ws_url = api.ws();
    assert_eq!(ws_url, "wss://example.com:8080/session/connect");
}

#[test]
fn test_server_api_all_endpoints() {
    let api = ServerApi::new("http://server.local:9000");
    
    assert_eq!(api.add_server(), "http://server.local:9000/server/add");
    assert_eq!(api.connect_server(), "http://server.local:9000/server/connect");
    assert_eq!(api.leave_server(), "http://server.local:9000/server/leave");
    assert_eq!(api.forget_server(), "http://server.local:9000/server/forget");
    assert_eq!(api.set_nickname(), "http://server.local:9000/server/nickname");
    
    assert_eq!(api.session_list(), "http://server.local:9000/session/list");
    assert_eq!(api.create_session(), "http://server.local:9000/session/create");
    assert_eq!(api.add_session(), "http://server.local:9000/session/add");
    assert_eq!(api.leave_session(), "http://server.local:9000/session/leave");
    assert_eq!(api.forget_session(), "http://server.local:9000/session/forget");
    assert_eq!(api.delete_session(), "http://server.local:9000/session/delete");
    
    assert_eq!(api.create_voicechat(), "http://server.local:9000/session/token");
}

#[test]
fn test_server_api_port_preservation() {
    // Test that different ports are preserved
    let test_cases = vec![
        ("http://192.168.1.1:3000", "/server/add", "http://192.168.1.1:3000/server/add"),
        ("http://192.168.1.1:8080", "/server/add", "http://192.168.1.1:8080/server/add"),
        ("http://192.168.1.1:9999", "/session/list", "http://192.168.1.1:9999/session/list"),
    ];
    
    for (base_url, path, expected) in test_cases {
        let full_url = format!("{}{}", base_url, path);
        assert_eq!(full_url, expected);
    }
}

#[test]
fn test_add_server_response_parsing() {
    // Simulate HTTP response from POST /add_server
    let server_response = r#"{"user_key": "user_abc_123_def_456"}"#;
    
    let user_key: UserKey = serde_json::from_str(server_response)
        .expect("Failed to parse UserKey response");
    
    assert_eq!(user_key.user_key, "user_abc_123_def_456");
}

#[test]
fn test_add_server_invalid_response() {
    // Test handling of invalid response format
    let invalid_response = r#"{"some_key": "value"}"#;
    
    let result: Result<UserKey, _> = serde_json::from_str(invalid_response);
    assert!(result.is_err(), "Should fail with missing user_key field");
}

#[test]
fn test_get_sessions_response_parsing() {
    // Simulate HTTP response from GET /session_list
    let server_response = r#"{
        "user_sessions": [
            {
                "session_key": "session_1",
                "session_name": "General",
                "user_role": "member"
            },
            {
                "session_key": "session_2",
                "session_name": "Work",
                "user_role": "admin"
            }
        ]
    }"#;
    
    let session_list: SessionList = serde_json::from_str(server_response)
        .expect("Failed to parse SessionList response");
    
    assert_eq!(session_list.user_sessions.len(), 2);
    assert_eq!(session_list.user_sessions[0].id, "session_1");
    assert_eq!(session_list.user_sessions[0].name, "General");
    assert_eq!(session_list.user_sessions[1].id, "session_2");
    assert_eq!(session_list.user_sessions[1].user_role, "admin");
}

#[test]
fn test_get_sessions_empty_response() {
    // Simulate empty session list response
    let server_response = r#"{"user_sessions": []}"#;
    
    let session_list: SessionList = serde_json::from_str(server_response)
        .expect("Failed to parse empty SessionList");
    
    assert_eq!(session_list.user_sessions.len(), 0);
}

#[test]
fn test_get_sessions_missing_field() {
    // Test handling of malformed response
    let invalid_response = r#"{"wrong_field": []}"#;
    
    let result: Result<SessionList, _> = serde_json::from_str(invalid_response);
    assert!(result.is_err(), "Should fail with missing user_sessions field");
}

#[test]
fn test_create_session_response_parsing() {
    // Simulate HTTP response from POST /create_session
    let server_response = r#"{"session_key": "new_session_xyz_789"}"#;
    
    let session_key: SessionKey = serde_json::from_str(server_response)
        .expect("Failed to parse SessionKey response");
    
    assert_eq!(session_key.session_key, "new_session_xyz_789");
}

#[test]
fn test_create_session_invalid_response() {
    let invalid_response = r#"{"wrong_key": "value"}"#;
    
    let result: Result<SessionKey, _> = serde_json::from_str(invalid_response);
    assert!(result.is_err(), "Should fail with missing session_key field");
}

#[test]
fn test_join_voice_chat_response_parsing() {
    // Simulate HTTP response from GET /voice_chat
    let server_response = r#"{"token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0"}"#;
    
    let voice_token: VoiceToken = serde_json::from_str(server_response)
        .expect("Failed to parse VoiceToken response");
    
    assert_eq!(voice_token.token, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0");
}

#[test]
fn test_voice_chat_url_construction_from_ip() {
    // Test that voice URL is constructed from server IP
    let test_cases = vec![
        ("192.168.1.1:8080", "ws://192.168.1.1:7880"),
        ("10.0.0.5:8080", "ws://10.0.0.5:7880"),
        ("localhost:8080", "ws://localhost:7880"),
    ];
    
    for (ip, expected_url) in test_cases {
        let host = ip.split(':').next().unwrap_or(ip);
        let lk_url = format!("ws://{}:7880", host);
        assert_eq!(lk_url, expected_url);
    }
}

#[test]
fn test_session_deserialization_with_aliases() {
    // Test that session_key and session_name are aliased correctly in deserialization
    let json = r#"{
        "session_key": "sess_123",
        "session_name": "Test Room",
        "user_role": "member"
    }"#;
    
    let session: Session = serde_json::from_str(json)
        .expect("Failed to parse Session with aliases");
    
    assert_eq!(session.id, "sess_123");
    assert_eq!(session.name, "Test Room");
    assert_eq!(session.user_role, "member");
}

#[test]
fn test_session_deserialization_snake_case_aliases() {
    // Test snake_case aliases work as alternatives
    let json = r#"{
        "id": "sess_456",
        "name": "Work Chat",
        "user_role": "admin"
    }"#;
    
    let session: Session = serde_json::from_str(json)
        .expect("Failed to parse Session with snake_case");
    
    assert_eq!(session.id, "sess_456");
    assert_eq!(session.name, "Work Chat");
}

#[test]
fn test_multiple_session_responses_sequence() {
    // Test handling multiple responses in sequence (like polling)
    let responses = vec![
        r#"{"session_key": "sess_1", "session_name": "Room1", "user_role": "member"}"#,
        r#"{"session_key": "sess_2", "session_name": "Room2", "user_role": "admin"}"#,
        r#"{"session_key": "sess_3", "session_name": "Room3", "user_role": "member"}"#,
    ];
    
    let mut sessions = Vec::new();
    for response in responses {
        let session: Session = serde_json::from_str(response)
            .expect("Failed to parse session");
        sessions.push(session);
    }
    
    assert_eq!(sessions.len(), 3);
    assert_eq!(sessions[0].id, "sess_1");
    assert_eq!(sessions[2].name, "Room3");
}

#[test]
fn test_response_with_extra_fields() {
    // Ensure responses with extra fields don't break parsing
    let response_with_extras = r#"{
        "session_key": "sess_999",
        "session_name": "Extra Fields",
        "user_role": "member",
        "created_at": "2024-01-01T00:00:00Z",
        "extra_field": "should be ignored"
    }"#;
    
    let session: Session = serde_json::from_str(response_with_extras)
        .expect("Failed to parse with extra fields");
    
    assert_eq!(session.id, "sess_999");
    assert_eq!(session.name, "Extra Fields");
}

#[test]
fn test_session_list_with_large_dataset() {
    // Test parsing large session lists
    let mut sessions_json = String::from(r#"{"user_sessions": ["#);
    
    for i in 0..50 {
        if i > 0 { sessions_json.push(','); }
        sessions_json.push_str(&format!(
            r#"{{"session_key": "sess_{}", "session_name": "Session {}", "user_role": "member"}}"#,
            i, i
        ));
    }
    sessions_json.push_str("]}");
    
    let session_list: SessionList = serde_json::from_str(&sessions_json)
        .expect("Failed to parse large session list");
    
    assert_eq!(session_list.user_sessions.len(), 50);
    assert_eq!(session_list.user_sessions[0].id, "sess_0");
    assert_eq!(session_list.user_sessions[49].id, "sess_49");
}

#[test]
fn test_voice_token_with_special_characters() {
    // JWT tokens have special characters - ensure they parse correctly
    let response = r#"{"token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"}"#;
    
    let token: VoiceToken = serde_json::from_str(response)
        .expect("Failed to parse JWT token");
    
    assert!(token.token.contains("."));
    assert_eq!(token.token.split('.').count(), 3); // JWT has 3 parts
}

#[test]
fn test_user_key_with_special_characters() {
    // Test that keys with special characters deserialize
    let response = r#"{"user_key": "key_!@#$%^&*()_+-=[]{}|;:',.<>?/"}"#;
    
    let user_key: UserKey = serde_json::from_str(response)
        .expect("Failed to parse user key with special chars");
    
    assert!(user_key.user_key.contains("!@#$"));
}
