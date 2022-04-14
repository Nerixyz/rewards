use crate::{AppAccessToken, RedisConn};
use anyhow::Result as AnyResult;
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use twitch_irc::message::PrivmsgMessage;

#[async_trait]
pub trait ChatCommand: Send {
    async fn execute(
        &mut self,
        msg: PrivmsgMessage,
        pool: &PgPool,
        redis: &mut RedisConn,
        app_access_token: Arc<RwLock<AppAccessToken>>,
    ) -> AnyResult<String>;
    fn parse(cmd: &str, args: Option<&str>) -> AnyResult<Box<dyn ChatCommand + Send>>
    where
        Self: Sized + Send;

    async fn check_permission(
        &mut self,
        _msg: &PrivmsgMessage,
        _pool: &PgPool,
        _redis: &mut RedisConn,
    ) -> bool {
        true
    }
}
