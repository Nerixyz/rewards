use errors::{json_error::JsonError, sql::SqlReason};
use sqlx::PgPool;
use twitch_pubsub::{TokenProvider, Topic};

use models::config::ConfigEntry;

#[derive(Debug)]
pub struct PubsubTokenProvider(pub PgPool);

impl PubsubTokenProvider {
    async fn get_token(
        &self,
    ) -> Result<String, <Self as TokenProvider>::Error> {
        ConfigEntry::get_user_token(&self.0)
            .await
            .map(|conf| conf.access_token)
    }
}

impl TokenProvider for PubsubTokenProvider {
    type Error = JsonError<SqlReason>;

    async fn provide_token(
        &self,
        _: &Topic,
    ) -> Result<Option<String>, Self::Error> {
        Ok(Some(self.get_token().await?))
    }

    async fn provide_many(
        &self,
        topics: Vec<Topic>,
    ) -> Result<Vec<(Vec<Topic>, Option<String>)>, Self::Error> {
        // here, we need to split up topics that don't need a token and topics that do
        // that's because topics that need one may error and we want to catch each error separately

        let (required, no_token): (Vec<Topic>, Vec<Topic>) =
            topics.into_iter().partition(|t| match t {
                Topic::AutoModQueue(_)
                | Topic::ChatModeratorActions(_)
                | Topic::UserModerationNotifications(_) => true,
                Topic::VideoPlayback(_) | Topic::VideoPlaybackById(_) => false,
                _ => {
                    log::error!(
                        "Invalid topic to be partitioned, providing token though: {:?}",
                        t
                    );
                    true
                }
            });

        if required.is_empty() {
            Ok(vec![(no_token, None)])
        } else {
            let token = self.get_token().await?;

            let mut provided: Vec<(Vec<Topic>, Option<String>)> = required
                .into_iter()
                .map(|topic| (vec![topic], Some(token.clone())))
                .collect();

            if !no_token.is_empty() {
                provided.push((no_token, None));
            }

            Ok(provided)
        }
    }
}
