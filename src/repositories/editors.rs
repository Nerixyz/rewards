use crate::models::editor::Editor;
use crate::services::jwt::JwtClaims;
use crate::services::twitch::requests::get_users;
use actix_web::{delete, get, put, web, Error, HttpResponse};
use sqlx::PgPool;

#[get("")]
async fn get_my_editors(claims: JwtClaims, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    let token = claims.get_user(&pool).await?.into();
    let editors = Editor::get_editors(claims.user_id(), &pool).await?;

    if editors.is_empty() {
        let data: Vec<String> = vec![];
        return Ok(HttpResponse::Ok().json(&data));
    }

    Ok(HttpResponse::Ok().json(get_users(editors, &token).await?))
}

#[get("/broadcasters")]
async fn get_broadcasters(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let token = claims.get_user(&pool).await?.into();
    let broadcasters = Editor::get_broadcasters(claims.user_id(), &pool).await?;

    if broadcasters.is_empty() {
        let data: Vec<String> = vec![];
        return Ok(HttpResponse::Ok().json(&data));
    }

    Ok(HttpResponse::Ok().json(get_users(broadcasters, &token).await?))
}

#[put("/{editor_name}")]
async fn add_editor(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    editor: web::Path<String>,
) -> Result<HttpResponse, Error> {
    Editor::add_editor(claims.user_id(), &editor, &pool).await?;
    Ok(HttpResponse::Ok().finish())
}

#[delete("/{editor_name}")]
async fn delete_editor(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    editor: web::Path<String>,
) -> Result<HttpResponse, Error> {
    Editor::delete_editor(claims.user_id(), &editor, &pool).await?;
    Ok(HttpResponse::Ok().finish())
}

pub fn init_editor_routes(config: &mut web::ServiceConfig) {
    config
        .service(get_my_editors)
        .service(get_broadcasters)
        .service(add_editor)
        .service(delete_editor);
}
