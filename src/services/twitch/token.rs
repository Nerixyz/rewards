use crate::{ClientId, Scope};
use config::CONFIG;
use once_cell::sync::OnceCell;
use std::{sync::Arc, time::Duration};
use twitch_api::{
    twitch_oauth2::{
        client::Client,
        tokens::{errors::RefreshTokenError, BearerTokenType},
        AccessToken, RefreshToken, TwitchToken,
    },
    types::{UserIdRef, UserNameRef},
};

static TOKEN: OnceCell<std::sync::Mutex<DbToken>> = OnceCell::new();

#[derive(Debug)]
pub struct TokenData {
    pub access_token: AccessToken,
    pub refresh_token: RefreshToken,
    pub client_id: ClientId,
}

#[derive(Debug)]
pub struct DbToken(Arc<TokenData>);

pub fn update_token(
    access_token: impl Into<AccessToken>,
    refresh_token: impl Into<RefreshToken>,
) {
    always_lock(TOKEN.get().expect("must have set token")).0 =
        Arc::new(TokenData {
            access_token: access_token.into(),
            refresh_token: refresh_token.into(),
            client_id: CONFIG.twitch.client_id.to_owned().into(),
        });
}

pub fn set_token(t: twitch_irc::login::UserAccessToken) {
    TOKEN
        .set(std::sync::Mutex::new(DbToken(Arc::new(t.into()))))
        .expect("must not have been set");
}

pub fn get_token() -> DbToken {
    always_lock(TOKEN.get().expect("must have set token")).clone()
}

impl Clone for DbToken {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl DbToken {
    pub fn data(&self) -> &TokenData {
        self.0.as_ref()
    }
}

impl From<twitch_irc::login::UserAccessToken> for TokenData {
    fn from(t: twitch_irc::login::UserAccessToken) -> Self {
        Self {
            access_token: t.access_token.into(),
            refresh_token: t.refresh_token.into(),
            client_id: CONFIG.twitch.client_id.to_owned().into(),
        }
    }
}

#[async_trait::async_trait]
impl TwitchToken for DbToken {
    fn token_type() -> BearerTokenType {
        BearerTokenType::UserToken
    }

    fn client_id(&self) -> &ClientId {
        &self.0.client_id
    }

    fn token(&self) -> &AccessToken {
        &self.0.access_token
    }

    fn login(&self) -> Option<&UserNameRef> {
        None
    }

    fn user_id(&self) -> Option<&UserIdRef> {
        None
    }

    async fn refresh_token<'a, C>(
        &mut self,
        _: &'a C,
    ) -> Result<(), RefreshTokenError<<C as Client>::Error>>
    where
        Self: Sized,
        C: Client,
    {
        Ok(())
    }

    fn expires_in(&self) -> Duration {
        Duration::from_secs(u64::MAX)
    }

    fn scopes(&self) -> &[Scope] {
        &[]
    }
}

fn always_lock<T>(m: &std::sync::Mutex<T>) -> std::sync::MutexGuard<T> {
    match m.lock() {
        Ok(v) => v,
        Err(e) => e.into_inner(),
    }
}
