extern crate twitch_api as twitch_api2;

use actix::{Actor, SystemRegistry};
use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::{
    http::header::{AUTHORIZATION, CONTENT_TYPE},
    middleware::{DefaultHeaders, Logger},
    web, App, HttpResponse, HttpServer,
};
use anyhow::Error as AnyError;
use log::LevelFilter;
use sqlx::{postgres::PgConnectOptions, ConnectOptions, PgPool};
use tokio::sync::RwLock;
use twitch_api2::{
    helix::Scope,
    twitch_oauth2::{AppAccessToken, ClientId, ClientSecret},
};

use actors::{irc::JoinAllMessage, pubsub::SubAllMessage};

use crate::{
    actors::{
        chat::ChatActor, db::DbActor, discord::DiscordActor, irc::IrcActor,
        live::LiveActor, pubsub::PubSubActor, rewards::RewardsActor,
        slot::SlotActor, supinic::SupinicActor, timeout::TimeoutActor,
        token_refresher::TokenRefresher,
    },
    middleware::useragent::UserAgentGuard,
    repositories::init_repositories,
    services::{
        eventsub::{
            clear_invalid_rewards, clear_unfulfilled_redemptions,
            register_eventsub_for_all_unregistered,
        },
        twitch::{self, requests::send_chat_message},
    },
};
use actix_web_prom::PrometheusMetricsBuilder;
use config::CONFIG;
use models::user::User;

pub type RedisPool = deadpool_redis::Pool;
pub type RedisConn = deadpool_redis::Connection;

mod actors;
mod chat;
mod extractors;
mod middleware;
mod repositories;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::builder().format_timestamp(None).init();

    log::info!("Initializing config");

    lazy_static::initialize(&CONFIG);

    let prometheus = PrometheusMetricsBuilder::new("actix")
        .endpoint("/api/v1/metrics")
        .build()
        .unwrap();

    log::info!("Connecting to database");

    let pool_options = CONFIG
        .db
        .url
        .parse::<PgConnectOptions>()
        .expect("invalid db config")
        .log_statements(LevelFilter::Debug);
    let pg_pool = PgPool::connect_with(pool_options)
        .await
        .expect("Could not connect to database");
    twitch::set_token(
        models::config::ConfigEntry::get_user_token(&pg_pool)
            .await
            .expect("must have token"),
    );

    log::info!("Connecting to redis");

    let redis_pool = deadpool_redis::Config {
        url: Some(CONFIG.redis.url.clone()),
        connection: None,
        ..Default::default()
    }
    .create_pool(Some(deadpool_redis::Runtime::Tokio1))
    .expect("Could not create redis pool");

    // make sure the connection is working and there's at least one connected client
    let _ = redis_pool.get().await.unwrap();

    log::info!("Starting Db, Irc and Slot-Actor");

    log::info!("Getting access-token");

    let app_access_token = get_app_access_token()
        .await
        .expect("Could not get app access token");
    let app_access_token = web::Data::new(RwLock::new(app_access_token));

    let chat_actor = ChatActor::new(
        pg_pool.clone(),
        redis_pool.clone(),
        app_access_token.clone().into_inner(),
    )
    .start();

    let timeout_actor = TimeoutActor::new(redis_pool.clone()).start();

    let db_actor = DbActor::new(pg_pool.clone()).start();
    let irc_actor = IrcActor::new(
        db_actor.clone(),
        chat_actor.recipient(),
        timeout_actor.clone(),
    )
    .start();
    let discord_user_actor = DiscordActor::new(pg_pool.clone()).start();

    SystemRegistry::set(
        SlotActor::new(
            pg_pool.clone(),
            redis_pool.clone(),
            discord_user_actor.clone(),
        )
        .start(),
    );

    log::info!("Announcing on twitch and discord");

    announce_start();

    log::info!("Joining all channels");

    let names = User::get_all_names(&pg_pool)
        .await
        .expect("Could not get users");
    irc_actor.do_send(JoinAllMessage(names));

    TokenRefresher::new(pg_pool.clone(), db_actor).start();
    let live_actor = LiveActor::new(pg_pool.clone()).start();
    let pubsub =
        PubSubActor::run(pg_pool.clone(), live_actor, timeout_actor.clone());
    let initial_listens = make_initial_pubsub_listens(&pg_pool)
        .await
        .expect("sql thingy");
    pubsub.do_send(SubAllMessage(initial_listens));

    let rewards_actor = RewardsActor {
        db: pg_pool.clone(),
        app_access_token: app_access_token.clone().into_inner(),
        timeout: timeout_actor.clone(),
        redis: redis_pool.clone(),
        discord: discord_user_actor,
    }
    .start();

    SupinicActor.start();

    log::info!("Clearing old rewards");

    clear_invalid_rewards(&app_access_token, &pg_pool)
        .await
        .expect("Could not clear invalid rewards");

    log::info!("Registering eventsub callbacks");

    register_eventsub_for_all_unregistered(&app_access_token, &pg_pool)
        .await
        .expect("Could not register eventsub FeelsMan");

    let clear_pool = pg_pool.clone();
    actix::spawn(async move {
        log_err!(
            clear_unfulfilled_redemptions(&clear_pool).await,
            "Failed to clear redemptions"
        );
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pg_pool.clone()))
            .app_data(web::Data::new(redis_pool.clone()))
            .app_data(web::Data::new(irc_actor.clone()))
            .app_data(web::Data::new(timeout_actor.clone()))
            .app_data(web::Data::new(rewards_actor.clone()))
            .app_data(web::Data::new(pubsub.clone()))
            .app_data(app_access_token.clone())
            .wrap(get_default_headers())
            .wrap(UserAgentGuard::single("paloaltonetworks.com".to_string()))
            .wrap(create_cors())
            .wrap(prometheus.clone())
            .wrap(Logger::default().exclude("/api/v1/metrics"))
            .service(
                web::scope("/api/v1")
                    .configure(init_repositories)
                    .default_service(web::route().to(HttpResponse::NotFound)),
            )
            .service(
                actix_files::Files::new("/", "web/dist")
                    .index_file("index.html"),
            )
            .default_service(NamedFile::open("web/dist/index.html").unwrap())
    })
    .bind(&CONFIG.server.bind_addr)?
    .run()
    .await
}

