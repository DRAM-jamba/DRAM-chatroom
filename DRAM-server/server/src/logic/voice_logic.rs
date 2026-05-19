use livekit_api::access_token::{AccessToken, VideoGrants};

use crate::errors::api_error::ApiError;


pub async fn l_create_voice_token(user_key: String, session_key: String) -> Result<String, ApiError> {
    let lk_key = match std::env::var("LIVEKIT_API_KEY") {
        Ok(e) => e,
        Err(_e) => return Err(ApiError::InternalError)
    };
    let lk_secret = match std::env::var("LIVEKIT_API_SECRET") {
        Ok(e) => e,
        Err(_e) => return Err(ApiError::InternalError)
    };

    let token = match AccessToken::with_api_key(&lk_key, &lk_secret)
                                        .with_identity(&user_key)
                                        .with_grants(VideoGrants {
                                            room_join: true,
                                            room: session_key,
                                            can_publish_sources: vec!["microphone".to_string()],
                                            ..Default::default()
                                        })
                                        .to_jwt() {
        Ok(t) => t,
        Err(e) => return Err(ApiError::InternalError) // TODO: not sure about this error
    };

    Ok(token)
}