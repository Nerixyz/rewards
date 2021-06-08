mod auth;
mod connections;
mod editors;
mod eventsub;
mod logs;
mod rewards;
mod user;

use crate::guards::eventsub::EventsubGuard;
use crate::repositories::auth::init_auth_routes;
use crate::repositories::connections::init_connection_routes;
use crate::repositories::editors::init_editor_routes;
use crate::repositories::eventsub::init_eventsub_routes;
use crate::repositories::logs::init_log_routes;
use crate::repositories::rewards::init_rewards_routes;
use crate::repositories::user::init_user_routes;
use actix_web::{web, HttpResponse};

pub fn init_repositories(config: &mut web::ServiceConfig) {
    config
        .service(web::scope("/auth").configure(init_auth_routes))
        .service(web::scope("/rewards").configure(init_rewards_routes))
        .service(web::scope("/users").configure(init_user_routes))
        .service(web::scope("/editors").configure(init_editor_routes))
        .service(web::scope("/logs").configure(init_log_routes))
        .service(web::scope("/connections").configure(init_connection_routes))
        .service(
            web::scope("/eventsub")
                .wrap(EventsubGuard)
                .configure(init_eventsub_routes),
        ).service(
        web::resource("")
            .route(web::get().to(HttpResponse::NotFound)));
}
