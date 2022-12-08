use std::fmt::{Debug, Formatter};

use actix::Addr;
use anyhow::Error as AnyError;
use async_trait::async_trait;
use twitch_irc::login::{TokenStorage, UserAccessToken};

use crate::actors::db::{DbActor, GetToken, SaveToken};

pub struct PgTokenStorage(pub Addr<DbActor>);

impl Debug for PgTokenStorage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("PgTokenStorage")
    }
}

#[async_trait]
impl TokenStorage for PgTokenStorage {
    type LoadError = AnyError;
    type UpdateError = AnyError;

    async fn load_token(&mut self) -> Result<UserAccessToken, Self::LoadError> {
        self.0.send(GetToken {}).await?.map_err(AnyError::new)
    }

    async fn update_token(
        &mut self,
        token: &UserAccessToken,
    ) -> Result<(), Self::UpdateError> {
        log::info!("Token updated");
        self.0
            .send(SaveToken(UserAccessToken {
                refresh_token: token.refresh_token.clone(),
                access_token: token.access_token.clone(),
                created_at: token.created_at.clone(),
                expires_at: token.expires_at.clone(),
            }))
            .await?
            .map_err(AnyError::new)
    }
}
