// hammering the server with requests to see if it breaks - KAN-25
// cargo run in one window, then run this binary in another

use reqwest::Client;
use serde_json::Value;
use std::time::Instant;

const BASE: &str = "http://localhost:3000";

#[tokio::test]
async fn test_add_many_times() {
    let client = Client::new();
    let start = Instant::now();
    let mut ok_count = 0;

    for i in 0..50 {
        let r = client.get(format!("{}/server/add", BASE)).send().await;
        match r {
            Ok(resp) if resp.status().as_u16() == 200 => ok_count += 1,
            Ok(resp) => println!("req {} got {}", i, resp.status()),
            Err(e) => println!("req {} errored: {}", i, e),
        }
    }

    println!("add x50: {} ok in {:.2?}", ok_count, start.elapsed());
    assert!(ok_count >= 45, "too many add requests failed: {}", ok_count);
}

#[tokio::test]
async fn test_connect_many_times() {
    let client = Client::new();

    // get a key to use
    let add = client
        .get(format!("{}/server/add", BASE))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    let key = add["user_key"].as_str().unwrap().to_string();
    let start = Instant::now();
    let mut no_crash = 0;

    for i in 0..50 {
        let r = client
            .get(format!("{}/server/connect/{}", BASE, key))
            .send()
            .await;
        match r {
            Ok(resp) if resp.status().as_u16() < 500 => no_crash += 1,
            Ok(resp) => println!("req {} server error: {}", i, resp.status()),
            Err(e) => println!("req {} failed: {}", i, e),
        }
    }

    println!("connect x50: {} non-500 in {:.2?}", no_crash, start.elapsed());
    // 500 means server crashed, anything else is ok
    assert!(no_crash >= 45, "server returned too many 500s: {}", no_crash);
}

#[tokio::test]
async fn test_mixed_load() {
    let client = Client::new();
    let start = Instant::now();
    let mut ok = 0;

    for i in 0..50 {
        // mix of add and connect with fake keys
        let url = if i % 2 == 0 {
            format!("{}/server/add", BASE)
        } else {
            format!("{}/server/connect/fakekey{}", BASE, i)
        };

        if let Ok(r) = client.get(&url).send().await {
            if r.status().as_u16() < 500 { ok += 1 }
        }
    }

    println!("mixed x50: {} ok in {:.2?}", ok, start.elapsed());
    assert!(ok >= 40, "too many failed in mixed load: {}", ok);
}