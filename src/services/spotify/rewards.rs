use crate::models::reward::SpotifyPlayOptions;
use crate::models::spotify::SpotifyData;
use crate::services::spotify::requests;
use crate::services::spotify::responses::{PlayerResponse, TrackObject};
use anyhow::{Error as AnyError, Result as AnyResult};
use lazy_static::lazy_static;
use regex::Regex;
use sqlx::PgPool;

pub async fn get_spotify_token(user_id: &str, pool: &PgPool) -> AnyResult<String> {
    SpotifyData::get_by_id(user_id, pool)
        .await?
        .ok_or_else(|| AnyError::msg("No spotify connection"))
        .map(|s| s.access_token)
}

pub async fn skip_track(user_id: &str, pool: &PgPool) -> AnyResult<String> {
    let token = get_spotify_token(user_id, pool).await?;
    let player = get_playing_player(&token).await?;

    requests::skip_next(&token).await.map_err(|e| {
        log::warn!("Could not skip: {}", e);
        AnyError::msg("Couldn't skip")
    })?;

    Ok(player
        .item
        .map(|i| i.to_string())
        .unwrap_or_else(|| "?".to_string()))
}

pub async fn queue_track(user_id: &str, track: TrackObject, pool: &PgPool) -> AnyResult<String> {
    let token = get_spotify_token(user_id, pool).await?;

    get_playing_player(&token).await?;

    requests::queue_item(&track.uri, &token)
        .await
        .map_err(|e| {
            log::warn!("Could not queue: {}", e);
            AnyError::msg("Couldn't queue")
        })?;

    Ok(track.to_string())
}

pub async fn play_track(user_id: &str, track: TrackObject, pool: &PgPool) -> AnyResult<String> {
    let token = get_spotify_token(user_id, pool).await?;

    get_playing_player(&token).await?;

    requests::play_track(&track.uri, &token)
        .await
        .map_err(|e| {
            log::warn!("Could not queue: {}", e);
            AnyError::msg("Couldn't play")
        })?;

    Ok(track.to_string())
}

pub async fn get_track_uri_from_input(
    input: &str,
    broadcaster_id: &str,
    options: &SpotifyPlayOptions,
    pool: &PgPool,
) -> AnyResult<TrackObject> {
    let token = get_spotify_token(broadcaster_id, pool).await?;
    if let Some(id) = extract_spotify_id(input) {
        let track = requests::get_track(id, &token)
            .await
            .map_err(|_| AnyError::msg("Could not find your track"))?;

        if track.explicit && !options.allow_explicit {
            return Err(AnyError::msg("Explicit tracks are disallowed!"));
        }
        Ok(track)
    } else {
        let tracks = requests::search_track(input, &token).await?;
        tracks
            .tracks
            .map(|tracks| {
                tracks
                    .items
                    .into_iter()
                    .find(|track| options.allow_explicit || !track.explicit)
            })
            .flatten()
            .ok_or_else(|| AnyError::msg("No track found"))
    }
}
fn extract_spotify_id(str: &str) -> Option<&str> {
    lazy_static! {
        static ref SPOTIFY_REGEX: Regex =
            Regex::new("\\b([A-Za-z0-9]{22,})\\b").expect("must compile");
    }
    SPOTIFY_REGEX
        .captures(str)
        .map(|c| c.iter().nth(1).flatten().map(|m| m.as_str()))
        .flatten()
}

async fn get_playing_player(token: &str) -> AnyResult<PlayerResponse> {
    let player = requests::get_player(&token).await.map_err(|e| {
        log::warn!("Could not get player: {}", e);
        AnyError::msg("Internal Error")
    })?;
    if !player.is_playing {
        return Err(AnyError::msg("There's no song playing"));
    }
    Ok(player)
}