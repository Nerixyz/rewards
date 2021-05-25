use crate::actors::messages::db_messages::{GetToken, SaveToken};
use crate::models::config::ConfigEntry;
use crate::services::sql::SqlError;
use actix::{Actor, Context, Handler, ResponseFuture};
use sqlx::PgPool;
use twitch_irc::login::UserAccessToken;

pub struct DbActor {
    pool: PgPool,
}

impl DbActor {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Actor for DbActor {
    type Context = Context<Self>;
}

impl Handler<GetToken> for DbActor {
    type Result = ResponseFuture<Result<UserAccessToken, SqlError>>;

    fn handle(&mut self, _msg: GetToken, _ctx: &mut Self::Context) -> Self::Result {
        let pool = self.pool.clone();
        Box::pin(async move { ConfigEntry::get_user_token(&pool).await })
    }
}

impl Handler<SaveToken> for DbActor {
    type Result = ResponseFuture<Result<(), SqlError>>;

    fn handle(&mut self, msg: SaveToken, _ctx: &mut Self::Context) -> Self::Result {
        let pool = self.pool.clone();
        Box::pin(async move { ConfigEntry::update_user_token(&pool, msg.0).await })
    }
}
