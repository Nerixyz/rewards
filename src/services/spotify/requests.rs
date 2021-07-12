use crate::constants::{SPOTIFY_CLIENT_ID, SPOTIFY_CLIENT_SECRET};
use crate::services::spotify::auth::get_redirect_url;
use crate::services::spotify::responses::{
    AccessTokenResponse, PlayerResponse, RefreshTokenResponse, SearchResponse, TrackObject,
};
use anyhow::{Error as AnyError, Result as AnyResult};
use futures::TryFutureExt;
use lazy_static::lazy_static;
use percent_encoding::{AsciiSet, CONTROLS};
use reqwest::Client;
use reqwest::{IntoUrl, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;

lazy_static! {
    static ref SPOTIFY_CLIENT: Client = Client::builder()
        .user_agent(format!(
            "RewardMore/{} github.com/Nerixyz/rewards",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .unwrap();
}

#[derive(Serialize)]
struct TokenRequest<'a> {
    grant_type: &'a str,
    code: &'a str,
    redirect_uri: String,
}

#[derive(Serialize)]
struct RefreshRequest<'a> {
    grant_type: &'a str,
    refresh_token: &'a str,
}

#[derive(Serialize)]
struct QueueTrack<'a> {
    uri: &'a str,
}

#[derive(Serialize)]
struct PlayTracks<'a> {
    uris: &'a [&'a str],
}

#[derive(Serialize)]
struct SearchQuery<'a> {
    q: &'a str,
    r#type: &'a str,
}

pub async fn get_token(code: &str) -> AnyResult<AccessTokenResponse> {
    Ok(SPOTIFY_CLIENT
        .post("https://accounts.spotify.com/api/token")
        .form(&TokenRequest {
            grant_type: "authorization_code",
            code,
            redirect_uri: get_redirect_url(),
        })
        .header(
            "Authorization",
            format!(
                "Basic {}",
                base64::encode(format!("{}:{}", SPOTIFY_CLIENT_ID, SPOTIFY_CLIENT_SECRET))
            ),
        )
        .send()
        .and_then(Response::json)
        .await?)
}
pub async fn refresh_token(refresh_token: &str) -> AnyResult<RefreshTokenResponse> {
    Ok(SPOTIFY_CLIENT
        .post("https://accounts.spotify.com/api/token")
        .form(&RefreshRequest {
            grant_type: "refresh_token",
            refresh_token,
        })
        .header(
            "Authorization",
            format!(
                "Basic {}",
                base64::encode(format!("{}:{}", SPOTIFY_CLIENT_ID, SPOTIFY_CLIENT_SECRET))
            ),
        )
        .send()
        .and_then(Response::json)
        .await?)
}

pub async fn skip_next(auth_token: &str) -> AnyResult<()> {
    post204("https://api.spotify.com/v1/me/player/next", auth_token).await?;
    Ok(())
}

pub async fn queue_item(uri: &str, auth_token: &str) -> AnyResult<()> {
    post204(
        format!(
            "https://api.spotify.com/v1/me/player/queue?{}",
            serde_qs::to_string(&QueueTrack { uri }).unwrap_or_else(|_| String::new())
        ),
        auth_token,
    )
    .await?;
    Ok(())
}

pub async fn play_track(uri: &str, auth_token: &str) -> AnyResult<()> {
    put204(
        "https://api.spotify.com/v1/me/player/play",
        &PlayTracks { uris: &[uri] },
        auth_token,
    )
    .await?;
    Ok(())
}

pub async fn get_player(auth_token: &str) -> AnyResult<PlayerResponse> {
    maybe_get("https://api.spotify.com/v1/me/player", auth_token)
        .await
        .map(|maybe| {
            maybe.unwrap_or(PlayerResponse {
                item: None,
                is_playing: false,
            })
        })
}

const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
pub async fn get_track(track_id: &str, auth_token: &str) -> AnyResult<TrackObject> {
    get(
        format!(
            "https://api.spotify.com/v1/tracks/{}",
            percent_encoding::utf8_percent_encode(track_id, FRAGMENT)
        ),
        auth_token,
    )
    .await
}

pub async fn search_track(q: &str, auth_token: &str) -> AnyResult<SearchResponse> {
    get(
        format!(
            "https://api.spotify.com/v1/search?{}",
            serde_qs::to_string(&SearchQuery { q, r#type: "track" })?
        ),
        auth_token,
    )
    .await
}

async fn post204<U: IntoUrl>(url: U, auth_token: &str) -> AnyResult<()> {
    let response = SPOTIFY_CLIENT
        .post(url)
        .json(&serde_json::Value::Null)
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await?;
    no_content_result(response)
}

async fn put204<U: IntoUrl, T: Serialize>(url: U, body: &T, auth_token: &str) -> AnyResult<()> {
    let response = SPOTIFY_CLIENT
        .put(url)
        .json(body)
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await?;
    no_content_result(response)
}

fn no_content_result(response: Response) -> AnyResult<()> {
    match response.status() {
        StatusCode::NO_CONTENT => Ok(()),
        StatusCode::FORBIDDEN => Err(AnyError::msg("Controlling the player requires Spotify premium :/")),
        x => Err(AnyError::msg(format!(
            "Expected 204 - got {}",
            x
        )))
    }
}

async fn get<U, T>(url: U, auth_token: &str) -> AnyResult<T>
where
    U: IntoUrl,
    T: DeserializeOwned,
{
    Ok(SPOTIFY_CLIENT
        .get(url)
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .and_then(Response::json)
        .await?)
}

async fn maybe_get<U, T>(url: U, auth_token: &str) -> AnyResult<Option<T>>
where
    U: IntoUrl,
    T: DeserializeOwned,
{
    Ok(SPOTIFY_CLIENT
        .get(url)
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .map_err(AnyError::from)
        .and_then(maybe_json)
        .await?)
}

async fn maybe_json<T>(res: Response) -> AnyResult<Option<T>>
where
    T: DeserializeOwned,
{
    match res.status() {
        StatusCode::NO_CONTENT => Ok(None),
        StatusCode::OK => Ok(Some(res.json().await?)),
        StatusCode::FORBIDDEN => Err(AnyError::msg("403 - Seems to be a temporary error by Spotify - maybe premium is required")),
        status => Err(AnyError::msg(format!("Bad status: {}", status))),
    }
}
