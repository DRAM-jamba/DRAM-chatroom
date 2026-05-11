use crate::{errors::api_error::ApiError, modules::{active_sessions::{SessionMap}, active_users::UsersMap}};

pub fn l_generate_auth_token() -> String {
    // TODO: implement this function

    "^se_Sp##RoJec(t_33-anqt1hyp3_wh0lqe_1o77a_re|)d*".into()
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