use crate::{
    actors::db::{DbActor, SaveToken},
    log_err,
    services::{spotify::requests::refresh_token, twitch},
};
use actix::{Actor, Addr, AsyncContext, Context, WrapFuture};
use anyhow::Result as AnyResult;
use chrono::Utc;
use config::CONFIG;
use models::{spotify::SpotifyData, user::User};
use sqlx::PgPool;
use std::time::Duration;
use twitch_api2::twitch_oauth2::{ClientId, ClientSecret, TwitchToken, UserToken};
use twitch_irc::login::UserAccessToken;

pub struct TokenRefresher {
    pool: PgPool,
    db: Addr<DbActor>,
}

impl TokenRefresher {
    pub fn new(pool: PgPool, db: Addr<DbActor>) -> Self {
        Self { pool, db }
    }

    async fn refresh(pool: &PgPool, db: &Addr<DbActor>) {
        log_err!(
            refresh_twitch_users(pool).await,
            "Failed to refresh twitch users"
        );
        log_err!(
            refresh_spotify_tokens(pool).await,
            "Failed to refresh spotify tokens"
        );
        log_err!(refresh_bot_token(db).await, "Failed to refresh bot token");
    }
}

impl Actor for TokenRefresher {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(40 * 60), |this, ctx| {
            let pool = this.pool.clone();
            let db = this.db.clone();
            ctx.spawn(
                async move { Self::refresh(&pool, &db).await }.into_actor(this),
            );
        });
        let pool = self.pool.clone();
        let db = self.db.clone();
        ctx.spawn(
            async move { Self::refresh(&pool, &db).await }.into_actor(self),
        );
    }
}

async fn refresh_twitch_users(pool: &PgPool) -> AnyResult<()> {
    let users = User::get_all(pool).await.unwrap_or_default();
    for user in users {
        let mut token: UserToken = user.into();
        if token.refresh_token(&*twitch::CLIENT).await.is_ok() {
            log_err!(
                User::update_refresh(
                    &token.user_id,
                    token.access_token.secret(),
                    token
                        .refresh_token
                        .as_ref()
                        .map(|t| t.as_str())
                        .unwrap_or(""),
                    pool,
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
    let users = SpotifyData::get_all(pool).await.unwrap_or_default();

    for user in users {
        match refresh_token(&user.refresh_token).await {
            Ok(res) => {
                log_err!(
                    SpotifyData::update_token(
                        &user.user_id,
                        &res.access_token,
                        pool
                    )
                    .await,
                    "Failed to insert"
                );
            }
            Err(e) => log::warn!("Could not refresh token: {}", e),
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

async fn refresh_bot_token(db_actor: &Addr<DbActor>) -> AnyResult<()> {
    let db = twitch::get_token();
    let mut token = UserToken::from_existing_unchecked(
        db.data().access_token.clone(),
        Some(db.data().refresh_token.clone()),
        ClientId::new(CONFIG.twitch.client_id.clone()),
        ClientSecret::new(CONFIG.twitch.client_secret.clone()),
        "".to_owned().into(),
        "".to_owned().into(),
        None,
        None,
    );
    if token.refresh_token(&*twitch::CLIENT).await.is_ok() {
        let exp = token.expires_in();
        if let Some(r) = token.refresh_token {
            db_actor
                .send(SaveToken(UserAccessToken {
                    refresh_token: r.take(),
                    access_token: token.access_token.take(),
                    created_at: Utc::now(),
                    expires_at: Some(
                        Utc::now()
                            + chrono::Duration::from_std(exp)
                                .unwrap_or(chrono::Duration::zero()),
                    ),
                }))
                .await??;
        }
    }
    Ok(())
}
