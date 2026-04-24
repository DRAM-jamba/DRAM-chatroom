// updating tests to match the new session routes - KAN-106
// routes changed on KAN-91, mainly session connect/forget/delete params

use reqwest::Client;
use serde_json::Value;
use std::fs;

const BASE: &str = "http://localhost:3000";

const USER_LIST: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/data/user_list.json");
const SESSION_LIST: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/data/session_list.json");

fn read_users() -> Vec<Value> {
    let data = fs::read_to_string(USER_LIST).unwrap_or("[]".into());
    serde_json::from_str(&data).unwrap_or(vec![])
}

fn read_sessions() -> Vec<Value> {
    let data = fs::read_to_string(SESSION_LIST).unwrap_or("[]".into());
    serde_json::from_str(&data).unwrap_or(vec![])
}

#[tokio::test]
async fn test_add_user() {
    let client = Client::new();
    let before = read_users().len();

    let res = client.get(format!("{}/server/add", BASE)).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);

    let body: Value = res.json().await.unwrap();
    let key = body["user_key"].as_str().unwrap().to_string();
    assert!(body["auth_token"].is_string());
    assert!(body["user_key"].is_string());

    let after = read_users();
    assert!(after.len() > before, "user wasnt saved to user_list.json");
    println!("user {} created and confirmed in json", key);

    let _ = client.get(format!("{}/server/forget/{}", BASE, key)).send().await;
}

#[tokio::test]
async fn test_connect_valid_key() {
    let client = Client::new();

    let add = client.get(format!("{}/server/add", BASE))
        .send().await.unwrap().json::<Value>().await.unwrap();
    let key = add["user_key"].as_str().unwrap().to_string();

    let res = client.get(format!("{}/server/connect/{}", BASE, key)).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);

    let _ = client.get(format!("{}/server/forget/{}", BASE, key)).send().await;
}

#[tokio::test]
async fn test_connect_bad_key() {
    let res = Client::new()
        .get(format!("{}/server/connect/thiskeyisnotreal", BASE))
        .send().await.unwrap();
    assert_eq!(res.status().as_u16(), 404);
}

#[tokio::test]
async fn test_leave_server() {
    let res = Client::new().get(format!("{}/server/leave", BASE)).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["message"], "not done yet");
}

#[tokio::test]
async fn test_forget_server() {
    let client = Client::new();

    let add = client.get(format!("{}/server/add", BASE))
        .send().await.unwrap().json::<Value>().await.unwrap();
    let key = add["user_key"].as_str().unwrap().to_string();

    let res = client.get(format!("{}/server/forget/{}", BASE, key)).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
}

#[tokio::test]
async fn test_set_nickname() {
    let client = Client::new();

    let add = client.get(format!("{}/server/add", BASE))
        .send().await.unwrap().json::<Value>().await.unwrap();
    let key = add["user_key"].as_str().unwrap().to_string();

    let res = client.get(format!("{}/server/set/nickname/{}/testname", BASE, key))
        .send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);

    let _ = client.get(format!("{}/server/forget/{}", BASE, key)).send().await;
}

#[tokio::test]
async fn test_get_session_list() {
    let client = Client::new();

    let add = client.get(format!("{}/server/add", BASE))
        .send().await.unwrap().json::<Value>().await.unwrap();
    let key = add["user_key"].as_str().unwrap().to_string();

    let res = client.get(format!("{}/session/list/{}", BASE, key)).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);

    let body: Value = res.json().await.unwrap();
    assert!(body["related_sessions"].is_array());

    let _ = client.get(format!("{}/server/forget/{}", BASE, key)).send().await;
}

