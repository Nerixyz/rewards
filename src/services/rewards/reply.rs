use anyhow::Result as AnyResult;

use crate::{
    log_err,
    services::{
        rewards::Redemption,
        twitch::{self, requests::send_chat_message},
    },
};

pub enum SpotifyAction {
    Skip,
    Play,
    Queue,
}

pub fn format_spotify_result(
    res: AnyResult<String>,
    action: SpotifyAction,
) -> AnyResult<String> {
    res.map(|msg| {
        format!(
            "{} {}",
            match action {
                SpotifyAction::Skip => "⏭ Skipped",
                SpotifyAction::Play => "▶ Playing",
                SpotifyAction::Queue => "🗒 Queued",
            },
            msg
        )
    })
}

pub fn get_reply_data(redemption: &Redemption) -> (String, String) {
    (
        redemption.broadcaster_user_id.clone().take(),
        redemption.user_login.clone().take(),
    )
}

pub async fn reply_to_redemption(
    res: AnyResult<Option<String>>,
    broadcaster_id: &str,
    user: &str,
) -> AnyResult<()> {
    match res {
        Ok(Some(msg)) => {
            log_err!(
                send_chat_message(broadcaster_id, &msg, &twitch::get_token())
                    .await,
                "Failed to send chat"
            );
            // don't return Err() since it will turn the redemption into Cancelled even though it was fulfilled
            Ok(())
        }
        Ok(None) => Ok(()),
        Err(e) => {
            log_err!(
                send_chat_message(
                    broadcaster_id,
                    &format!("@{} [⚠ Refund] {}", user, e),
                    &twitch::get_token(),
                )
                .await,
                "Failed to send chat"
            );
            Err(e)
        }
    }
}
