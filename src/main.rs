use crate::actors::chat_actor::ChatActor;
use crate::actors::db_actor::DbActor;
use crate::actors::irc_actor::IrcActor;
use crate::actors::live_actor::LiveActor;
use crate::actors::messages::irc_messages::JoinAllMessage;
use crate::actors::messages::pubsub_messages::SubAllMessage;
use crate::actors::pubsub_actor::PubSubActor;
use crate::actors::slot_actor::SlotActor;
use crate::actors::timeout_actor::TimeoutActor;
use crate::actors::token_refresher::TokenRefresher;
use crate::constants::{DATABASE_URL, SERVER_URL, TWITCH_CLIENT_ID, TWITCH_CLIENT_SECRET};
use crate::middleware::useragent::UserAgentGuard;
use crate::models::user::User;
use crate::repositories::init_repositories;
use crate::services::eventsub::{
    clear_invalid_rewards, clear_unfulfilled_redemptions, register_eventsub_for_all_unregistered,
};
use crate::services::metrics::register_metrics;
use crate::services::timed_mode::resolve_timed_modes;
use actix::Actor;
use actix_cors::Cors;
use actix_files::NamedFile;
use actix_metrics::Metrics;
use actix_web::http::header::{AUTHORIZATION, CONTENT_TYPE};
use actix_web::middleware::{DefaultHeaders, Logger};
use actix_web::{web, App, HttpResponse, HttpServer};
use anyhow::Error as AnyError;
use log::LevelFilter;
use metrics_exporter_prometheus::PrometheusBuilder;
use sqlx::postgres::PgConnectOptions;
use sqlx::{ConnectOptions, PgPool};
use std::str::FromStr;
use tokio::sync::RwLock;
use twitch_api2::helix::Scope;
use twitch_api2::twitch_oauth2::client::reqwest_http_client;
use twitch_api2::twitch_oauth2::{AppAccessToken, ClientId, ClientSecret};

mod actors;
mod chat;
mod constants;
mod extractors;
mod middleware;
mod models;
mod repositories;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::builder().format_timestamp(None).init();

    let prom_recorder = Box::leak(Box::new(PrometheusBuilder::new().build()));
    let prom_handle = prom_recorder.handle();
    metrics::set_recorder(prom_recorder).expect("Couldn't set recorder");
    Metrics::register_metrics();
    register_metrics();

    log::info!("Connecting to database");

    let mut pool_options =
        PgConnectOptions::from_str(DATABASE_URL).expect("couldn't read database url");
    pool_options.log_statements(LevelFilter::Debug);
    let pool = PgPool::connect_with(pool_options)
        .await
        .expect("Could not connect to database");

    log::info!("Starting Db, Irc and Slot-Actor");

    let chat_actor = ChatActor::new(pool.clone()).start();

    let timeout_actor = TimeoutActor::new(pool.clone()).start();

    let db_actor = DbActor::new(pool.clone()).start();
    let irc_actor = IrcActor::new(
        db_actor.clone(),
        pool.clone(),
        chat_actor.recipient(),
        timeout_actor.clone(),
    )
    .start();
    let _slot_actor = SlotActor::new(pool.clone()).start();

    log::info!("Joining all channels");

    let names = User::get_all_names(&pool)
        .await
        .expect("Could not get users");
    irc_actor.do_send(JoinAllMessage(names));

    log::info!("Getting access-token");

    let app_access_token = get_app_access_token()
        .await
        .expect("Could not get app access token");
    let app_access_token = web::Data::new(RwLock::new(app_access_token));
    let _refresh_actor = TokenRefresher::new(pool.clone()).start();
    let live_actor = LiveActor::new(pool.clone(), irc_actor.clone()).start();
    let pubsub = PubSubActor::new(pool.clone(), live_actor, timeout_actor.clone()).start();
    let initial_listens = make_initial_pubsub_listens(&pool)
        .await
        .expect("sql thingy");
    pubsub.do_send(SubAllMessage(initial_listens));

    log::info!("Clearing old rewards");

    clear_invalid_rewards(&app_access_token, &pool)
        .await
        .expect("Could not clear invalid rewards");

    log::info!("Registering eventsub callbacks");

    register_eventsub_for_all_unregistered(&app_access_token, &pool)
        .await
        .expect("Could not register eventsub FeelsMan");

    let clear_pool = pool.clone();
    let clear_irc = irc_actor.clone();
    actix::spawn(async move {
        let (redemptions, timed_mode) = futures::future::join(
            clear_unfulfilled_redemptions(&clear_pool),
            resolve_timed_modes(clear_irc, &clear_pool),
        )
        .await;
        log_err!(redemptions, "Failed to clear redemptions");
        log_err!(timed_mode, "Could not clear timed modes");
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(irc_actor.clone()))
            .app_data(web::Data::new(timeout_actor.clone()))
            .app_data(web::Data::new(pubsub.clone()))
            .app_data(web::Data::new(prom_handle.clone()))
            .app_data(app_access_token.clone())
            .wrap(get_default_headers())
            .wrap(create_cors())
            .wrap(UserAgentGuard::single("paloaltonetworks.com".to_string()))
            .wrap(Logger::default().exclude("/api/v1/metrics"))
            .service(
                web::scope("/api/v1")
                    .configure(init_repositories)
                    .default_service(web::route().to(HttpResponse::NotFound)),
            )
            .service(actix_files::Files::new("/", "web/dist").index_file("index.html"))
            .default_service(NamedFile::open("web/dist/index.html").unwrap())
    })
    .bind("127.0.0.1:8082")?
    .run()
    .await
}

async fn get_app_access_token() -> Result<AppAccessToken, AnyError> {
    Ok(AppAccessToken::get_app_access_token(
        reqwest_http_client,
        ClientId::new(TWITCH_CLIENT_ID.to_string()),
        ClientSecret::new(TWITCH_CLIENT_SECRET.to_string()),
        vec![
            Scope::ChannelReadRedemptions,
            Scope::ChannelManageRedemptions,
        ],
    )
    .await?)
}

fn get_default_headers() -> DefaultHeaders {
    DefaultHeaders::new().header("X-Rewards-Version", env!("CARGO_PKG_VERSION"))
}

fn create_cors() -> Cors {
    if cfg!(debug_assertions) {
        Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allowed_headers(vec![AUTHORIZATION, CONTENT_TYPE])
    } else {
        Cors::default().allowed_origin(SERVER_URL)
    }
}

async fn make_initial_pubsub_listens(pool: &PgPool) -> Result<Vec<String>, AnyError> {
    let users = User::get_all(pool).await?;

    Ok(users.into_iter().map(|user| user.id).collect())
}
