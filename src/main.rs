use crate::actors::db_actor::DbActor;
use crate::actors::irc_actor::IrcActor;
use crate::actors::live_actor::LiveActor;
use crate::actors::messages::irc_messages::JoinAllMessage;
use crate::actors::messages::pubsub_messages::SubAllMessage;
use crate::actors::pubsub_actor::PubSubActor;
use crate::actors::slot_actor::SlotActor;
use crate::actors::token_refresher::TokenRefresher;
use crate::constants::{DATABASE_URL, TWITCH_CLIENT_ID, TWITCH_CLIENT_SECRET};
use crate::models::user::User;
use crate::repositories::init_repositories;
use crate::services::eventsub::{
    clear_invalid_rewards, clear_unfulfilled_redemptions, register_eventsub_for_all_unregistered,
};
use actix::Actor;
use actix_files::NamedFile;
use actix_web::dev::Service;
use actix_web::http::header::USER_AGENT;
use actix_web::http::HeaderValue;
use actix_web::middleware::{DefaultHeaders, Logger};
use actix_web::{guard, web, App, HttpResponse, HttpServer};
use anyhow::Error as AnyError;
use log::LevelFilter;
use rand::distributions::Alphanumeric;
use rand::Rng;
use sqlx::postgres::PgConnectOptions;
use sqlx::{ConnectOptions, PgPool};
use std::str::FromStr;
use tokio::sync::Mutex;
use twitch_api2::helix::Scope;
use twitch_api2::pubsub::video_playback::VideoPlaybackById;
use twitch_api2::pubsub::{listen_command, Topics};
use twitch_api2::twitch_oauth2::client::reqwest_http_client;
use twitch_api2::twitch_oauth2::{AppAccessToken, ClientId, ClientSecret};

mod actors;
mod constants;
mod extractors;
mod guards;
mod models;
mod repositories;
mod services;

async fn web_index() -> std::io::Result<NamedFile> {
    NamedFile::open("web/dist/index.html")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::builder().format_timestamp(None).init();

    log::info!("Connecting to database");

    let mut pool_options =
        PgConnectOptions::from_str(DATABASE_URL).expect("couldn't read database url");
    pool_options.log_statements(LevelFilter::Debug);
    let pool = PgPool::connect_with(pool_options)
        .await
        .expect("Could not connect to database");

    log::info!("Starting Db, Irc and Slot-Actor");

    let db_actor = DbActor::new(pool.clone()).start();
    let irc_actor = IrcActor::new(db_actor.clone()).start();
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
    let app_access_token = web::Data::new(Mutex::new(app_access_token));
    let _refresh_actor = TokenRefresher::new(pool.clone()).start();
    let live_actor = LiveActor::new(pool.clone(), irc_actor.clone()).start();
    let pubsub = PubSubActor::new(live_actor).start();
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
    actix::spawn(async move {
        if let Err(e) = clear_unfulfilled_redemptions(&clear_pool).await {
            log::warn!("Failed to clear redemptions: {}", e);
        }
    });

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(irc_actor.clone())
            .data(pubsub.clone())
            .app_data(app_access_token.clone())
            .wrap(get_default_headers())
            .wrap(Logger::default())
            .wrap_fn(|req, srv| {
                let header: &str = req
                    .headers()
                    .get(USER_AGENT)
                    .map(|ua: &HeaderValue| ua.to_str().ok())
                    .flatten()
                    .unwrap_or("");
                let fut = if header.contains("paloaltonetworks.com") {
                    None
                } else {
                    Some(srv.call(req))
                };
                async {
                    if let Some(fut) = fut {
                        fut.await
                    } else {
                        Err(actix_web::error::ErrorImATeapot("No, I don't think so"))
                    }
                }
            })
            .service(
                web::scope("/api/v1")
                    .configure(init_repositories)
                    .default_service(
                        web::resource("").route(web::route().to(HttpResponse::NotFound)),
                    ),
            )
            .service(actix_files::Files::new("/", "web/dist").index_file("index.html"))
            .default_service(
                web::resource("")
                    .route(web::get().to(web_index))
                    .route(web::route().guard(guard::Options()).to(HttpResponse::Ok))
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::NotFound),
                    ),
            )
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
    let headers = DefaultHeaders::new().header("X-Rewards-Version", env!("CARGO_PKG_VERSION"));

    if cfg!(debug_assertions) {
        headers
            .header("Access-Control-Allow-Origin", "*")
            .header(
                "Access-Control-Allow-Methods",
                "GET, POST, PUT, DELETE, OPTIONS, PATCH",
            )
            .header(
                "Access-Control-Allow-Headers",
                "Authorization, Content-Type",
            )
    } else {
        headers
    }
}

async fn make_initial_pubsub_listens(pool: &PgPool) -> Result<Vec<String>, AnyError> {
    let users = User::get_all(pool).await?;

    Ok(users
        .into_iter()
        .filter_map(|user| {
            let nonce = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(30)
                .map(char::from)
                .collect::<String>();
            listen_command(
                &[Topics::VideoPlaybackById(VideoPlaybackById {
                    channel_id: user.id.parse().unwrap_or_default(),
                })],
                &user.access_token,
                nonce.as_str(),
            )
            .ok()
        })
        .collect())
}
