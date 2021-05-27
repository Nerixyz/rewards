use crate::actors::irc_actor::IrcActor;
use crate::actors::messages::irc_messages::{JoinMessage, PartMessage};
use crate::constants::{SERVER_URL, TWITCH_CLIENT_ID, TWITCH_CLIENT_SECRET};
use crate::models::user::User;
use crate::services::eventsub::{register_eventsub_for_id, unregister_eventsub_for_id};
use crate::services::jwt::{encode_jwt, JwtClaims};
use actix::Addr;
use actix_web::body::Body;
use actix_web::cookie::CookieBuilder;
use actix_web::http::{header, StatusCode};
use actix_web::{delete, error, get, web, BaseHttpResponse, Result, HttpResponse};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use tokio::sync::Mutex;
use twitch_api2::twitch_oauth2::client::reqwest_http_client;
use twitch_api2::twitch_oauth2::tokens::UserTokenBuilder;
use twitch_api2::twitch_oauth2::{
    AppAccessToken, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TwitchToken, UserToken,
};

#[derive(Debug, derive_more::Display, derive_more::Error)]
#[display(fmt = "Error during oauth authorization")]
struct OAuthError;

impl error::ResponseError for OAuthError {
    fn error_response(&self) -> BaseHttpResponse<Body> {
        // HttpResponse::MovedPermanently().append_header((header::LOCATION, "/failed-auth")).finish().into_body();

        let mut resp = BaseHttpResponse::new(StatusCode::FOUND);
        resp.headers_mut().insert(
            header::LOCATION,
            header::HeaderValue::from_static("/failed-auth"),
        );
        resp.set_body(Body::None)
    }
}

#[derive(Deserialize)]
#[non_exhaustive]
struct TwitchCallbackQuery {
    code: String,
    scope: String,
}

#[get("/twitch-callback")]
async fn twitch_callback(
    pool: web::Data<PgPool>,
    irc: web::Data<Addr<IrcActor>>,
    app_access_token: web::Data<Mutex<AppAccessToken>>,
    query: web::Query<TwitchCallbackQuery>,
) -> Result<HttpResponse> {
    let mut builder = UserTokenBuilder::new(
        ClientId::new(TWITCH_CLIENT_ID.to_string()),
        ClientSecret::new(TWITCH_CLIENT_SECRET.to_string()),
        RedirectUrl::new(format!("{}/api/v1/auth/twitch-callback", SERVER_URL))
            .expect("Invalid redirect-url"),
    )
    .expect("Invalid url");

    builder.set_csrf(CsrfToken::new("".to_string()));

    let user_token = builder
        .get_user_token(reqwest_http_client, "", &query.code)
        .await
        .map_err(|_| OAuthError)?;

    let refresh_token = user_token.refresh_token.ok_or(OAuthError)?;

    let user = User {
        id: user_token.user_id.clone(),
        refresh_token: refresh_token.secret().clone(),
        access_token: user_token.access_token.secret().clone(),
        scopes: query.scope.clone(),
        name: user_token.login.clone(),
        eventsub_id: None,
    };

    user.create(&pool).await.map_err(|_| OAuthError)?;

    // register and save the id into the database
    register_eventsub_for_id(&user_token.user_id, &app_access_token, &pool).await?;

    log::info!("AUTH: Registered {}", user.name);

    // join the user's channel
    irc.do_send(JoinMessage(user.name));

    let token = encode_jwt(&JwtClaims::new(user_token.user_id.clone())).map_err(|_| OAuthError)?;
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

#[derive(Serialize)]
struct TwitchAuthUrlResponse {
    url: String,
}

#[get("/twitch-auth-url")]
fn create_twitch_url() -> HttpResponse {
    // TODO: redirect
    let params = TwitchOAuthParams {
        client_id: TWITCH_CLIENT_ID.to_string(),
        redirect_uri: format!("{}/api/v1/auth/twitch-callback", SERVER_URL),
        response_type: "code".to_string(),
        scope: vec![
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

    HttpResponse::Ok().json(TwitchAuthUrlResponse { url })
}

#[delete("")]
async fn revoke(
    claims: JwtClaims,
    app_access_token: web::Data<Mutex<AppAccessToken>>,
    pool: web::Data<PgPool>,
    irc: web::Data<Addr<IrcActor>>,
) -> Result<HttpResponse> {
    let user = claims.get_user(&pool).await?;
    let user_name = user.name.clone();
    let eventsub_id = user.eventsub_id.clone();
    let token: UserToken = user.into();
    if let Err(e) = token.revoke_token(reqwest_http_client).await {
        // we don't return the error, so me make sure everything is cleaned up
        println!("Revoke token error: {}", e);
    }

    if let Some(id) = eventsub_id {
        if let Err(e) = unregister_eventsub_for_id(id, &app_access_token, &pool).await {
            // we don't return the error, so me make sure everything is cleaned up
            println!("Eventsub unregister error: {}", e);
        }
    }
    log::info!("AUTH: Revoked {}", user_name);

    irc.do_send(PartMessage(user_name));

    // here we can return the error as there's no work afterwards
    User::delete(claims.user_id(), &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

pub fn init_auth_routes(config: &mut web::ServiceConfig) {
    config
        .service(create_twitch_url)
        .service(twitch_callback)
        .service(revoke);
}
