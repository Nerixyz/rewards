use actix_web::dev::Payload;
use actix_web::{http::header, Error, FromRequest, HttpRequest, error};
use futures_util::future::{err, Ready, ready};
use crate::services::jwt::{JwtClaims, decode_jwt};

impl FromRequest for JwtClaims {
    type Config = ();
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let auth: Result<&str, Error> = req
            .headers()
            .get(header::AUTHORIZATION).map(|h| h.to_str().ok()).flatten().ok_or(error::ErrorUnauthorized(""));

        let auth = match auth {
            Ok(auth) => auth,
            Err(e) => return err(e)
        };


        let (head, token) = auth.split_at(7);
        if head != "Bearer " {
            return err(error::ErrorUnauthorized(""));
        }

        ready(decode_jwt(token).map_err(|_| error::ErrorUnauthorized("")).map(|t| t.claims))
    }
}