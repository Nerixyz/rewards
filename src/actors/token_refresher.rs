use crate::log_err;
use crate::models::spotify::SpotifyData;
use crate::models::user::User;
use crate::services::spotify::requests::refresh_token;
use actix::{Actor, Context};
use actix_web::rt::{self, task::JoinHandle};
use anyhow::Result as AnyResult;
use sqlx::PgPool;
use std::time::Duration;
use twitch_api2::twitch_oauth2::client::reqwest_http_client;
use twitch_api2::twitch_oauth2::{TwitchToken, UserToken};

pub struct TokenRefresher {
    pool: PgPool,
    join_handle: Option<JoinHandle<()>>,
}

impl TokenRefresher {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            join_handle: None,
        }
    }
}

impl Actor for TokenRefresher {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        let pool = self.pool.clone();

        self.join_handle = Some(rt::spawn(async move {
            loop {
                log_err!(
                    refresh_twitch_users(&pool).await,
                    "Failed to refresh twitch users"
                );
                log_err!(
                    refresh_spotify_tokens(&pool).await,
                    "Failed to refresh spotify tokens"
                );

                tokio::time::sleep(Duration::from_secs(40 * 60)).await;
            }
        }));
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        if let Some(handle) = &self.join_handle {
            handle.abort();
        }
    }
}

async fn refresh_twitch_users(pool: &PgPool) -> AnyResult<()> {
    let users = User::get_all(&pool).await.unwrap_or_default();
    for user in users {
        let mut token: UserToken = user.into();
        if token.refresh_token(reqwest_http_client).await.is_ok() {
            log_err!(
                User::update_refresh(
                    &token.user_id,
                    token.access_token.secret(),
                    &token
                        .refresh_token
                        .map(|t| t.secret().clone())
                        .unwrap_or_default(),
                    &pool,
                )
                .await,
                "Failed to insert"
            );
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

async fn refresh_spotify_tokens(pool: &PgPool) -> AnyResult<()> {
    let users = SpotifyData::get_all(&pool).await.unwrap_or_default();

    for user in users {
        match refresh_token(&user.refresh_token).await {
            Ok(res) => {
                log_err!(
                    SpotifyData::update_token(&user.user_id, &res, pool).await,
                    "Failed to insert"
                );
            }
            Err(e) => log::warn!("Could not refresh token: {}", e),
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}
