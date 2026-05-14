// updated server integration tests for KAN-123
// routes now use json request bodies instead of url params
// http methods also changed - post/put/delete/patch instead of get

use reqwest::Client;
use serde_json::{Value, json};

const BASE: &str = "http://localhost:3000";

// helper - adds a new user and returns their user_key
async fn add_user(client: &Client) -> String {
    let res = client.post(format!("{}/server/add", BASE))
        .send().await.unwrap().json::<Value>().await.unwrap();
    res["user_key"].as_str().unwrap().to_string()
}

// helper - creates a session and returns session_key
async fn create_session(client: &Client, user_key: &str, name: &str) -> String {
    let res = client.post(format!("{}/session/create", BASE))
        .json(&json!({"user_key": user_key, "session_name": name}))
        .send().await.unwrap().json::<Value>().await.unwrap();
    res["session_key"].as_str().unwrap().to_string()
}

// helper - cleanup user after test
async fn forget_user(client: &Client, user_key: &str) {
    let _ = client.delete(format!("{}/server/forget", BASE))
        .json(&json!({"user_key": user_key}))
        .send().await;
}

// helper - cleanup session after test
async fn forget_session(client: &Client, user_key: &str, session_key: &str) {
    let _ = client.delete(format!("{}/session/forget", BASE))
        .json(&json!({"user_key": user_key, "session_key": session_key}))
        .send().await;
}

// server/add - should return a user_key
#[tokio::test]
async fn test_add_user() {
    let client = Client::new();
    let res = client.post(format!("{}/server/add", BASE))
        .send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);

    let body: Value = res.json().await.unwrap();
    let key = body["user_key"].as_str().unwrap().to_string();
    assert!(!key.is_empty());
    println!("user created: {}", key);

    forget_user(&client, &key).await;
}

// server/connect - valid key should give back auth_token
#[tokio::test]
async fn test_connect_valid_key() {
    let client = Client::new();
    let key = add_user(&client).await;

    let res = client.put(format!("{}/server/connect", BASE))
        .json(&json!({"user_key": key}))
        .send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);

    let body: Value = res.json().await.unwrap();
    assert!(body["auth_token"].is_string());

    forget_user(&client, &key).await;
}

// server/connect - fake key should get 404
#[tokio::test]
async fn test_connect_bad_key() {
    let client = Client::new();
    let res = client.put(format!("{}/server/connect", BASE))
        .json(&json!({"user_key": "thiskeyisnotreal"}))
        .send().await.unwrap();
    assert_eq!(res.status().as_u16(), 404);
}

// server/leave - stub route, should return 200 with placeholder message
#[tokio::test]
async fn test_leave_server() {
    let res = Client::new().delete(format!("{}/server/leave", BASE))
        .send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["message"], "not done yet");
}

// server/forget - should delete the user successfully
#[tokio::test]
async fn test_forget_server() {
    let client = Client::new();
    let key = add_user(&client).await;

    let res = client.delete(format!("{}/server/forget", BASE))
        .json(&json!({"user_key": key}))
        .send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
}

// server/nickname - should update nickname successfully
#[tokio::test]
async fn test_set_nickname() {
    let client = Client::new();
    let key = add_user(&client).await;

    let res = client.patch(format!("{}/server/nickname", BASE))
        .json(&json!({"user_key": key, "nickname": "testname123"}))
        .send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);

    forget_user(&client, &key).await;
}

// session/list - should return user_sessions array
#[tokio::test]
async fn test_get_session_list() {
    let client = Client::new();
    let key = add_user(&client).await;

    let res = client.get(format!("{}/session/list", BASE))
        .json(&json!({"user_key": key}))
        .send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);

    let body: Value = res.json().await.unwrap();
    assert!(body["user_sessions"].is_array());

    forget_user(&client, &key).await;
}

// session/create - should create a session and return session_key
#[tokio::test]
async fn test_create_session() {
    let client = Client::new();
    let key = add_user(&client).await;

    let res = client.post(format!("{}/session/create", BASE))
        .json(&json!({"user_key": key, "session_name": "mysession"}))
        .send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);

    let body: Value = res.json().await.unwrap();
    let skey = body["session_key"].as_str().unwrap().to_string();
    assert!(!skey.is_empty());
    println!("session created: {}", skey);

    forget_user(&client, &key).await;
}

// session/add - adding a valid session key to a user's list
#[tokio::test]
async fn test_add_session() {
    let client = Client::new();

    // need two users - one to own the session, one to join it
    let owner_key = add_user(&client).await;
    let member_key = add_user(&client).await;

    let skey = create_session(&client, &owner_key, "addsessiontest").await;

    let res = client.post(format!("{}/session/add", BASE))
        .json(&json!({"user_key": member_key, "session_key": skey}))
        .send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);

    forget_session(&client, &member_key, &skey).await;
    forget_user(&client, &member_key).await;
    forget_user(&client, &owner_key).await;
}

// session/add - fake session key should be rejected
#[tokio::test]
async fn test_add_session_bad_key() {
    let client = Client::new();
    let key = add_user(&client).await;

    let res = client.post(format!("{}/session/add", BASE))
        .json(&json!({"user_key": key, "session_key": "fakekeyxyz"}))
        .send().await.unwrap();
    assert!(res.status().as_u16() >= 400, "expected error for fake session key");

    forget_user(&client, &key).await;
}

// session/connect - plain get without ws headers should get 400 not 500
#[tokio::test]
async fn test_connect_to_session_no_ws() {
    let client = Client::new();
    let key = add_user(&client).await;
    let skey = create_session(&client, &key, "wstest").await;

    // connect now uses headers for user_key and session_key
    let res = client.get(format!("{}/session/connect", BASE))
        .header("user_key", &key)
        .header("session_key", &skey)
        .send().await.unwrap();
    println!("connect without ws upgrade: {}", res.status());
    assert!(res.status().as_u16() < 500, "server crashed on ws connect");

    forget_user(&client, &key).await;
}

// session/leave - stub, should return 200
#[tokio::test]
async fn test_leave_session() {
    let res = Client::new().delete(format!("{}/session/leave", BASE))
        .send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
}

// session/forget - member removing session from their list
#[tokio::test]
async fn test_forget_session() {
    let client = Client::new();
    let owner_key = add_user(&client).await;
    let member_key = add_user(&client).await;

    let skey = create_session(&client, &owner_key, "forgetsessiontest").await;

    // member joins first
    let _ = client.post(format!("{}/session/add", BASE))
        .json(&json!({"user_key": member_key, "session_key": skey}))
        .send().await;

    // member forgets it
    let res = client.delete(format!("{}/session/forget", BASE))
        .json(&json!({"user_key": member_key, "session_key": skey}))
        .send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);

    forget_user(&client, &member_key).await;
    forget_user(&client, &owner_key).await;
}

// session/delete - owner deleting their session
#[tokio::test]
async fn test_delete_session() {
    let client = Client::new();
    let key = add_user(&client).await;
    let skey = create_session(&client, &key, "deletesessiontest").await;

    let res = client.delete(format!("{}/session/delete", BASE))
        .json(&json!({"user_key": key, "session_key": skey}))
        .send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);

    forget_user(&client, &key).await;
}