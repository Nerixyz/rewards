use crate::models::editor::Editor;
use crate::models::user::User;
use crate::services::jwt::JwtClaims;
use actix_web::error;
use actix_web::http::StatusCode;
use sqlx::{Error, PgPool};

#[derive(Debug, derive_more::Error, derive_more::Display)]
pub enum SqlError {
    #[display(fmt = "NotFound")]
    NotFound,
    #[display(fmt = "Internal")]
    Internal,
}

impl From<sqlx::Error> for SqlError {
    fn from(e: Error) -> Self {
        match e {
            Error::RowNotFound | Error::TypeNotFound { .. } | Error::ColumnNotFound(_) => {
                log::warn!("NotFound sql-error: {}", e);
                Self::NotFound
            }
            _ => {
                log::warn!("Internal sql-error: {}", e);
                Self::Internal
            }
        }
    }
}

impl error::ResponseError for SqlError {
    fn status_code(&self) -> StatusCode {
        match self {
            SqlError::NotFound => StatusCode::NOT_FOUND,
            SqlError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
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
            .map_err(|_| error::ErrorForbidden("The user isn't an editor for the broadcaster."))?
    })
}
