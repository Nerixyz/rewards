use crate::services::jwt::JwtClaims;
use models::{editor::Editor, user::User};
use sqlx::PgPool;

pub async fn get_user_or_editor(
    claims: &JwtClaims,
    broadcaster_id: &str,
    pool: &PgPool,
) -> Result<User, actix_web::Error> {
    let user = claims.get_user(pool).await?;
    Ok(if user.id == broadcaster_id {
        user
    } else {
        Editor::get_broadcaster_for_editor(&user.id, broadcaster_id, pool)
            .await
            .map_err(|_| errors::ErrorForbidden("The user isn't an editor for the broadcaster."))?
    })
}
