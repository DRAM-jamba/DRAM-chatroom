//! Websocket module for DRAM client.
//! client handles the websocket connection and communication with the server.
//! heartbeat handles the periodic heartbeat pings to keep the connection alive.
mod client;
mod heartbeat;
pub use client::WsClient;
pub use heartbeat::{start as start_heartbeat, stop as stop_heartbeat};