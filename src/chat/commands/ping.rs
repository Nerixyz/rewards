use crate::chat::command::ChatCommand;
use sqlx::PgPool;
use anyhow::Result as AnyResult;
use twitch_irc::message::PrivmsgMessage;
use async_trait::async_trait;

pub struct Ping;

#[async_trait]
impl ChatCommand for Ping {
    async fn execute(&mut self, msg: PrivmsgMessage, _pool: &PgPool) -> AnyResult<String> {
        Ok(format!("@{}, Pong!", msg.sender.login))
    }

    fn parse(_msg: Option<&str>) -> AnyResult<Self> {
        Ok(Self)
    }
}