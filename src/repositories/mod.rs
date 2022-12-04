mod auth;
mod connections;
mod editors;
mod eventsub;
mod logs;
mod rewards;
mod user;

use crate::repositories::{
    auth::init_auth_routes, connections::init_connection_routes,
    editors::init_editor_routes, eventsub::init_eventsub_routes,
    logs::init_log_routes, rewards::init_rewards_routes,
    user::init_user_routes,
};
use actix_web::web;

pub fn init_repositories(config: &mut web::ServiceConfig) {
    config
        .service(web::scope("/auth").configure(init_auth_routes))
        .service(web::scope("/rewards").configure(init_rewards_routes))
        .service(web::scope("/users").configure(init_user_routes))
        .service(web::scope("/editors").configure(init_editor_routes))
        .service(web::scope("/logs").configure(init_log_routes))
        .service(web::scope("/connections").configure(init_connection_routes))
        .service(web::scope("/eventsub").configure(init_eventsub_routes));
}
