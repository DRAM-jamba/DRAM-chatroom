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