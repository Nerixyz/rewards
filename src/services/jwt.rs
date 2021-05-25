use crate::constants::JWT_BASE64_SECRET;
use crate::models::user::User;
use actix_web::{error, Error};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

#[derive(Serialize, Deserialize, Debug)]
pub struct JwtClaims {
    exp: usize,
    user_id: String,
}

impl JwtClaims {
    pub fn new(user_id: String) -> Self {
        Self {
            exp: (OffsetDateTime::now_utc() + Duration::days(365)).unix_timestamp() as usize,
            user_id,
        }
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub async fn get_user(&self, pool: &PgPool) -> Result<User, Error> {
        User::get_by_id(self.user_id(), pool)
            .await
            .map_err(|_| error::ErrorUnauthorized(""))
    }
}

pub fn decode_jwt(token: &str) -> jsonwebtoken::errors::Result<TokenData<JwtClaims>> {
    decode::<JwtClaims>(
        token,
        &DecodingKey::from_base64_secret(JWT_BASE64_SECRET).expect("invalid key"),
        &Validation::new(Algorithm::HS256),
    )
}

pub fn encode_jwt(claims: &JwtClaims) -> jsonwebtoken::errors::Result<String> {
    encode(
        &Header::new(Algorithm::HS256),
        claims,
        &EncodingKey::from_base64_secret(JWT_BASE64_SECRET).expect("invalid key"),
    )
}
