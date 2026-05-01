use crate::{data_logic::{connection_data::d_get_user_role, session_data::d_get_session, user_data::{d_get_user}}, errors::api_error::ApiError, modules::{active_sessions::{SessionChat, SessionMap}, server_state::ServerState, user::User}};
use axum::{extract::{WebSocketUpgrade, ws::{Message, WebSocket}}, response::IntoResponse};
use futures_util::{SinkExt, StreamExt};

pub async fn l_connection_handler(server_state: ServerState, user_key: String, session_key: String, 
                                ws: WebSocketUpgrade) -> Result<impl IntoResponse, ApiError> {
    match d_get_session(server_state.db_pool.clone(), &session_key).await {
        Ok(_s) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };
    match d_get_user_role(server_state.db_pool.clone(), &user_key, &session_key).await {
        Ok(_c) => (),
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };
    let user = match d_get_user(server_state.db_pool.clone(), &user_key).await {
        Ok(u) => u,
        Err(e) => return Err(ApiError::InvalidInput(e.to_string()))
    };

    let mut active_users = server_state.active_users.write().await;
    if active_users.contains_key(user_key.as_str()) {
        drop(active_users);
        return Err(ApiError::InvalidInput("User already in session".into()))
    }
    else {
        active_users.insert(user_key.clone(), session_key.clone());
    }
    drop(active_users);

    let session_chat: SessionChat = l_get_or_create_active_session(&server_state.active_sessions, &session_key).await;    

    let new_session_key = session_key.clone();

    let response = ws.on_upgrade(move |socket| { 
        l_handle_websocket(session_chat, socket, user, new_session_key, server_state.clone())
    });

    Ok(response) 
}

async fn l_get_or_create_active_session(active_sessions: &SessionMap, session_key: &String) -> SessionChat {
    let mut map = active_sessions.write().await;

    let sc = map.entry(session_key.clone())
       .or_insert_with(|| SessionChat::new())
       .clone();
    
    drop(map);

    sc
}

async fn l_handle_websocket(session_chat: SessionChat,mut ws: WebSocket, user: User, session_key: String, server_state: ServerState) {

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
                    l_broadcast_message(&session_chat, text.to_string(), &user.nickname).await;
                },
                Message::Binary(_bytes) => {}, // TODO: use this to send images. for later
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
        let mut active_sessions = server_state.active_sessions.write().await;
        active_sessions.remove(&session_key);
        drop(active_sessions);
    }

    let mut active_users = server_state.active_users.write().await;
    active_users.remove(&user.user_key);
    drop(active_users);

}

async fn l_broadcast_message(session_chat: &SessionChat, mut msg: String, nickname: &String) {
    let mut history = session_chat.history.write().await;
    msg = format!("{nickname}: {msg}");
    if history.len() >= 100 { // messages limit
        history.pop_front();
    }
    history.push_back(msg.clone());
    drop(history);
    let _ = session_chat.tx.send(msg);
}