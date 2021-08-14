use crate::{
    models::{discord, log_entry::LogEntry},
    services::{jwt::JwtClaims, sql::get_user_or_editor},
};
use actix_web::{
    delete, get, patch,
    web::{self, ServiceConfig},
    HttpResponse, Result,
};
use serde::Deserialize;
use sqlx::PgPool;

#[get("/{target_id}")]
async fn get_logs(
    claims: JwtClaims,
    target_id: web::Path<String>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse> {
    let user = get_user_or_editor(&claims, &target_id, &pool).await?;

    Ok(HttpResponse::Ok().json(LogEntry::get_for_user(&user.id, &pool).await?))
}

#[derive(Deserialize)]
struct SetUrlBody {
    url: String,
}

#[patch("/{target_id}/discord")]
async fn set_discord_url(
    claims: JwtClaims,
    target_id: web::Path<String>,
    body: web::Json<SetUrlBody>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse> {
    let user = get_user_or_editor(&claims, &target_id, &pool).await?;

    discord::set_discord_webhook_url(&user.id, &body.url, &pool).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[get("/{target_id}/discord")]
async fn get_discord_settings(
    claims: JwtClaims,
    target_id: web::Path<String>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse> {
    let user = get_user_or_editor(&claims, &target_id, &pool).await?;

    let settings = discord::get_discord_settings(&user.id, &pool).await?;

    Ok(HttpResponse::Ok().json(settings))
}

#[delete("/{target_id}/discord")]
async fn delete_discord_url(
    claims: JwtClaims,
    target_id: web::Path<String>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse> {
    let user = get_user_or_editor(&claims, &target_id, &pool).await?;

    discord::delete_discord_webhook_url(&user.id, &pool).await?;

    Ok(HttpResponse::NoContent().finish())
}

pub fn init_log_routes(config: &mut ServiceConfig) {
    config
        .service(get_logs)
        .service(get_discord_settings)
        .service(set_discord_url)
        .service(delete_discord_url);
}
