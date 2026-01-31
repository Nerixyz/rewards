use crate::services::spotify::{requests, rewards::get_token_and_verify};
use anyhow::Result as AnyResult;
use sqlx::PgPool;
use std::fmt::Write;
use twitch_irc::message::PrivmsgMessage;

pub async fn execute(msg: PrivmsgMessage, pool: &PgPool) -> AnyResult<String> {
    let token = get_token_and_verify(&msg.channel_id, true, pool).await?;
    let player = requests::get_player(&token).await?;
    let mut msg = String::new();
    if let Some(ref item) = player.item.as_ref().filter(|_| player.is_playing) {
        write!(&mut msg, "{item} ({})", item.spotify_url())?;
    } else {
        write!(&mut msg, "No song is playing")?;
    }

    // should this print the queue?
    // if so, both the player and queue have to be requested

    Ok(msg)
}
