use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration};

use crate::{data_logic::{connection_data::d_get_user_role, session_data::d_get_session, user_data::d_get_user}, errors::api_error::ApiError, modules::{active_sessions::{SessionChat, SessionMap}, message::{BackMessageObj, MessageObj, MessageType}, server_state::ServerState, user::User}};
use axum::{extract::{WebSocketUpgrade, ws::{Message, WebSocket}}, response::IntoResponse};
use futures_util::{SinkExt, StreamExt};
use tokio::time::{MissedTickBehavior, interval};

const PING_TIME: u64 = 30;

pub async fn l_connection_handler(server_state: ServerState, user_key: String, session_key: String, 
                                ws: WebSocketUpgrade) -> Result<impl IntoResponse, ApiError> {
    match d_get_session(server_state.db_pool.clone(), &session_key).await {
        Ok(_s) => (),
        Err(e) => return Err(e.into())
    };
    let user = match d_get_user(server_state.db_pool.clone(), &user_key).await {
        Ok(u) => u,
        Err(e) => return Err(e.into())
    };
    match d_get_user_role(server_state.db_pool.clone(), &user_key, &session_key).await {
        Ok(_c) => (),
        Err(e) => return Err(e.into())
    };

    // i check if user in active_users in session_routes, so he is not in active_users
    server_state.active_users.write().await.insert(user_key.clone(), session_key.clone());
    
    let session_chat: SessionChat = l_get_or_create_active_session(&server_state.active_sessions, &session_key).await;    

    session_chat.users.write().await.push(user.nickname.clone());

    let response = ws.on_upgrade(move |socket| { 
        l_handle_websocket(session_chat, socket, user, session_key, server_state.clone())
    });

    Ok(response) 
}

async fn l_get_or_create_active_session(active_sessions: &SessionMap, session_key: &String) -> SessionChat {
    active_sessions.write().await
                   .entry(session_key.clone())
                   .or_insert_with(|| SessionChat::new())
                   .clone()
}

