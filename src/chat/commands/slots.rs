use crate::{chat::command::ChatCommand, models::slot::Slot};
use anyhow::{Error as AnyError, Result as AnyResult};
use async_trait::async_trait;
use sqlx::PgPool;
use twitch_irc::message::PrivmsgMessage;

pub struct SlotsCommand;

#[async_trait]
impl ChatCommand for SlotsCommand {
    async fn execute(&mut self, msg: PrivmsgMessage, pool: &PgPool) -> AnyResult<String> {
        let occupation = Slot::get_occupation(&msg.channel_id, pool)
            .await
            .map_err(|_| AnyError::msg("Some kind of internal error"))?;
        Ok(format!(
            "@{}, There are {} of {} slots free in this channel",
            msg.sender.login,
            occupation.available.unwrap_or(0),
            occupation.total.unwrap_or(0)
        ))
    }

    fn parse(_args: Option<&str>) -> AnyResult<Box<dyn ChatCommand + Send>>
    where
        Self: Sized + Send,
    {
        Ok(Box::new(Self))
    }
}
