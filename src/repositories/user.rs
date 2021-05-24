use actix_web::{web, HttpResponse, Error, get};
use crate::services::jwt::JwtClaims;
use sqlx::PgPool;
use crate::services::twitch::requests::{get_user, get_user_by_login};

#[get("/me")]
async fn me(claims: JwtClaims, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    let user = claims.get_user(&pool).await?;
    let data = get_user(claims.user_id().to_string(), &user.into()).await?;

    Ok(HttpResponse::Ok().json(&data))
}

#[get("/{user_login}")]
async fn info(claims: JwtClaims, pool: web::Data<PgPool>, login: web::Path<String>) -> Result<HttpResponse, Error> {
    let user = claims.get_user(&pool).await?;
    let data = get_user_by_login(login.into_inner(), &user.into()).await?;

    Ok(HttpResponse::Ok().json(&data))
}

pub fn init_user_routes(config: &mut web::ServiceConfig) {
    config.service(me).service(info);
}