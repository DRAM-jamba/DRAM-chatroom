use crate::{data_logic::session_data::get_session_by_session_key, errors::{api_error::ApiError, app_error::AppError}, modules::session_chat::{SessionChat, SessionMap}};
use axum::{extract::{WebSocketUpgrade, ws::{Message, WebSocket}}, response::IntoResponse};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast::{self, Sender};

pub async fn connection_handler(active_sessions: SessionMap, session_key: String, 
                                ws: WebSocketUpgrade) -> Result<impl IntoResponse, ApiError> {
    match get_session_by_session_key(&session_key) {
        Ok(s) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };
    // TODO: maybe check if the session is related to user?
    let session_chat: SessionChat = get_or_create_active_session(&active_sessions, &session_key).await;    

    let response = ws.on_upgrade(move |socket| { 
        handle_websocket(session_chat, socket, session_key, active_sessions.clone())
    });

    Ok(response) 
}

async fn get_or_create_active_session(active_sessions: &SessionMap, session_key: &String) -> SessionChat {
    let mut map = active_sessions.write().await;

    let sc = map.entry(session_key.clone())
       .or_insert_with(|| SessionChat::new())
       .clone();
    
    sc
}

async fn handle_websocket(session_chat: SessionChat,mut ws: WebSocket, session_key: String, active_sessions: SessionMap) {

    let history = session_chat.history.read().await;
    for msg in history.iter() {
        if ws.send(Message::Text(msg.clone().into())).await.is_err() {
            return;
        }
    }
    drop(history);

    let (mut sender, mut receiver) = ws.split();
    let mut rx = session_chat.tx.subscribe();

    //TODO: add ping logic to check connection

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    while let Some(result) = receiver.next().await {
        match result {
            Ok(msg) => match msg {
                Message::Text(text) => {
                    broadcast_message(&session_chat, text.to_string()).await;
                },
                Message::Binary(bytes) => {}, // TODO: use this to send images. for later
                Message::Ping(_) => {},
                Message::Pong(_) => {},
                Message::Close(frame) => {
                    match frame {
                        Some(cf) => {
                            println!("closed: code={}, reason={}", cf.code, cf.reason);
                        },
                        None => {println!("closed without frame");}
                    }
                    break;
                }
            },
            Err(e) => {
                eprintln!("websocket error: {}", e);
                break;
            }
        }
    }

    send_task.abort(); // it is not happening in moment, so tx.receiver_count think that it is still exits

    if session_chat.tx.receiver_count() == 1 { // TODO: not sure in this. but it works fine
        active_sessions.write().await.remove(&session_key);
    }
}

async fn broadcast_message(session_chat: &SessionChat, msg: String) {
    let mut history = session_chat.history.write().await;
    if history.len() >= 100 {
        history.pop_front();
    }
    history.push_back(msg.clone());
    drop(history);
    let _ = session_chat.tx.send(msg);
}