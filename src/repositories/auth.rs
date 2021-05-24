use actix_web::{web, HttpResponse, error, BaseHttpResponse, get, Error, delete};
use crate::constants::{TWITCH_CLIENT_ID, SERVER_URL, TWITCH_CLIENT_SECRET};
use twitch_api2::twitch_oauth2::{Scope, ClientId, ClientSecret, RedirectUrl, CsrfToken, TwitchToken, UserToken};
use itertools::Itertools;
use actix_web::body::Body;
use actix_web::http::{StatusCode, header};
use sqlx::PgPool;
use twitch_api2::twitch_oauth2::tokens::UserTokenBuilder;
use twitch_api2::twitch_oauth2::client::reqwest_http_client;
use crate::models::user::User;
use crate::services::jwt::{encode_jwt, JwtClaims};
use actix_web::cookie::CookieBuilder;
use time::{OffsetDateTime, Duration};
use serde::{Deserialize, Serialize};

#[derive(Debug, derive_more::Display, derive_more::Error)]
#[display(fmt = "Error during oauth authorization")]
struct OAuthError;

impl error::ResponseError for OAuthError {
    fn error_response(&self) -> BaseHttpResponse<Body> {
        // HttpResponse::MovedPermanently().append_header((header::LOCATION, "/failed-auth")).finish().into_body();

        let mut resp = BaseHttpResponse::new(StatusCode::MOVED_PERMANENTLY);
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
async fn twitch_callback(pool: web::Data<PgPool>, query: web::Query<TwitchCallbackQuery>) -> Result<HttpResponse, Error> {
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
        name: user_token.login.clone()
    };

    user.create(&pool).await.map_err(|_| OAuthError)?;

    let token = encode_jwt(&JwtClaims::new(user_token.user_id.clone())).map_err(|_| OAuthError)?;
    Ok(HttpResponse::MovedPermanently()
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
async fn revoke(claims: JwtClaims, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    let user = claims.get_user(&pool).await?;
    let token: UserToken = user.into();
    token.revoke_token(reqwest_http_client).await.map_err(|_| error::ErrorInternalServerError(""))?;
    User::delete(claims.user_id(), &pool).await?;

    Ok(HttpResponse::Ok().finish())
}


pub fn init_auth_routes(config: &mut web::ServiceConfig) {
    config
        .service(create_twitch_url)
        .service(twitch_callback)
        .service(revoke);
}