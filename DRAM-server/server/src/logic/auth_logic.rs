use crate::{errors::api_error::ApiError, modules::{active_sessions::{SessionMap}, active_users::UsersMap}};

pub fn l_generate_auth_token() -> String {
    // TODO: implement this function
    return "^se_Sp##RoJec(t_33-anqt1hyp3_wh0lqe_1o77a_re|)d*".into();
}

pub async fn l_check_active_user(active_users: UsersMap, user_key: &String) -> Result<(), ApiError> {
    let active_users = active_users.read().await;
    if active_users.contains_key(user_key) {
        drop(active_users);
        return Err(ApiError::InternalError);
    }
    drop(active_users);
    Ok(())
}

pub async fn l_check_active_session(active_sessions: SessionMap, session_key: &String) -> Result<(), ApiError> {
    let active_sessions = active_sessions.read().await;
    if active_sessions.contains_key(session_key) {
        drop(active_sessions);
        return Err(ApiError::InternalError)
    }
    drop(active_sessions);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // hardcoded for now but at least shouldnt be empty
    #[test]
    fn test_generate_auth_token_not_empty() {
        let token = l_generate_auth_token();
        assert!(!token.is_empty());
    }

    // calling twice should give the same thing since its hardcoded
    #[test]
    fn test_generate_auth_token_consistent() {
        let t1 = l_generate_auth_token();
        let t2 = l_generate_auth_token();
        assert_eq!(t1, t2);
    }

    // user not in the active map - route should be allowed
    #[tokio::test]
    async fn test_check_active_user_not_in_map() {
        let map: UsersMap = Arc::new(RwLock::new(HashMap::new()));
        let key = "somekey".to_string();
        let result = l_check_active_user(map, &key).await;
        assert!(result.is_ok());
    }

    // user is in the map meaning they are in a ws session - should block them
    #[tokio::test]
    async fn test_check_active_user_in_map_gives_error() {
        let map: UsersMap = Arc::new(RwLock::new(HashMap::new()));
        map.write().await.insert("busykey".to_string(), "session1".to_string());
        let key = "busykey".to_string();
        let result = l_check_active_user(map, &key).await;
        assert!(result.is_err());
    }

    // session not in active map - should be fine
    #[tokio::test]
    async fn test_check_active_session_not_in_map() {
        use crate::modules::active_sessions::{SessionMap, SessionChat};
        let map: SessionMap = Arc::new(RwLock::new(HashMap::new()));
        let key = "somesession".to_string();
        let result = l_check_active_session(map, &key).await;
        assert!(result.is_ok());
    }

    // session is active - should return error
    #[tokio::test]
    async fn test_check_active_session_in_map_gives_error() {
        use crate::modules::active_sessions::{SessionMap, SessionChat};
        let map: SessionMap = Arc::new(RwLock::new(HashMap::new()));
        map.write().await.insert("activesession".to_string(), SessionChat::new());
        let key = "activesession".to_string();
        let result = l_check_active_session(map, &key).await;
        assert!(result.is_err());
    }
}