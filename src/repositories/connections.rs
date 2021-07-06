use crate::models::spotify::SpotifyData;
use crate::services::errors;
use crate::services::errors::redirect_error::RedirectError;
use crate::services::jwt::{decode_jwt, JwtClaims};
use crate::services::spotify::auth::{get_auth_url, SpotifyAuthResponse};
use crate::services::spotify::requests::get_token;
use actix_web::cookie::CookieBuilder;
use actix_web::{
    delete, get,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse, Result,
};
use serde::Serialize;
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

#[derive(Serialize)]
struct ConnectionsList {
    spotify: bool,
}

#[get("")]
async fn list_connections(claims: JwtClaims, pool: web::Data<PgPool>) -> Result<HttpResponse> {
    let spotify = SpotifyData::get_by_id(claims.user_id(), &pool).await?;

    Ok(HttpResponse::Ok().json(ConnectionsList {
        spotify: spotify.is_some(),
    }))
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
    SpotifyData::add(claims.user_id(), &auth_data, &pool)
        .await
        .map_err(|_| RedirectError::new("/failed-auth", Some("DB-Error")))?;

    Ok(HttpResponse::Found()
        .insert_header(("location", "/connections"))
        .del_cookie(&cookie)
        .finish())
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
        .service(remove_spotify_data);
}
