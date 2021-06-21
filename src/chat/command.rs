use anyhow::Result as AnyResult;
use async_trait::async_trait;
use sqlx::PgPool;
use twitch_irc::message::PrivmsgMessage;

#[async_trait]
pub trait ChatCommand: Send {
    async fn execute(&mut self, msg: PrivmsgMessage, pool: &PgPool) -> AnyResult<String>;
    fn parse(args: Option<&str>) -> AnyResult<Box<dyn ChatCommand + Send>>
    where
        Self: Sized + Send;
}
