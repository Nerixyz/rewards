use crate::{
    models::log_entry::LogEntry,
    services::{jwt::JwtClaims, sql::get_user_or_editor},
};
use actix_web::{
    get,
    web::{self, ServiceConfig},
    HttpResponse, Result,
};
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

pub fn init_log_routes(config: &mut ServiceConfig) {
    config.service(get_logs);
}
