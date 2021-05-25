use crate::models::user::User;
use actix::{Actor, Context};
use actix_web::rt::{self, task::JoinHandle};
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
                let users = User::get_all(&pool).await.unwrap_or(vec![]);
                for user in users {
                    let mut token: UserToken = user.into();
                    if let Ok(_) = token.refresh_token(reqwest_http_client).await {
                        let res = User::update_refresh(
                            &token.user_id,
                            token.access_token.secret(),
                            &token
                                .refresh_token
                                .map(|t| t.secret().clone())
                                .unwrap_or(String::new()),
                            &pool,
                        )
                        .await;
                        if let Err(e) = res {
                            println!("{:?}", e);
                        }
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                tokio::time::sleep(Duration::from_secs(40 * 60)).await;
            }
        }));
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        todo!()
    }
}
