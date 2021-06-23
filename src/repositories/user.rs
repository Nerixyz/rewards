use crate::services::jwt::JwtClaims;
use crate::services::twitch::requests::{get_user, get_user_by_login};
use actix_web::{get, web, HttpResponse, Result};
use sqlx::PgPool;
use twitch_api2::twitch_oauth2::UserToken;

#[get("/me")]
async fn me(claims: JwtClaims, pool: web::Data<PgPool>) -> Result<HttpResponse> {
    let user = claims.get_user(&pool).await?;
    let data = get_user(claims.user_id().to_string(), &user.into()).await?;

    Ok(HttpResponse::Ok().json(&data))
}

#[get("/{user_login}")]
async fn info(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    login: web::Path<String>,
) -> Result<HttpResponse> {
    let user = claims.get_user(&pool).await?;
    let data = get_user_by_login::<UserToken>(login.into_inner(), &user.into()).await?;

    Ok(HttpResponse::Ok().json(&data))
}

pub fn init_user_routes(config: &mut web::ServiceConfig) {
    config.service(me).service(info);
}
