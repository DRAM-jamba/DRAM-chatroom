use dram_client_lib::models::PersistedServer;

#[test]
fn test_server_persistence_structure() {
    // Test that server data can be serialized and deserialized for storage
    let servers = vec![
        PersistedServer {
            id: "id1".to_string(),
            ip: "192.168.1.1:8080".to_string(),
            server_name: "Office".to_string(),
            user_key: "key_abc123".to_string(),
            user_nickname: Some("My Office".to_string()),
        },
        PersistedServer {
            id: "id2".to_string(),
            ip: "192.168.1.2:8080".to_string(),
            server_name: "Home".to_string(),
            user_key: "key_def456".to_string(),
            user_nickname: None,
        },
    ];

    // Simulate storage: serialize to JSON
    let json = serde_json::to_string(&servers).expect("Serialization failed");
    
    // Simulate retrieval: deserialize from JSON
    let restored: Vec<PersistedServer> = serde_json::from_str(&json).expect("Deserialization failed");

    assert_eq!(restored.len(), 2);
    assert_eq!(restored[0].id, "id1");
    assert_eq!(restored[0].user_nickname, Some("My Office".to_string()));
    assert_eq!(restored[1].user_nickname, None);
}

#[test]
fn test_add_server_to_list() {
    let mut servers: Vec<PersistedServer> = Vec::new();

    // Add first server
    let server1 = PersistedServer {
        id: "uuid1".to_string(),
        ip: "10.0.0.1:8080".to_string(),
        server_name: "Server1".to_string(),
        user_key: "key1".to_string(),
        user_nickname: None,
    };
    servers.push(server1.clone());

    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].ip, "10.0.0.1:8080");

    // Add second server
    let server2 = PersistedServer {
        id: "uuid2".to_string(),
        ip: "10.0.0.2:8080".to_string(),
        server_name: "Server2".to_string(),
        user_key: "key2".to_string(),
        user_nickname: None,
    };
    servers.push(server2);

    assert_eq!(servers.len(), 2);
}

#[test]
fn test_duplicate_server_detection() {
    let servers = vec![
        PersistedServer {
            id: "id1".to_string(),
            ip: "192.168.1.1:8080".to_string(),
            server_name: "Server1".to_string(),
            user_key: "key1".to_string(),
            user_nickname: None,
        },
        PersistedServer {
            id: "id2".to_string(),
            ip: "192.168.1.2:8080".to_string(),
            server_name: "Server2".to_string(),
            user_key: "key2".to_string(),
            user_nickname: None,
        },
    ];

    // Check if IP exists
    let ip_exists = servers.iter().any(|s| s.ip == "192.168.1.1:8080");
    assert!(ip_exists);

    // Check if new IP would be duplicate
    let new_ip = "192.168.1.3:8080";
    let is_duplicate = servers.iter().any(|s| s.ip == new_ip);
    assert!(!is_duplicate);
}

#[test]
fn test_remove_server_from_list() {
    let mut servers = vec![
        PersistedServer {
            id: "id1".to_string(),
            ip: "10.0.0.1:8080".to_string(),
            server_name: "Server1".to_string(),
            user_key: "key1".to_string(),
            user_nickname: None,
        },
        PersistedServer {
            id: "id2".to_string(),
            ip: "10.0.0.2:8080".to_string(),
            server_name: "Server2".to_string(),
            user_key: "key2".to_string(),
            user_nickname: None,
        },
        PersistedServer {
            id: "id3".to_string(),
            ip: "10.0.0.3:8080".to_string(),
            server_name: "Server3".to_string(),
            user_key: "key3".to_string(),
            user_nickname: None,
        },
    ];

    servers.retain(|s| s.ip != "10.0.0.2:8080");

    assert_eq!(servers.len(), 2);
    assert!(servers.iter().all(|s| s.ip != "10.0.0.2:8080"));
    assert_eq!(servers[0].ip, "10.0.0.1:8080");
    assert_eq!(servers[1].ip, "10.0.0.3:8080");
}

#[test]
fn test_update_server_nickname() {
    let mut servers = vec![PersistedServer {
        id: "id1".to_string(),
        ip: "192.168.1.1:8080".to_string(),
        server_name: "MyServer".to_string(),
        user_key: "key1".to_string(),
        user_nickname: None,
    }];

    // Update nickname
    if let Some(server) = servers.iter_mut().find(|s| s.ip == "192.168.1.1:8080") {
        server.user_nickname = Some("Office Server".to_string());
    }

    assert_eq!(
        servers[0].user_nickname,
        Some("Office Server".to_string())
    );
}

#[test]
fn test_server_list_roundtrip_with_encryption_pattern() {
    // Test the pattern: serialize -> encode -> decode -> deserialize
    let servers = vec![
        PersistedServer {
            id: "id1".to_string(),
            ip: "192.168.1.100:8080".to_string(),
            server_name: "Production".to_string(),
            user_key: "prod_key_xyz".to_string(),
            user_nickname: Some("Prod Server".to_string()),
        },
    ];

    // Serialize
    let plaintext = serde_json::to_vec(&servers).expect("Serialization failed");
    
    // Simulate encryption: encode as hex (would be ciphertext in real scenario)
    let hex_encoded = hex::encode(&plaintext);
    
    // Simulate decryption: decode from hex
    let decoded = hex::decode(&hex_encoded).expect("Hex decode failed");
    
    // Deserialize
    let restored: Vec<PersistedServer> = serde_json::from_slice(&decoded)
        .expect("Deserialization failed");

    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].ip, "192.168.1.100:8080");
    assert_eq!(restored[0].user_nickname, Some("Prod Server".to_string()));
}

#[test]
fn test_multiple_servers_operations() {
    let mut servers: Vec<PersistedServer> = Vec::new();

    // Add 5 servers
    for i in 1..=5 {
        servers.push(PersistedServer {
            id: format!("id{}", i),
            ip: format!("10.0.0.{}:8080", i),
            server_name: format!("Server{}", i),
            user_key: format!("key{}", i),
            user_nickname: if i % 2 == 0 { Some(format!("Nick{}", i)) } else { None },
        });
    }

    assert_eq!(servers.len(), 5);

    // Remove one
    servers.retain(|s| s.ip != "10.0.0.3:8080");
    assert_eq!(servers.len(), 4);

    // Update nickname on remaining
    if let Some(server) = servers.iter_mut().find(|s| s.ip == "10.0.0.1:8080") {
        server.user_nickname = Some("Updated".to_string());
    }

    // Verify state
    assert_eq!(
        servers.iter().find(|s| s.ip == "10.0.0.1:8080").unwrap().user_nickname,
        Some("Updated".to_string())
    );
}

#[test]
fn test_empty_server_list_persistence() {
    let servers: Vec<PersistedServer> = Vec::new();
    
    let json = serde_json::to_string(&servers).expect("Serialization failed");
    let restored: Vec<PersistedServer> = serde_json::from_str(&json)
        .expect("Deserialization failed");

    assert_eq!(restored.len(), 0);
}

#[test]
fn test_server_with_special_characters_in_key() {
    let server = PersistedServer {
        id: "id1".to_string(),
        ip: "192.168.1.1:8080".to_string(),
        server_name: "Server".to_string(),
        user_key: "key_with!@#$%^&*()special+chars=".to_string(),
        user_nickname: None,
    };

    let json = serde_json::to_string(&server).expect("Serialization failed");
    let restored: PersistedServer = serde_json::from_str(&json)
        .expect("Deserialization failed");

    assert_eq!(restored.user_key, "key_with!@#$%^&*()special+chars=");
}