#[tokio::test]
async fn test_create_session() {
    let client = Client::new();
    let before = read_sessions().len();

    let add = client.get(format!("{}/server/add", BASE))
        .send().await.unwrap().json::<Value>().await.unwrap();
    let key = add["user_key"].as_str().unwrap().to_string();

    let res = client.get(format!("{}/session/create/{}/mysession", BASE, key))
        .send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);

    let body: Value = res.json().await.unwrap();
    let skey = body["session_key"].as_str().unwrap().to_string();

    let after = read_sessions().len();
    println!("sessions before: {}, after: {}", before, after);
    assert!(after > before, "session wasnt written to session_list.json");

    // forget now takes user_key and session_key, changed from old single param
    let _ = client.get(format!("{}/session/forget/{}/{}", BASE, key, skey)).send().await;
    let _ = client.get(format!("{}/server/forget/{}", BASE, key)).send().await;
}

#[tokio::test]
async fn test_add_session() {
    let client = Client::new();

    let add = client.get(format!("{}/server/add", BASE))
        .send().await.unwrap().json::<Value>().await.unwrap();
    let key = add["user_key"].as_str().unwrap().to_string();

    let res = client.get(format!("{}/session/add/{}/somesessionkey", BASE, key))
        .send().await.unwrap();
    assert!(res.status().as_u16() < 500);

    let _ = client.get(format!("{}/server/forget/{}", BASE, key)).send().await;
}

// connect now does websocket upgrade so plain GET returns 400
// just checking server doesnt crash, not testing actual ws here
#[tokio::test]
async fn test_connect_to_session_no_ws() {
    let client = Client::new();

    let add = client.get(format!("{}/server/add", BASE))
        .send().await.unwrap().json::<Value>().await.unwrap();
    let key = add["user_key"].as_str().unwrap().to_string();

    let create = client.get(format!("{}/session/create/{}/wstest", BASE, key))
        .send().await.unwrap().json::<Value>().await.unwrap();
    let skey = create["session_key"].as_str().unwrap().to_string();

    let res = client.get(format!("{}/session/connect/{}/{}", BASE, key, skey))
        .send().await.unwrap();
    println!("connect without ws headers: {}", res.status());
    assert!(res.status().as_u16() < 500, "server crashed on ws connect");

    let _ = client.get(format!("{}/session/forget/{}/{}", BASE, key, skey)).send().await;
    let _ = client.get(format!("{}/server/forget/{}", BASE, key)).send().await;
}

#[tokio::test]
async fn test_leave_session() {
    let res = Client::new().get(format!("{}/session/leave", BASE)).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
}

// forget session changed - now needs user_key and session_key together
#[tokio::test]
async fn test_forget_session() {
    let client = Client::new();

    let add = client.get(format!("{}/server/add", BASE))
        .send().await.unwrap().json::<Value>().await.unwrap();
    let key = add["user_key"].as_str().unwrap().to_string();

    let create = client.get(format!("{}/session/create/{}/forgetsession", BASE, key))
        .send().await.unwrap().json::<Value>().await.unwrap();
    let skey = create["session_key"].as_str().unwrap().to_string();

    // add session to user first so forget can find it
    let _ = client.get(format!("{}/session/add/{}/{}", BASE, key, skey))
        .send().await;

    let res = client.get(format!("{}/session/forget/{}/{}", BASE, key, skey))
        .send().await.unwrap();

    // forget returns empty 200 on success, not json
    assert!(res.status().as_u16() < 500, "forget session failed: {}", res.status());

    let _ = client.get(format!("{}/server/forget/{}", BASE, key)).send().await;
}

// delete order was reversed from old tests - user_key first, then session_key
#[tokio::test]
async fn test_delete_session() {
    let client = Client::new();

    let add = client.get(format!("{}/server/add", BASE))
        .send().await.unwrap().json::<Value>().await.unwrap();
    let key = add["user_key"].as_str().unwrap().to_string();

    let create = client.get(format!("{}/session/create/{}/deletesession", BASE, key))
        .send().await.unwrap().json::<Value>().await.unwrap();
    let skey = create["session_key"].as_str().unwrap().to_string();

    let res = client.get(format!("{}/session/delete/{}/{}", BASE, key, skey))
        .send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);

    let _ = client.get(format!("{}/server/forget/{}", BASE, key)).send().await;
}