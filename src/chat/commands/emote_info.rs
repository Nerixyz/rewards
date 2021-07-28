use crate::{
    chat::{command::ChatCommand, parse::opt_next_space},
    models::{slot::Slot, swap_emote::SwapEmote},
    services::formatting::human_format_duration,
};
use anyhow::{Error as AnyError, Result as AnyResult};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use twitch_irc::message::PrivmsgMessage;

pub struct EmoteInfo(String);

#[async_trait]
impl ChatCommand for EmoteInfo {
    async fn execute(&mut self, msg: PrivmsgMessage, pool: &PgPool) -> AnyResult<String> {
        let slot = Slot::get_slot_by_emote_name(&msg.channel_id, &self.0, pool)
            .await
            .map_err(|_| AnyError::msg("Internal Error"))?;
        match slot {
            None => {
                let emote = SwapEmote::by_name(&msg.channel_id, &self.0, pool)
                    .await
                    .map_err(|_| AnyError::msg("Internal error"))?
                    .ok_or_else(|| AnyError::msg("No such emote"))?;

                format_swap(&msg.sender.login, emote)
            }
            Some(slot) => format_slot(&msg.sender.login, slot),
        }
    }

    fn parse(args: Option<&str>) -> AnyResult<Box<dyn ChatCommand + Send>>
    where
        Self: Sized + Send,
    {
        let args = args.ok_or_else(|| AnyError::msg("No emote specified"))?;
        let (user, _) = opt_next_space(args);
        Ok(Box::new(Self(user.to_string())))
    }
}

fn format_slot(sender: &str, slot: Slot) -> AnyResult<String> {
    match (slot.name, slot.added_at, slot.added_by, slot.expires) {
        (Some(name), Some(added_at), Some(added_by), Some(expired)) => {
            let now = Utc::now();
            let added_duration = now - added_at;
            let expired_duration = now - expired;
            Ok(format!(
                "@{}, {} was added {} by @{} and will be removed {}",
                sender,
                name,
                human_format_duration(&added_duration),
                added_by,
                human_format_duration(&expired_duration)
            ))
        }
        _ => Err(AnyError::msg("Not enough information")),
    }
}

fn format_swap(sender: &str, emote: SwapEmote) -> AnyResult<String> {
    let now = Utc::now();
    let added_duration = now - emote.added_at;
    Ok(format!(
        "@{}, {} was added {} by @{} [{}]",
        sender,
        emote.name,
        human_format_duration(&added_duration),
        emote.added_by,
        emote.platform
    ))
}
