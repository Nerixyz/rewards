use crate::services::{
    jwt::{decode_jwt, JwtClaims},
    spotify::{
        auth::{get_auth_url, SpotifyAuthResponse},
        requests::get_token,
    },
};
use actix_web::{
    cookie::CookieBuilder,
    delete, get, patch,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse, Result,
};
use errors::redirect_error::RedirectError;
use models::spotify::{SpotifyData, SpotifySettings};
use serde::Serialize;
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

#[derive(Serialize)]
struct ConnectionsList {
    spotify: Option<SpotifySettings>,
}

#[get("")]
async fn list_connections(claims: JwtClaims, pool: web::Data<PgPool>) -> Result<HttpResponse> {
    let spotify = SpotifySettings::by_id(claims.user_id(), &pool).await?;

    Ok(HttpResponse::Ok().json(ConnectionsList { spotify }))
}

#[get("/spotify-callback")]
async fn spotify_callback(
    query: web::Query<SpotifyAuthResponse>,
    pool: web::Data<PgPool>,
    raw: HttpRequest,
) -> Result<HttpResponse> {
    let (code, claims, cookie) = match query.into_inner() {
        SpotifyAuthResponse::Success { code, state } => {
            let cookie = raw
                .cookie("csrf")
                .ok_or_else(|| RedirectError::new("/failed-auth", Some("No csrf")))?;
            if cookie.value() != state {
                return Err(RedirectError::new("/failed-auth", Some("Invalid csrf")).into());
            }
            let claims = decode_jwt(&state)
                .map_err(|_| RedirectError::new("/failed-auth", Some("Invalid state")))?;
            (code, claims.claims, cookie)
        }
        SpotifyAuthResponse::Error { error, .. } => {
            return Err(RedirectError::new("/failed-auth", Some(error)).into());
        }
    };

    let auth_data = get_token(&code)
        .await
        .map_err(|_| RedirectError::new("/failed-auth", Some("Invalid code")))?;
    SpotifyData::add(
        claims.user_id(),
        &auth_data.access_token,
        &auth_data.refresh_token,
        &pool,
    )
    .await
    .map_err(|_| RedirectError::new("/failed-auth", Some("DB-Error")))?;

    let mut res = HttpResponse::Found()
        .insert_header(("location", "/connections"))
        .finish();
    res.add_removal_cookie(&cookie)?;
    Ok(res)
}

#[get("/spotify-auth-url")]
async fn spotify_auth(claims: JwtClaims) -> Result<HttpResponse> {
    let (url, jwt) = get_auth_url(claims.into_user_id()).map_err(|e| {
        log::warn!("Error creating auth-url: {}", e);

        errors::ErrorInternalServerError("Could not serialize")
    })?;

    Ok(HttpResponse::Ok()
        .cookie(
            CookieBuilder::new("csrf", jwt)
                .path("/api/v1/connections")
                .expires(OffsetDateTime::now_utc() + Duration::hours(2))
                .finish(),
        )
        .body(url))
}

#[patch("/spotify")]
async fn update_spotify_data(
    claims: JwtClaims,
    data: web::Json<SpotifySettings>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse> {
    data.0.save(claims.user_id(), &pool).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[delete("/spotify")]
async fn remove_spotify_data(claims: JwtClaims, pool: web::Data<PgPool>) -> Result<HttpResponse> {
    SpotifyData::remove_for_id(claims.user_id(), &pool).await?;

    Ok(HttpResponse::NoContent().finish())
}

pub fn init_connection_routes(config: &mut ServiceConfig) {
    config
        .service(spotify_auth)
        .service(spotify_callback)
        .service(list_connections)
        .service(remove_spotify_data)
        .service(update_spotify_data);
}
