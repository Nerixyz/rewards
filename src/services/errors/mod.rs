use actix_web::http::StatusCode;

pub mod json_error;
pub mod macros;
pub mod redirect_error;

// this is pretty much the actix code
macro_rules! error_helper {
    ($name:ident, $status:ident) => {
        paste::paste! {
            #[doc = "Helper function that wraps any error and generates a `" $status "` response."]
            #[allow(non_snake_case)]
            pub fn $name<T>(err: T) -> actix_web::Error
            where
            T: std::fmt::Debug + std::fmt::Display + serde::Serialize + 'static,
            {
                json_error::JsonError::new(err, StatusCode::$status).into()
            }
        }
    };
}

error_helper!(ErrorBadRequest, BAD_REQUEST);
error_helper!(ErrorUnauthorized, UNAUTHORIZED);
error_helper!(ErrorForbidden, FORBIDDEN);
error_helper!(ErrorInternalServerError, INTERNAL_SERVER_ERROR);
error_helper!(ErrorImATeapot, IM_A_TEAPOT);
// error_helper!(ErrorNotFound, NOT_FOUND);
// error_helper!(ErrorNotImplemented, NOT_IMPLEMENTED);
