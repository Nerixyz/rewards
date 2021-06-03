use std::sync::Arc;

use actix::Addr;
use anyhow::Result as AnyResult;

use crate::actors::irc_actor::IrcActor;
use crate::actors::messages::irc_messages::SayMessage;

pub async fn send_emote_reply(
    data: AnyResult<(Option<String>, String)>,
    irc: &Arc<Addr<IrcActor>>,
    broadcaster: String,
    user: String,
) -> AnyResult<()> {
    match data {
        Ok((Some(removed), added)) => {
            irc.send(SayMessage(
                broadcaster,
                format!("@{} ☑ Added {} - 🗑 Removed {}", user, added, removed),
            ))
            .await??;
        }
        Ok((None, added)) => {
            irc.send(SayMessage(
                broadcaster,
                format!("@{} ☑ Added {}", user, added),
            ))
            .await??;
        }
        Err(e) => {
            irc.send(SayMessage(broadcaster, format!("@{} ⚠ {}", user, e)))
                .await??;

            return Err(e);
        }
    };
    Ok(())
}

pub async fn send_slot_reply(
    data: AnyResult<(String, usize)>,
    irc: &Arc<Addr<IrcActor>>,
    broadcaster: String,
    user: String,
) -> AnyResult<()> {
    match data {
        Ok((added, remaining)) if remaining > 1 => {
            irc.send(SayMessage(
                broadcaster,
                format!("@{} ☑ Added {} - 🔳 {} slots open", user, added, remaining),
            ))
            .await??;
        }
        Ok((added, remaining)) if remaining == 1 => {
            irc.send(SayMessage(
                broadcaster,
                format!("@{} ☑ Added {} - 🔳 {} slot open", user, added, remaining),
            ))
            .await??;
        }
        Ok((added, _)) => {
            irc.send(SayMessage(
                broadcaster,
                format!("@{} ☑ Added {} - 0 slots open - 🔒 closing", user, added),
            ))
            .await??;
        }
        Err(e) => {
            irc.send(SayMessage(broadcaster, format!("@{} ⚠ {}", user, e)))
                .await??;

            return Err(e);
        }
    };
    Ok(())
}

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