async fn get_app_access_token() -> Result<AppAccessToken, AnyError> {
    Ok(AppAccessToken::get_app_access_token(
        &reqwest::Client::new(),
        ClientId::new(CONFIG.twitch.client_id.to_string()),
        ClientSecret::new(CONFIG.twitch.client_secret.to_string()),
        vec![
            Scope::ChannelReadRedemptions,
            Scope::ChannelManageRedemptions,
        ],
    )
    .await?)
}

fn get_default_headers() -> DefaultHeaders {
    DefaultHeaders::new().add(("X-Rewards-Version", env!("CARGO_PKG_VERSION")))
}

fn create_cors() -> Cors {
    if cfg!(debug_assertions) {
        Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allowed_headers(vec![AUTHORIZATION, CONTENT_TYPE])
    } else {
        Cors::default().allowed_origin(&CONFIG.server.url)
    }
}

async fn make_initial_pubsub_listens(
    pool: &PgPool,
) -> Result<Vec<String>, AnyError> {
    let users = User::get_all(pool).await?;

    Ok(users.into_iter().map(|user| user.id).collect())
}

fn announce_start() {
    let announce = match CONFIG.log.announce {
        Some(ref a) => a,
        _ => return,
    };
    let instance_str = format!(
        "[{build_profile}] 🏗 {git_info} 🖥 {build_info} 🛠 rustc {rustc_info}",
        git_info = env!("RW_GIT_INFO"),
        rustc_info = env!("RW_RUSTC_INFO"),
        build_info = env!("RW_BUILD_INFO"),
        build_profile = env!("RW_BUILD_PROFILE")
    );

    if announce.discord {
        log_discord!(format!("Running. {}", instance_str));
    }
    if let Some(ref twitch) = announce.twitch {
        let prefix = twitch.prefix.as_deref().unwrap_or("Running.");
        tokio::spawn(async move {
            let token = twitch::get_token();
            let Ok(user) =
                twitch::requests::get_user_by_login(&twitch.channel, &token)
                    .await
            else {
                log::warn!("Failed to get initial channel");
                return;
            };
            log_err!(
                send_chat_message(
                    user.id.as_str(),
                    &format!("{} {}", prefix, instance_str),
                    &twitch::get_token()
                )
                .await,
                "Could not announce in channel"
            );
        });
    }
}
