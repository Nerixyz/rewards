mod auth;
mod connections;
mod editors;
mod eventsub;
mod logs;
mod rewards;
mod user;

use crate::middleware::eventsub::EventsubGuard;
use crate::repositories::auth::init_auth_routes;
use crate::repositories::connections::init_connection_routes;
use crate::repositories::editors::init_editor_routes;
use crate::repositories::eventsub::init_eventsub_routes;
use crate::repositories::logs::init_log_routes;
use crate::repositories::rewards::init_rewards_routes;
use crate::repositories::user::init_user_routes;
use actix_metrics::Metrics;
use actix_web::{get, web, Responder};
use metrics_exporter_prometheus::PrometheusHandle;
use std::future::{ready, Ready};

#[get("/metrics")]
fn metrics_render(handle: web::Data<PrometheusHandle>) -> Ready<impl Responder> {
    ready(handle.render())
}

pub fn init_repositories(config: &mut web::ServiceConfig) {
    config
        .service(
            web::scope("/auth")
                .wrap(Metrics::new("auth"))
                .configure(init_auth_routes),
        )
        .service(
            web::scope("/rewards")
                .wrap(Metrics::new("rewards"))
                .configure(init_rewards_routes),
        )
        .service(
            web::scope("/users")
                .wrap(Metrics::new("users"))
                .configure(init_user_routes),
        )
        .service(
            web::scope("/editors")
                .wrap(Metrics::new("editors"))
                .configure(init_editor_routes),
        )
        .service(
            web::scope("/logs")
                .wrap(Metrics::new("logs"))
                .configure(init_log_routes),
        )
        .service(
            web::scope("/connections")
                .wrap(Metrics::new("connections"))
                .configure(init_connection_routes),
        )
        .service(
            web::scope("/eventsub")
                .wrap(Metrics::new("eventsub"))
                .wrap(EventsubGuard)
                .configure(init_eventsub_routes),
        )
        .service(metrics_render);
}
