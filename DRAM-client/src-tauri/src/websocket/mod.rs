//! Websocket module for DRAM client.
//! client handles the websocket connection and communication with the server.
//! heartbeat handles the periodic heartbeat pings to keep the connection alive.
pub mod client;
pub mod heartbeat;

pub use self::client::WebsocketClient;
pub use self::heartbeat::WebsocketHeartbeat;