use crate::{
    log_err,
    services::{
        emotes,
        jwt::JwtClaims,
        sql::get_user_or_editor,
        twitch::requests::{get_user, get_user_by_login},
    },
    RedisConn, RedisPool,
};
use actix_web::{get, web, HttpResponse, Result};
use deadpool_redis::redis::AsyncCommands;
use serde::Serialize;
use sqlx::PgPool;
use twitch_api::twitch_oauth2::UserToken;

#[get("/me")]
async fn me(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse> {
    let mut redis_conn = redis
        .get()
        .await
        .map_err(|_| errors::ErrorInternalServerError("Redis is dead"))?;

    if let Ok(user) = redis_conn
        .get::<_, String>(format!("rewards:user:{}", claims.user_id()))
        .await
    {
        return Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(user));
    }

    let user = claims.get_user(&pool).await?;
    let data = get_user(claims.user_id().to_string(), &user.into()).await?;

    log_err!(
        redis_conn
            .set_ex::<_, _, ()>(
                format!("rewards:user:{}", claims.user_id()),
                serde_json::to_string(&data)?,
                60 * 60
            )
            .await,
        "Couldn't set on redis"
    );

    Ok(HttpResponse::Ok().json(&data))
}

#[get("/{user_login}")]
async fn info(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    login: web::Path<String>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse> {
    let mut redis_conn = redis
        .get()
        .await
        .map_err(|_| errors::ErrorInternalServerError("Redis is dead"))?;

    let login = login.into_inner();
    let key = format!("rewards:user:login:{}", login);
    if let Ok(user) = redis_conn.get::<_, String>(&key).await {
        return Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(user));
    }

    let user = claims.get_user(&pool).await?;
    let data = get_user_by_login::<UserToken>(login, &user.into()).await?;

    log_err!(
        redis_conn
            .set_ex::<_, _, ()>(key, serde_json::to_string(&data)?, 60 * 60)
            .await,
        "Couldn't set on redis"
    );

    Ok(HttpResponse::Ok().json(&data))
}

#[derive(Serialize)]
struct RefreshEmotesResult {
    n_removed: usize,
}

#[get("/{broadcaster_id}/refresh-emotes")]
async fn refresh_emotes(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let broadcaster_id = path.into_inner();
    get_user_or_editor(&claims, &broadcaster_id, &pool).await?;

    let mut redis_conn = redis
        .get()
        .await
        .map_err(|_| errors::ErrorInternalServerError("Redis is dead"))?;

    if !check_update_refresh_cooldown(&mut redis_conn, &broadcaster_id)
        .await
        .map_err(|e| {
            errors::ErrorInternalServerError(format!(
                "Failed to check cooldown: {e}"
            ))
        })?
    {
        return Err(errors::ErrorTooManyRequests(
            "You are refreshing too often",
        ));
    }

    let n_removed = emotes::refresh::refresh_emotes(
        &broadcaster_id,
        &mut redis_conn,
        &pool,
    )
    .await
    .map_err(|e| {
        errors::ErrorInternalServerError(format!("Failed to refresh: {e}"))
    })?;
    Ok(HttpResponse::Ok().json(RefreshEmotesResult { n_removed }))
}

async fn check_update_refresh_cooldown(
    conn: &mut RedisConn,
    channel: &str,
) -> anyhow::Result<bool> {
    let key = format!("rewards:refresh-cooldown:{}", channel);
    let existing: i8 = conn.exists(&key).await?;
    if existing != 0 {
        return Ok(false);
    }
    conn.set_ex::<_, _, ()>(&key, 1, 10).await?;
    Ok(true)
}

pub fn init_user_routes(config: &mut web::ServiceConfig) {
    config.service(me).service(info).service(refresh_emotes);
}
