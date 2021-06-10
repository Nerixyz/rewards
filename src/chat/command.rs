use async_trait::async_trait;
use twitch_irc::message::PrivmsgMessage;
use sqlx::PgPool;
use anyhow::{Result as AnyResult};

#[async_trait]
pub trait ChatCommand {
    async fn execute(&mut self, msg: PrivmsgMessage, pool: &PgPool) -> AnyResult<String>;
    fn parse(args: Option<&str>) -> AnyResult<Self> where Self: Sized + Send;
}