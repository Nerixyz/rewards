use async_trait::async_trait;
use errors::{json_error::JsonError, sql::SqlReason};
use sqlx::PgPool;
use twitch_pubsub::{TokenProvider, Topic};

use crate::models::config::ConfigEntry;

#[derive(Debug)]
pub struct PubsubTokenProvider(pub PgPool);

impl PubsubTokenProvider {
    async fn get_token(&self) -> Result<String, <Self as TokenProvider>::Error> {
        ConfigEntry::get_user_token(&self.0)
            .await
            .map(|conf| conf.access_token)
    }
}

#[async_trait]
impl TokenProvider for PubsubTokenProvider {
    type Error = JsonError<SqlReason>;

    async fn provide_token(&self, _: &Topic) -> Result<Option<String>, Self::Error> {
        Ok(Some(self.get_token().await?))
    }

    async fn provide_many(
        &self,
        topics: Vec<Topic>,
    ) -> Result<Vec<(Vec<Topic>, Option<String>)>, Self::Error> {
        Ok(vec![(topics, Some(self.get_token().await?))])
    }
}
