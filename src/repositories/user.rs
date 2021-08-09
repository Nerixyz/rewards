use crate::{
    log_err,
    services::{
        jwt::JwtClaims,
        twitch::requests::{get_user, get_user_by_login},
    },
    RedisPool,
};
use actix_web::{get, web, HttpResponse, Result};
use deadpool_redis::redis::AsyncCommands;
use sqlx::PgPool;
use twitch_api2::twitch_oauth2::UserToken;

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

pub fn init_user_routes(config: &mut web::ServiceConfig) {
    config.service(me).service(info);
}
