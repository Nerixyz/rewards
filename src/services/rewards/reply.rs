use actix::{Addr, MailboxError};
use anyhow::Result as AnyResult;

use crate::{
    actors::irc::{IrcActor, SayMessage},
    services::rewards::Redemption,
};

pub enum SpotifyAction {
    Skip,
    Play,
    Queue,
}

pub fn format_spotify_result(res: AnyResult<String>, action: SpotifyAction) -> AnyResult<String> {
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
        redemption
            .event
            .broadcaster_user_login
            .clone()
            .into_string(),
        redemption.event.user_login.clone().into_string(),
    )
}

pub async fn reply_to_redemption(
    res: AnyResult<String>,
    irc: &Addr<IrcActor>,
    broadcaster: String,
    user: String,
) -> AnyResult<()> {
    match res {
        Ok(msg) => {
            log_irc_error(
                irc.send(SayMessage(broadcaster, format!("@{} {}", user, msg)))
                    .await,
            );
            // don't return Err() since it will turn the redemption into Cancelled even though it was fulfilled
            Ok(())
        }
        Err(e) => {
            log_irc_error(
                irc.send(SayMessage(broadcaster, format!("@{} ⚠ {}", user, e)))
                    .await,
            );
            Err(e)
        }
    }
}

fn log_irc_error(res: Result<AnyResult<()>, MailboxError>) {
    match res {
        Err(e) => log::error!("could not send: irc mailbox full - {}", e),
        Ok(Err(e)) => log::error!("could not send: {}", e),
        _ => (),
    }
}
