use std::sync::Arc;

use actix::Addr;
use anyhow::Result as AnyResult;

use crate::actors::{irc_actor::IrcActor, messages::irc_messages::SayMessage};

pub enum SpotifyAction {
    Skip,
    Play,
    Queue,
}
pub async fn send_spotify_reply(
    action: SpotifyAction,
    data: AnyResult<String>,
    irc: &Arc<Addr<IrcActor>>,
    broadcaster: String,
    user: String,
) -> AnyResult<()> {
    match data {
        Ok(item_name) => {
            let action = match action {
                SpotifyAction::Skip => "⏭ Skipped",
                SpotifyAction::Play => "▶ Playing",
                SpotifyAction::Queue => "🗒 Queued",
            };
            irc.send(SayMessage(
                broadcaster,
                format!("@{} {} {}", user, action, item_name),
            ))
            .await??
        }
        Err(e) => {
            irc.send(SayMessage(broadcaster, format!("@{} ⚠ {}", user, e)))
                .await??;

            return Err(e);
        }
    };

    Ok(())
}
