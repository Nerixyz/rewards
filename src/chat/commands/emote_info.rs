use crate::chat::command::ChatCommand;
use crate::chat::parse::opt_next_space;
use crate::models::slot::Slot;
use crate::services::formatting::human_format_duration;
use anyhow::{Error as AnyError, Result as AnyResult};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use twitch_irc::message::PrivmsgMessage;

pub struct EmoteInfo(String);

#[async_trait]
impl ChatCommand for EmoteInfo {
    async fn execute(&mut self, msg: PrivmsgMessage, pool: &PgPool) -> AnyResult<String> {
        let slot = Slot::get_slot_by_emote_name(&self.0, pool)
            .await
            .map_err(|_| AnyError::msg("Internal Error"))?
            .ok_or_else(|| AnyError::msg("No such emote"))?;
        match (slot.name, slot.added_at, slot.added_by, slot.expires) {
            (Some(name), Some(added_at), Some(added_by), Some(expired)) => {
                let now = Utc::now();
                let added_duration = now - added_at;
                let expired_duration = now - expired;
                Ok(format!(
                    "@{}, {} was added {} by @{} and will be removed {}",
                    msg.sender.login,
                    name,
                    human_format_duration(&added_duration),
                    added_by,
                    human_format_duration(&expired_duration)
                ))
            }
            _ => Err(AnyError::msg("Not enough information")),
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
