mod auth;
mod rewards;
mod user;
mod editors;

use actix_web::web;
use crate::repositories::auth::init_auth_routes;
use crate::repositories::rewards::init_rewards_routes;
use crate::repositories::user::init_user_routes;
use crate::repositories::editors::init_editor_routes;

pub fn init_repositories(config: &mut web::ServiceConfig) {
    config
        .service(web::scope("/auth").configure(init_auth_routes))
        .service(web::scope("/rewards").configure(init_rewards_routes))
        .service(web::scope("/users").configure(init_user_routes))
        .service(web::scope("/editors").configure(init_editor_routes));
}