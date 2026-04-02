// hammering the server with concurrent requests - KAN-25

use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tokio::task;

const BASE: &str = "http://localhost:3000";

// fires 50 add requests all at the same time
#[tokio::test]
async fn test_add_concurrent() {
    let client = Arc::new(Client::new());
    let start = Instant::now();
    let mut handles = vec![];

    for _ in 0..50 {
        let c = client.clone();
        handles.push(task::spawn(async move {
            c.get(format!("{}/server/add", BASE)).send().await
        }));
    }

    let mut ok = 0;
    let mut fail = 0;
    for h in handles {
        match h.await.unwrap() {
            Ok(r) if r.status().as_u16() == 200 => ok += 1,
            Ok(r) => { println!("got {}", r.status()); fail += 1 }
            Err(e) => { println!("err: {}", e); fail += 1 }
        }
    }

    println!("concurrent add x50: {} ok, {} fail in {:.2?}", ok, fail, start.elapsed());
    assert!(ok >= 30, "too many add failures under load: {}", fail);
}

// gets a key then fires 50 connect requests all at once
#[tokio::test]
async fn test_connect_concurrent() {
    let client = Arc::new(Client::new());

    let add = client
        .get(format!("{}/server/add", BASE))
        .send().await.unwrap()
        .json::<Value>().await.unwrap();

    let key = Arc::new(add["user_key"].as_str().unwrap().to_string());
    println!("using key: {}", key);

    let start = Instant::now();
    let mut handles = vec![];

    for _ in 0..50 {
        let c = client.clone();
        let k = key.clone();
        handles.push(task::spawn(async move {
            c.get(format!("{}/server/connect/{}", BASE, k)).send().await
        }));
    }

    let mut no_crash = 0;
    for h in handles {
        if let Ok(Ok(r)) = h.await {
            if r.status().as_u16() < 500 { no_crash += 1 }
        }
    }

    println!("concurrent connect x50: {} non-500 in {:.2?}", no_crash, start.elapsed());
    assert!(no_crash >= 40, "server 500d too much under concurrent load: {}", no_crash);
}

// fires mixed add and connect concurrently at the same time
#[tokio::test]
async fn test_mixed_concurrent() {
    let client = Arc::new(Client::new());
    let start = Instant::now();
    let mut handles = vec![];

    for i in 0..50 {
        let c = client.clone();
        let url = if i % 2 == 0 {
            format!("{}/server/add", BASE)
        } else {
            format!("{}/server/connect/fakekey{}", BASE, i)
        };
        handles.push(task::spawn(async move {
            c.get(url).send().await
        }));
    }

    let mut ok = 0;
    for h in handles {
        if let Ok(Ok(r)) = h.await {
            if r.status().as_u16() < 500 { ok += 1 }
        }
    }

    println!("concurrent mixed x50: {} ok in {:.2?}", ok, start.elapsed());
    assert!(ok >= 35, "too many failed in concurrent mixed: {}", ok);
}