async fn l_handle_websocket(session_chat: SessionChat,mut ws: WebSocket, user: User, session_key: String, server_state: ServerState) {

    let history = session_chat.history.read().await;
    for msg in history.iter() {
        let json = match serde_json::to_string(&msg) {
            Ok(s) => s,
            Err(e) => format!("Problem with message sending: {e}")
        };
        if ws.send(Message::Text(json.into())).await.is_err() {
            return;
        }
    }

    let (mut sender, mut receiver) = ws.split();
    let mut rx = session_chat.tx.subscribe();
    let (dead_tx, dead_rx) = tokio::sync::oneshot::channel::<()>();

    let waiting_for_pong = Arc::new(AtomicBool::new(false));
    let waiting_for_pong_clone = waiting_for_pong.clone();

    let send_task = tokio::spawn(async move {
        
        let mut ping_interval = interval(Duration::from_secs(PING_TIME));
        ping_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = ping_interval.tick() => {
                    if waiting_for_pong.load(Ordering::Relaxed) {
                        let _ = dead_tx.send(());
                        break;
                    }
                    waiting_for_pong.store(true, Ordering::Relaxed);
                    if sender.send(Message::Ping(vec![].into())).await.is_err() {
                        break;
                    }
                }
                msg = rx.recv() => {
                    match msg {
                        Ok(m) => {
                            if sender.send(Message::Text(m.into())).await.is_err() {
                                let _ = dead_tx.send(());
                                break;
                            }
                        },
                        Err(_e) => ()
                    }
                }
            }
        }

    });


    // Send starting messages.
    let user_list= session_chat.users.read().await;
    let json = match serde_json::to_string(&user_list.clone()) {
        Ok(s) => s,
        Err(e) => format!("Problem with message sending: {e}")
    };
    drop(user_list);

    let voice_list= session_chat.voice_users.read().await;
    let json_voice = match serde_json::to_string(&voice_list.clone()) {
        Ok(s) => s,
        Err(e) => format!("Problem with message sending: {e}")
    };
    drop(voice_list);

    drop(history); // drop here, so no new messages will appear until user's stuff are prepared

    l_broadcast_message(&session_chat, MessageType::Connect, "".into(), &user.nickname, true).await;
    l_broadcast_message(&session_chat, MessageType::UserList, json, &user.nickname, false).await;
    l_broadcast_message(&session_chat, MessageType::VoiceList, json_voice, &user.nickname, false).await;


    // Handling message from client
    tokio::select! {
        _ = async { // never closed, only when Close message appear
            while let Some(result) = receiver.next().await {
                match result {
                    Ok(msg) => match msg {
                        Message::Text(text) => {
                            l_handle_message(&session_chat, text.to_string(), &user.nickname).await;
                        },
                        Message::Binary(_bytes) => {}, // TODO: use this to send images. for later
                        Message::Ping(_) => {},
                        Message::Pong(_) => {
                            waiting_for_pong_clone.store(false, Ordering::Relaxed);
                        },
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
        } => {}
        _ = dead_rx => {} // when connection is lost, you gone
    }

    send_task.abort();

    // remove user from active_users, so user can join to other session
    server_state.active_users.write().await.remove(&user.user_key);

    // remove user from session users list
    let mut s_users = session_chat.users.write().await;
    match s_users.iter().position(|n| n == &user.nickname) {
        Some(i) => s_users.remove(i),
        None => "".to_string()
    };
    let is_empty = s_users.is_empty();
    let json = match serde_json::to_string(&s_users.clone()) {
        Ok(s) => s,
        Err(e) => format!("Problem with message sending: {e}")
    };
    drop(s_users);

    let mut s_voice_users = session_chat.voice_users.write().await;
    match s_voice_users.iter().position(|n| n == &user.nickname) {
        Some(i) => s_voice_users.remove(i),
        None => "".to_string()
    };
    let json_voice = match serde_json::to_string(&s_voice_users.clone()) {
        Ok(s) => s,
        Err(e) => format!("Problem with message sending: {e}")
    };
    drop(s_voice_users);

    l_broadcast_message(&session_chat, MessageType::Disconnect, "".into(), &user.nickname, true).await;
    l_broadcast_message(&session_chat, MessageType::UserList, json, &user.nickname, false).await;
    l_broadcast_message(&session_chat, MessageType::VoiceList, json_voice, &user.nickname, false).await;


    if is_empty { 
        let mut active_sessions = server_state.active_sessions.write().await;
        active_sessions.remove(&session_key);
        drop(active_sessions);
    }

}

async fn l_handle_message(session_chat: &SessionChat, text: String, nickname: &String) {
    let msg: BackMessageObj = match serde_json::from_str(&text) {
        Ok(m) => m,
        Err(e) => BackMessageObj {m_type: MessageType::Message, body: format!("Problem with message sending: {e}").into()}
    };
    match msg.m_type {
        MessageType::Message => l_broadcast_message(session_chat, MessageType::Message, msg.body, nickname, true).await,
        MessageType::VoiceStart => {
            let mut voice_users = session_chat.voice_users.write().await;
            voice_users.push(nickname.clone());
            let voice_users_list = voice_users.clone();
            drop(voice_users);

            let json_voice = match serde_json::to_string(&voice_users_list.clone()) {
                Ok(s) => s,
                Err(e) => format!("Problem with message sending: {e}")
            };

            l_broadcast_message(&session_chat, MessageType::VoiceList, json_voice, nickname, false).await;
        },
        MessageType::VoiceEnd => {
            let mut voice_users = session_chat.voice_users.write().await;
            match voice_users.iter().position(|n| n == nickname) {
                Some(i) => voice_users.remove(i),
                None => "".to_string()
            };
            let voice_users_list = voice_users.clone();
            drop(voice_users);

            let json_voice = match serde_json::to_string(&voice_users_list.clone()) {
                Ok(s) => s,
                Err(e) => format!("Problem with message sending: {e}")
            };
            
            l_broadcast_message(&session_chat, MessageType::VoiceList, json_voice, nickname, false).await;
        },
        _ => {},
    }

}

async fn l_broadcast_message(session_chat: &SessionChat, m_type: MessageType,  msg: String, nickname: &String, in_history: bool) {
    let ts = chrono::offset::Utc::now();
    let new_msg: MessageObj = MessageObj { m_type: m_type, from: nickname.clone(), body: msg, ts: ts.timestamp() };
    if in_history == true {
        let mut history = session_chat.history.write().await;
        if history.len() >= 100 { // messages limit
            history.pop_front();
        }
        history.push_back(new_msg.clone());
        drop(history);
    }
    let json = match serde_json::to_string(&new_msg) {
        Ok(s) => s,
        Err(e) => format!("Problem with message sending: {e}")
    };

    let _ = session_chat.tx.send(json);
}