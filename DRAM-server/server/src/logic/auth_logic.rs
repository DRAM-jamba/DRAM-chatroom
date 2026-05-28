use chrono::{Duration, Utc};
use dotenvy_macro::dotenv;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sqlx::{Pool, Postgres};

use crate::{data_logic::user_data::d_get_user, errors::api_error::ApiError, modules::{active_nonce::Nonce, active_sessions::SessionMap, active_users::UsersMap, request_bodies::ChallengeResponse, server_state::ServerState, token::Claims}};

pub async fn l_create_challenge(server_state: ServerState, user_key: &String) -> Result<String, ApiError> {
    
    // check user existance
    let _user = match d_get_user(server_state.db_pool.clone(), user_key).await {
        Ok(u) => u,
        Err(e) => return Err(e.into())
    };

    // create nonce
    let mut raw = [0u8; 32];
    rand::fill(&mut raw);
    let nonce = hex::encode(raw);

    let expires_at = Utc::now() + Duration::minutes(2);

    // save nonce in state
    server_state.active_nonce.insert(user_key.clone(), Nonce {
        nonce: nonce.clone(),
        expires_at: expires_at
    });

    Ok(nonce)
}

pub async fn l_handle_challenge(server_state: ServerState, payload: ChallengeResponse) -> Result<String, ApiError> {

    // check user, check nonce
    let user = match d_get_user(server_state.db_pool.clone(), &payload.user_key).await {
        Ok(u) => u,
        Err(e) => return Err(e.into())
    };

    let (_, pending) = match server_state.active_nonce.remove(&payload.user_key) {
        Some(p) => p,
        None => return Err(ApiError::Unauthorized)
    };

    if Utc::now() > pending.expires_at {
        return Err(ApiError::Unauthorized);
    }

    if pending.nonce != payload.nonce {
        return Err(ApiError::Unauthorized)
    }

    // decode public key from hex
    let pub_key_bytes = hex::decode(&user.public_key)
        .map_err(|_| ApiError::InternalError)?;

    let pub_key_array: [u8; 32] = pub_key_bytes
        .try_into()
        .map_err(|_| ApiError::InternalError)?;

    let verifying_key = VerifyingKey::from_bytes(&pub_key_array)
        .map_err(|_| ApiError::InternalError)?;

    // decode signature from hex
    let sig_bytes = hex::decode(&payload.signature)
        .map_err(|_| ApiError::InternalError)?;

    let sig_array: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| ApiError::InternalError)?;

    let signature = Signature::from_bytes(&sig_array);


    verifying_key
        .verify(pending.nonce.as_bytes(), &signature)
        .map_err(|_| ApiError::Unauthorized)?;
    
    // create token
    let token = l_create_jwt(&payload.user_key).await?;

    // send token
    Ok(token)
}

pub async fn l_refresh_token(db_pool: Pool<Postgres>, user_key: &String) -> Result<String, ApiError> {
    let _user = d_get_user(db_pool, user_key).await?;

    let token = l_create_jwt(user_key).await?;

    Ok(token)
}


pub async fn l_create_jwt(user_key: &String) -> Result<String, ApiError> {
    
    let mut now = Utc::now();
    let iat = now.timestamp() as usize;
    let expires_in = Duration::minutes(10);
    now += expires_in;
    let exp = now.timestamp() as usize;
    let claim = Claims {exp, iat, sub: user_key.to_string()};

    let secret = dotenv!("JWT_SECRET");
    let key = EncodingKey::from_secret(secret.as_bytes());

    encode(&Header::default(), &claim, &key)
        .map_err(|_| ApiError::InternalError)
}

pub async fn l_is_valid(token: &String) -> Result<Claims, ApiError> {
    
    let secret = dotenv!("JWT_SECRET");
    let key = DecodingKey::from_secret(secret.as_bytes());
    let claim = decode::<Claims>(token, &key, &Validation::default())
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => ApiError::Unauthorized,
            _ => ApiError::InternalError
        })?;

    Ok(claim.claims)
}

pub async fn l_ensure_user_not_in_session(active_users: UsersMap, user_key: &String) -> Result<(), ApiError> {
    let contains = active_users.read().await.contains_key(user_key);
    if contains {
        return Err(ApiError::Forbidden("User in session. Leave session and try again.".into()));
    }
    Ok(())
}

pub async fn l_ensure_session_is_not_active(active_sessions: SessionMap, session_key: &String) -> Result<(), ApiError> {
    let contains = active_sessions.read().await.contains_key(session_key);
    if contains {
        return Err(ApiError::Forbidden("Some users use this session right now. Wait until this session will not be active.".into()))
    }
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::collections::HashMap;
//     use std::sync::Arc;
//     use tokio::sync::RwLock;

//     // hardcoded for now but at least shouldnt be empty
//     #[test]
//     fn test_generate_auth_token_not_empty() {
//         let token = l_generate_auth_token();
//         assert!(!token.is_empty());
//     }

//     // calling twice should give the same thing since its hardcoded
//     #[test]
//     fn test_generate_auth_token_consistent() {
//         let t1 = l_generate_auth_token();
//         let t2 = l_generate_auth_token();
//         assert_eq!(t1, t2);
//     }

//     // user not in the active map - route should be allowed
//     #[tokio::test]
//     async fn test_check_active_user_not_in_map() {
//         let map: UsersMap = Arc::new(RwLock::new(HashMap::new()));
//         let key = "somekey".to_string();
//         let result = l_ensure_user_not_in_session(map, &key).await;
//         assert!(result.is_ok());
//     }

//     // user is in the map meaning they are in a ws session - should block them
//     #[tokio::test]
//     async fn test_check_active_user_in_map_gives_error() {
//         let map: UsersMap = Arc::new(RwLock::new(HashMap::new()));
//         map.write().await.insert("busykey".to_string(), "session1".to_string());
//         let key = "busykey".to_string();
//         let result = l_ensure_user_not_in_session(map, &key).await;
//         assert!(result.is_err());
//     }

//     // session not in active map - should be fine
//     #[tokio::test]
//     async fn test_check_active_session_not_in_map() {
//         use crate::modules::active_sessions::{SessionMap};
//         let map: SessionMap = Arc::new(RwLock::new(HashMap::new()));
//         let key = "somesession".to_string();
//         let result = l_ensure_session_is_not_active(map, &key).await;
//         assert!(result.is_ok());
//     }

//     // session is active - should return error
//     #[tokio::test]
//     async fn test_check_active_session_in_map_gives_error() {
//         use crate::modules::active_sessions::{SessionMap, SessionChat};
//         let map: SessionMap = Arc::new(RwLock::new(HashMap::new()));
//         map.write().await.insert("activesession".to_string(), SessionChat::new());
//         let key = "activesession".to_string();
//         let result = l_ensure_session_is_not_active(map, &key).await;
//         assert!(result.is_err());
//     }
// }