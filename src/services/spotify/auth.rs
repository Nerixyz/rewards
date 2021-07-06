use crate::constants::{SERVER_URL, SPOTIFY_CLIENT_ID};
use crate::services::jwt::{encode_jwt, JwtClaims};
use anyhow::Result as AnyResult;
use serde::{Deserialize, Serialize};

pub const SPOTIFY_SCOPES: &str =
    "user-read-playback-state user-modify-playback-state user-read-currently-playing streaming";

#[derive(Deserialize)]
#[serde(untagged)]
pub enum SpotifyAuthResponse {
    Success { code: String, state: String },
    Error { error: String, state: String },
}

#[derive(Serialize)]
struct AuthUriQuery<'a> {
    client_id: &'a str,
    response_type: &'a str,
    redirect_uri: String,
    scope: &'a str,
    state: &'a str,
}

pub fn get_auth_url(user_id: String) -> AnyResult<(String, String)> {
    let jwt = encode_jwt(&JwtClaims::new_short(user_id))?;
    let query = serde_qs::to_string(&AuthUriQuery {
        client_id: SPOTIFY_CLIENT_ID,
        response_type: "code",
        redirect_uri: get_redirect_url(),
        scope: SPOTIFY_SCOPES,
        state: &jwt,
    })?;
    Ok((
        format!("https://accounts.spotify.com/authorize?{}", query),
        jwt,
    ))
}

pub fn get_redirect_url() -> String {
    format!("{}/api/v1/connections/spotify-callback", SERVER_URL)
}
