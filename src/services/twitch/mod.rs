use crate::{
    log_err,
    services::twitch::{errors::TwitchApiError, requests::get_users},
    RedisConn,
};
use deadpool_redis::redis;
use itertools::{Either, Itertools};
use twitch_api2::{
    helix::users::User as HelixUser, twitch_oauth2::UserToken, HelixClient,
};
use twitch_api2::types::UserIdRef;

pub mod errors;
pub mod eventsub;
pub mod requests;
mod token;
pub use token::*;

pub type RHelixClient<'a> = HelixClient<'a, reqwest::Client>;
pub type HelixResult<T> = Result<T, TwitchApiError>;

lazy_static::lazy_static! {
    pub static ref CLIENT: reqwest::Client = reqwest::Client::default();
    pub static ref HELIX_CLIENT: RHelixClient<'static> = RHelixClient::default();
}

pub async fn get_many_users(
    ids: Vec<String>,
    token: &UserToken,
    redis: &mut RedisConn,
) -> HelixResult<Vec<HelixUser>> {
    let mut cmd = redis::cmd("MGET");
    for id in ids.iter() {
        cmd.arg(format!("rewards:user:{}", id));
    }
    match cmd
        .query_async::<_, Vec<Option<String>>>(redis)
        .await
        .ok()
        .map(|res| {
            res.into_iter()
                .map(|user| {
                    user.and_then(|user| {
                        serde_json::from_str::<HelixUser>(&user).ok()
                    })
                })
                .collect::<Vec<Option<HelixUser>>>()
        }) {
        Some(mut cached) if cached.len() == ids.len() => {
            let (mut done, pending): (Vec<_>, Vec<_>) =
                ids.into_iter().enumerate().partition_map(|(idx, id)| {
                    match cached.get_mut(idx).and_then(|c| c.take()) {
                        Some(u) => Either::Left(u),
                        None => Either::Right(id),
                    }
                });
            if pending.is_empty() {
                return Ok(done);
            }

            let to_get: Vec<&UserIdRef> = pending.iter().map(|i| i.as_str().into()).collect();
            let mut users = get_users(&to_get, token).await?;
            save_users_to_redis(&users, redis).await?;

            done.append(&mut users);

            Ok(done)
        }
        _ => {
            let to_get: Vec<&UserIdRef> = ids.iter().map(|i| i.as_str().into()).collect();
            let users = get_users(&to_get, token).await?;
            save_users_to_redis(&users, redis).await?;

            Ok(users)
        }
    }
}

async fn save_users_to_redis(
    users: &[HelixUser],
    redis: &mut RedisConn,
) -> HelixResult<()> {
    let mut pipe = redis::pipe();
    for user in users.iter() {
        pipe.cmd("SETEX")
            .arg(format!("rewards:user:{}", user.id))
            .arg(60 * 60)
            .arg(
                serde_json::to_string(user)
                    .map_err(|_| TwitchApiError::Serde)?,
            );
    }
    log_err!(
        pipe.query_async::<_, ()>(redis).await,
        "Couldn't set on redis"
    );
    Ok(())
}
