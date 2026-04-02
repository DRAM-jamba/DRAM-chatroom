// testing all the routes from server_routes.rs and session_routes.rs

use reqwest::Client;
use serde_json::Value;

const BASE: &str = "http://localhost:3000";

// server_routes

#[tokio::test]
async fn test_add_user() {
    let res = Client::new()
        .get(format!("{}/server/add", BASE))
        .send()
        .await
        .unwrap();

    // /server/add creates a new user, should give back a key and token
    assert_eq!(res.status().as_u16(), 200);

    let body: Value = res.json().await.unwrap();
    assert!(body["auth_token"].is_string());
    assert!(body["user_key"].is_string());
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

    let key = add["user_key"].as_str().unwrap();

    let res = client
        .get(format!("{}/server/connect/{}", BASE, key))
        .send()
        .await
        .unwrap();

    // key came from the server so it should work
    assert_eq!(res.status().as_u16(), 200);
}

#[tokio::test]
async fn test_connect_bad_key() {
    let res = Client::new()
        .get(format!("{}/server/connect/thiskeyisnotreal", BASE))
        .send()
        .await
        .unwrap();

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

    // still a stub but shouldnt crash
    assert_eq!(res.status().as_u16(), 200);

    let body: Value = res.json().await.unwrap();
    assert_eq!(body["message"], "not done yet");
}

#[tokio::test]
async fn test_forget_server() {
    let res = Client::new()
        .get(format!("{}/server/forget/somekey", BASE))
        .send()
        .await
        .unwrap();

    // stub, just checking it returns 200
    assert_eq!(res.status().as_u16(), 200);
}

#[tokio::test]
async fn test_set_nickname() {
    let client = Client::new();

    // need a real user key first
    let add = client
        .get(format!("{}/server/add", BASE))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    let key = add["user_key"].as_str().unwrap();

    let res = client
        .get(format!("{}/server/set/nickname/{}/testname", BASE, key))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status().as_u16(), 200);
}

// session_routes

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

    let key = add["user_key"].as_str().unwrap();

    let res = client
        .get(format!("{}/session/list/{}", BASE, key))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status().as_u16(), 200);

    let body: Value = res.json().await.unwrap();
    // should get back a list even if its empty
    assert!(body["related_sessions"].is_array());
}

#[tokio::test]
async fn test_create_session() {
    let client = Client::new();

    let add = client
        .get(format!("{}/server/add", BASE))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    let key = add["user_key"].as_str().unwrap();

    let res = client
        .get(format!("{}/session/create/{}/mysession", BASE, key))
        .send()
        .await
        .unwrap();

    // not totally sure what this gives back yet, just checking no 500
    assert!(res.status().as_u16() < 500);
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

    let key = add["user_key"].as_str().unwrap();

    let res = client
        .get(format!("{}/session/add/{}/somesessionkey", BASE, key))
        .send()
        .await
        .unwrap();

    assert!(res.status().as_u16() < 500);
}

#[tokio::test]
async fn test_connect_to_session() {
    let res = Client::new()
        .get(format!("{}/session/connect/somesessionkey", BASE))
        .send()
        .await
        .unwrap();

    // stub, should be 200
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

    // stub
    assert_eq!(res.status().as_u16(), 200);
}