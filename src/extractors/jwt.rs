use crate::services::jwt::{decode_jwt, JwtClaims};
use actix_web::{
    dev::Payload, http::header, Error, FromRequest, HttpRequest, Result,
};
use futures_util::future::{err, ready, Ready};

impl FromRequest for JwtClaims {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let auth: Result<&str> = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| errors::ErrorUnauthorized("No header"));

        let auth = match auth {
            Ok(auth) => auth,
            Err(e) => return err(e),
        };

        if auth.len() <= 7 {
            return err(errors::ErrorUnauthorized("Bad header value"));
        }

        let (head, token) = auth.split_at(7);
        if head != "Bearer " {
            return err(errors::ErrorUnauthorized("Bad header value"));
        }

        ready(
            decode_jwt(token)
                .map_err(|_| errors::ErrorUnauthorized("Bad token"))
                .map(|t| t.claims),
        )
    }
}
