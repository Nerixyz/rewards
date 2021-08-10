use crate::{
    log_discord,
    models::editor::Editor,
    services::{jwt::JwtClaims, twitch::get_many_users},
    RedisPool,
};
use actix_web::{delete, get, put, web, HttpResponse, Result};
use sqlx::PgPool;

#[get("")]
async fn get_my_editors(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse> {
    let token = claims.get_user(&pool).await?.into();
    let editors = Editor::get_editors(claims.user_id(), &pool).await?;

    if editors.is_empty() {
        let data: Vec<String> = vec![];
        return Ok(HttpResponse::Ok().json(&data));
    }

    let mut redis_conn = redis
        .get()
        .await
        .map_err(|_| errors::ErrorInternalServerError("Redis is dead"))?;

    Ok(HttpResponse::Ok().json(get_many_users(editors, &token, &mut redis_conn).await?))
}

#[get("/broadcasters")]
async fn get_broadcasters(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse> {
    let token = claims.get_user(&pool).await?.into();
    let broadcasters = Editor::get_broadcasters(claims.user_id(), &pool).await?;

    if broadcasters.is_empty() {
        let data: Vec<String> = vec![];
        return Ok(HttpResponse::Ok().json(&data));
    }

    let mut redis_conn = redis
        .get()
        .await
        .map_err(|_| errors::ErrorInternalServerError("Redis is dead"))?;

    Ok(HttpResponse::Ok().json(get_many_users(broadcasters, &token, &mut redis_conn).await?))
}

#[put("/{editor_name}")]
async fn add_editor(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    editor: web::Path<String>,
) -> Result<HttpResponse> {
    Editor::add_editor(claims.user_id(), &editor, &pool).await?;
    log_discord!(
        "Editors",
        format!("✏ Added editor for {}", claims.user_id()),
        "editor_name" = editor.as_ref()
    );

    Ok(HttpResponse::Ok().finish())
}

#[delete("/{editor_name}")]
async fn delete_editor(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    editor: web::Path<String>,
) -> Result<HttpResponse> {
    Editor::delete_editor(claims.user_id(), &editor, &pool).await?;
    log_discord!(
        "Editors",
        format!("🗑 Removed editor for {}", claims.user_id()),
        "editor_name" = editor.as_ref()
    );

    Ok(HttpResponse::Ok().finish())
}

pub fn init_editor_routes(config: &mut web::ServiceConfig) {
    config
        .service(get_my_editors)
        .service(get_broadcasters)
        .service(add_editor)
        .service(delete_editor);
}
