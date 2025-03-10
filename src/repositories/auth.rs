use actix::Addr;
use actix_web::{
    cookie::CookieBuilder, delete, get, web, HttpResponse, Result,
};
use errors::redirect_error::RedirectError;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::future;
use time::{Duration, OffsetDateTime};
use tokio::sync::RwLock;
use twitch_api::twitch_oauth2::{
    tokens::UserTokenBuilder, AppAccessToken, ClientId, ClientSecret,
    CsrfToken, Scope, TwitchToken, UserToken,
};
use url::Url;

use crate::{
    actors::{
        irc::{IrcActor, JoinMessage, PartMessage},
        pubsub::{PubSubActor, SubMessage},
    },
    log_discord,
    services::{
        eventsub::{
            register_all_eventsub_for_id, unregister_eventsub_for_user,
        },
        jwt::{encode_jwt, JwtClaims},
        twitch::{self, requests::delete_reward},
    },
    util::result::ResultExt,
};
use config::CONFIG;
use models::{reward::Reward, user::User};

#[derive(Deserialize)]
#[non_exhaustive]
struct TwitchCallbackQuery {
    code: Option<String>,
    scope: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[get("/twitch-callback")]
async fn twitch_callback(
    pool: web::Data<PgPool>,
    irc: web::Data<Addr<IrcActor>>,
    pubsub: web::Data<Addr<PubSubActor>>,
    app_access_token: web::Data<RwLock<AppAccessToken>>,
    query: web::Query<TwitchCallbackQuery>,
) -> Result<HttpResponse> {
    let query = query.into_inner();
    let (code, scope) = match (query.code, query.scope) {
        (Some(code), Some(scope)) => (code, scope),
        _ => {
            log::info!("{:?} {:?}", query.error, query.error_description);
            return Err(RedirectError::new(
                "/failed-auth",
                query.error_description.or(query.error),
            )
            .into());
        }
    };

    let mut builder = UserTokenBuilder::new(
        ClientId::new(CONFIG.twitch.client_id.to_string()),
        ClientSecret::new(CONFIG.twitch.client_secret.to_string()),
        Url::parse(&format!(
            "{}/api/v1/auth/twitch-callback",
            CONFIG.server.url
        ))
        .expect("Invalid redirect-url"),
    );

    builder.set_csrf(CsrfToken::new("".to_string()));

    let user_token = builder
        .get_user_token(&*twitch::CLIENT, "", &code)
        .await
        .map_err(|_| {
            RedirectError::new("/failed-auth", Some("Could not get token"))
        })?;

    let refresh_token = user_token
        .refresh_token
        .ok_or_else(|| RedirectError::<&str, &str>::simple("/failed-auth"))?
        .take();

    let user = User {
        id: user_token.user_id.clone().take(),
        refresh_token,
        access_token: user_token.access_token.take(),
        scopes: scope,
        name: user_token.login.take(),
    };

    user.create(&pool).await.map_err(|_| {
        RedirectError::new("/failed-auth", Some("Could not create user"))
    })?;

    // register and save the id into the database
    register_all_eventsub_for_id(&user_token.user_id, &app_access_token, &pool)
        .await?;

    log::info!("AUTH: Registered {}", user.name);
    log_discord!(
        "Auth",
        format!("🎉 Registered {}", user.name),
        "scopes" = user.scopes
    );

    // join the user's channel
    irc.do_send(JoinMessage(user.name));
    pubsub.do_send(SubMessage(user.id));

    let token = encode_jwt(&JwtClaims::new(user_token.user_id.take()))
        .map_err(|_| {
            RedirectError::new("/failed-auth", Some("Could not encode"))
        })?;
    Ok(HttpResponse::Found()
        .append_header(("location", "/"))
        .cookie(
            CookieBuilder::new("auth_token", token)
                .expires(Some(OffsetDateTime::now_utc() + Duration::days(365)))
                .path("/")
                .finish(),
        )
        .finish())
}

#[derive(Serialize)]
struct TwitchOAuthParams {
    client_id: String,
    redirect_uri: String,
    response_type: String,
    scope: String,
}

#[get("/twitch-auth")]
fn redirect_to_twitch_auth() -> future::Ready<HttpResponse> {
    let params = TwitchOAuthParams {
        client_id: CONFIG.twitch.client_id.to_string(),
        redirect_uri: format!(
            "{}/api/v1/auth/twitch-callback",
            CONFIG.server.url
        ),
        response_type: "code".to_string(),
        scope: [
            Scope::ChannelManageRedemptions,
            Scope::ChannelReadRedemptions,
        ]
        .iter()
        .map(ToString::to_string)
        .join(" "),
    };
    let url = format!(
        "https://id.twitch.tv/oauth2/authorize?{}",
        serde_qs::to_string(&params).expect("Failed to serialize")
    );

    future::ready(
        HttpResponse::Found()
            .append_header(("location", url))
            .finish(),
    )
}

#[delete("")]
async fn revoke(
    claims: JwtClaims,
    app_access_token: web::Data<RwLock<AppAccessToken>>,
    pool: web::Data<PgPool>,
    irc: web::Data<Addr<IrcActor>>,
) -> Result<HttpResponse> {
    let user = claims.get_user(&pool).await?;
    let user_name = user.name.clone();
    let token: UserToken = user.into();

    unregister_eventsub_for_user(
        token.user_id.as_str(),
        &app_access_token,
        &pool,
    )
    .await
    .log_if_err("unregistering eventsub");

    if let Ok(rewards) = Reward::get_all_for_user(&token.user_id, &pool).await {
        for reward in rewards {
            delete_reward(&reward.user_id, reward.id, &token).await.ok();
        }
    }

    if let Err(e) = token.revoke_token(&*twitch::CLIENT).await {
        // we don't return the error, so me make sure everything is cleaned up
        log::warn!("Revoke token error: {}", e);
    }

    log::info!("AUTH: Revoked {}", user_name);
    log_discord!("Auth", format!("❌ Revoked {}", user_name),);

    irc.do_send(PartMessage(user_name));

    // here we can return the error as there's no work afterwards
    User::delete(claims.user_id(), &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

pub fn init_auth_routes(config: &mut web::ServiceConfig) {
    config
        .service(redirect_to_twitch_auth)
        .service(twitch_callback)
        .service(revoke);
}
