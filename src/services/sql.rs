use crate::{
    models::{editor::Editor, user::User},
    services::{errors, errors::json_error::JsonError, jwt::JwtClaims},
};
use actix_web::http::StatusCode;
use serde::Serialize;
use sqlx::{Error, PgPool};

pub type SqlResult<T> = Result<T, JsonError<SqlReason>>;

#[derive(Debug, Serialize, derive_more::Display)]
pub enum SqlReason {
    #[display(fmt = "SQL: NotFound")]
    NotFound,
    #[display(fmt = "SQL: Internal")]
    Internal,
}

impl From<sqlx::Error> for JsonError<SqlReason> {
    fn from(e: Error) -> Self {
        match e {
            Error::RowNotFound | Error::TypeNotFound { .. } | Error::ColumnNotFound(_) => {
                log::warn!("NotFound sql-error: {}", e);
                Self::new(SqlReason::NotFound, StatusCode::NOT_FOUND)
            }
            _ => {
                log::warn!("Internal sql-error: {}", e);
                Self::new(SqlReason::Internal, StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub async fn get_user_or_editor(
    claims: &JwtClaims,
    broadcaster_id: &str,
    pool: &PgPool,
) -> Result<User, actix_web::Error> {
    let user = claims.get_user(&pool).await?;
    Ok(if user.id == broadcaster_id {
        user
    } else {
        Editor::get_broadcaster_for_editor(&user.id, &broadcaster_id, &pool)
            .await
            .map_err(|_| errors::ErrorForbidden("The user isn't an editor for the broadcaster."))?
    })
}
