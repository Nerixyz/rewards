use crate::{chat::command::ChatCommand, models::slot::Slot};
use anyhow::{Error as AnyError, Result as AnyResult};
use async_trait::async_trait;
use itertools::Itertools;
use sqlx::PgPool;
use twitch_irc::message::PrivmsgMessage;

pub struct Emotes;

#[async_trait]
impl ChatCommand for Emotes {
    async fn execute(&mut self, msg: PrivmsgMessage, pool: &PgPool) -> AnyResult<String> {
        let occupied = Slot::get_occupied_emotes(&msg.channel_id, pool)
            .await
            .map_err(|_| AnyError::msg("Internal error"))?;
        Ok(if occupied.is_empty() {
            format!(
                "@{}, there are no occupied emote-slots in this channel!",
                msg.sender.login
            )
        } else {
            format!(
                "@{}, these are the current emotes: {}",
                msg.sender.login,
                occupied.iter().join(" ")
            )
        })
    }

    fn parse(_args: Option<&str>) -> AnyResult<Box<dyn ChatCommand + Send>>
    where
        Self: Sized + Send,
    {
        Ok(Box::new(Self))
    }
}
