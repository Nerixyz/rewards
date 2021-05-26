use crate::actors::db_actor::DbActor;
use crate::actors::irc_actor::IrcActor;
use crate::actors::messages::irc_messages::JoinAllMessage;
use crate::actors::token_refresher::TokenRefresher;
use crate::constants::{DATABASE_URL, TWITCH_CLIENT_ID, TWITCH_CLIENT_SECRET};
use crate::models::user::User;
use crate::repositories::init_repositories;
use crate::services::eventsub::{clear_invalid_rewards, register_eventsub_for_all_unregistered};
use actix::Actor;
use actix_files::NamedFile;
use actix_web::middleware::DefaultHeaders;
use actix_web::{guard, web, App, HttpResponse, HttpServer};
use anyhow::Error as AnyError;
use sqlx::PgPool;
use tokio::sync::Mutex;
use twitch_api2::helix::Scope;
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
    let pool = PgPool::connect(DATABASE_URL)
        .await
        .expect("Could not connect to database");

    let db_actor = DbActor::new(pool.clone()).start();
    let irc_actor = IrcActor::new(db_actor.clone()).start();

    let names = User::get_all_names(&pool)
        .await
        .expect("Could not get users");
    irc_actor.do_send(JoinAllMessage(names));

    let app_access_token = get_app_access_token()
        .await
        .expect("Could not get app access token");
    let app_access_token = web::Data::new(Mutex::new(app_access_token));
    let _refresh_actor = TokenRefresher::new(pool.clone()).start();

    clear_invalid_rewards(&app_access_token, &pool)
        .await
        .expect("Could not clear invalid rewards");
    register_eventsub_for_all_unregistered(&app_access_token, &pool)
        .await
        .expect("Could not register eventsub FeelsMan");

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(irc_actor.clone())
            .app_data(app_access_token.clone())
            .wrap(get_default_headers())
            .service(web::scope("/api/v1").configure(init_repositories))
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
