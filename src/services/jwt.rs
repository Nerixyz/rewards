use serde::{Deserialize, Serialize};
use jsonwebtoken::{DecodingKey, EncodingKey, decode, TokenData, Validation, Algorithm, encode, Header};
use time::{OffsetDateTime, Duration};
use crate::constants::JWT_BASE64_SECRET;
use sqlx::PgPool;
use crate::models::user::User;
use actix_web::{Error, error};

#[derive(Serialize, Deserialize, Debug)]
pub struct JwtClaims {
    exp: usize,
    user_id: String,
}

impl JwtClaims {
    pub fn new(user_id: String) -> Self {
        Self {
            exp: (OffsetDateTime::now_utc() + Duration::days(365)).unix_timestamp() as usize,
            user_id
        }
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub async fn get_user(&self, pool: &PgPool) -> Result<User, Error> {
        User::get_by_id(self.user_id(), pool).await.map_err(|_| error::ErrorUnauthorized(""))
    }
}

pub fn decode_jwt(token: &str) -> jsonwebtoken::errors::Result<TokenData<JwtClaims>> {
    decode::<JwtClaims>(token, &DecodingKey::from_base64_secret(JWT_BASE64_SECRET).expect("invalid key"), &Validation::new(Algorithm::HS256))
}

pub fn encode_jwt(claims: &JwtClaims) -> jsonwebtoken::errors::Result<String> {
    encode(&Header::new(Algorithm::HS256), claims, &EncodingKey::from_base64_secret(JWT_BASE64_SECRET).expect("invalid key"))
}