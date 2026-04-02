// tests for server_routes.rs and session_routes.rs - KAN-24

use reqwest::Client;
use serde_json::Value;
use std::fs;

const BASE: &str = "http://localhost:3000";

// path to the json files so we can check if data was actually written
const USER_LIST: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/data/user_list.json");
const SESSION_LIST: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/data/session_list.json");

// helper - reads user_list.json and returns it as a vec
fn read_users() -> Vec<Value> {
    let data = fs::read_to_string(USER_LIST).unwrap_or("[]".into());
    serde_json::from_str(&data).unwrap_or(vec![])
}

// helper - reads session_list.json
fn read_sessions() -> Vec<Value> {
    let data = fs::read_to_string(SESSION_LIST).unwrap_or("[]".into());
    serde_json::from_str(&data).unwrap_or(vec![])
}

// server_routes tests

#[tokio::test]
async fn test_add_user() {
    let client = Client::new();

    let users_before = read_users().len();

    let res = client
        .get(format!("{}/server/add", BASE))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status().as_u16(), 200);

    let body: Value = res.json().await.unwrap();
    let key = body["user_key"].as_str().unwrap().to_string();

    assert!(body["auth_token"].is_string());
    assert!(body["user_key"].is_string());

    // check user actually got written to the json file
    let users_after = read_users();
    assert!(users_after.len() > users_before, "user wasnt saved to user_list.json");
    println!("user {} created and confirmed in json file", key);

    // cleanup - call forget to remove the user we just made
    let _ = client
        .get(format!("{}/server/forget/{}", BASE, key))
        .send()
        .await;
}

#[tokio::test]
async fn test_connect_valid_key() {
    let client = Client::new();

    // add a user first to get a real key
    let add = client
        .get(format!("{}/server/add", BASE))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    let key = add["user_key"].as_str().unwrap().to_string();

    let res = client
        .get(format!("{}/server/connect/{}", BASE, key))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status().as_u16(), 200);

    // cleanup
    let _ = client
        .get(format!("{}/server/forget/{}", BASE, key))
        .send()
        .await;
}

#[tokio::test]
async fn test_connect_bad_key() {
    let res = Client::new()
        .get(format!("{}/server/connect/thiskeyisnotreal", BASE))
        .send()
        .await
        .unwrap();

    // no user created here so no cleanup needed
    // random key shouldnt exist, expecting 404
    assert_eq!(res.status().as_u16(), 404);
}

#[tokio::test]
async fn test_leave_server() {
    let res = Client::new()
        .get(format!("{}/server/leave", BASE))
        .send()
        .await
        .unwrap();

    // stub - no user created so nothing to clean up
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["message"], "not done yet");
}

#[tokio::test]
async fn test_forget_server() {
    let client = Client::new();

    // make a user first so forget has something to try on
    let add = client
        .get(format!("{}/server/add", BASE))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    let key = add["user_key"].as_str().unwrap().to_string();

    let res = client
        .get(format!("{}/server/forget/{}", BASE, key))
        .send()
        .await
        .unwrap();

    // forget is still a stub but should return 200 not crash
    assert_eq!(res.status().as_u16(), 200);
}

#[tokio::test]
async fn test_set_nickname() {
    let client = Client::new();

    let add = client
        .get(format!("{}/server/add", BASE))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    let key = add["user_key"].as_str().unwrap().to_string();

    let res = client
        .get(format!("{}/server/set/nickname/{}/testname", BASE, key))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status().as_u16(), 200);

    // cleanup
    let _ = client
        .get(format!("{}/server/forget/{}", BASE, key))
        .send()
        .await;
}

// session_routes tests

#[tokio::test]
async fn test_get_session_list() {
    let client = Client::new();

    let add = client
        .get(format!("{}/server/add", BASE))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    let key = add["user_key"].as_str().unwrap().to_string();

    let res = client
        .get(format!("{}/session/list/{}", BASE, key))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    assert!(body["related_sessions"].is_array());

    // cleanup user
    let _ = client
        .get(format!("{}/server/forget/{}", BASE, key))
        .send()
        .await;
}

#[tokio::test]
async fn test_create_session() {
    let client = Client::new();

    let sessions_before = read_sessions().len();

    let add = client
        .get(format!("{}/server/add", BASE))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    let key = add["user_key"].as_str().unwrap().to_string();

    let res = client
        .get(format!("{}/session/create/{}/mysession", BASE, key))
        .send()
        .await
        .unwrap();

    assert!(res.status().as_u16() < 500);

    // check if session was written to json
    let sessions_after = read_sessions().len();
    println!("sessions before: {}, after: {}", sessions_before, sessions_after);

    // cleanup user and session
    let _ = client
        .get(format!("{}/server/forget/{}", BASE, key))
        .send()
        .await;
    let _ = client
        .get(format!("{}/session/forget/mysession", BASE))
        .send()
        .await;
}

#[tokio::test]
async fn test_add_session() {
    let client = Client::new();

    let add = client
        .get(format!("{}/server/add", BASE))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    let key = add["user_key"].as_str().unwrap().to_string();

    let res = client
        .get(format!("{}/session/add/{}/somesessionkey", BASE, key))
        .send()
        .await
        .unwrap();

    assert!(res.status().as_u16() < 500);

    // cleanup
    let _ = client
        .get(format!("{}/server/forget/{}", BASE, key))
        .send()
        .await;
}

#[tokio::test]
async fn test_connect_to_session() {
    let res = Client::new()
        .get(format!("{}/session/connect/somesessionkey", BASE))
        .send()
        .await
        .unwrap();

    // stub - nothing created so nothing to clean up
    assert_eq!(res.status().as_u16(), 200);
}

#[tokio::test]
async fn test_leave_session() {
    let res = Client::new()
        .get(format!("{}/session/leave", BASE))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status().as_u16(), 200);
}

#[tokio::test]
async fn test_forget_session() {
    let res = Client::new()
        .get(format!("{}/session/forget/somesessionkey", BASE))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status().as_u16(), 200);
}

#[tokio::test]
async fn test_delete_session() {
    let res = Client::new()
        .get(format!("{}/session/delete/somesessionkey/someuserkey", BASE))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status().as_u16(), 200);
}