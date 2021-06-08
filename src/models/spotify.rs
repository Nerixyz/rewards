use crate::services::spotify::responses::{AccessTokenResponse, RefreshTokenResponse};
use crate::services::sql::SqlResult;
use sqlx::{FromRow, PgPool};

#[derive(FromRow)]
pub struct SpotifyData {
    pub user_id: String,
    pub access_token: String,
    pub refresh_token: String,
}

impl SpotifyData {
    pub async fn get_by_id(id: &str, pool: &PgPool) -> SqlResult<Option<Self>> {
        // language=PostgreSQL
        let data = sqlx::query_as!(Self, r#"SELECT * FROM spotify WHERE user_id=$1"#, id)
            .fetch_optional(pool)
            .await?;

        Ok(data)
    }

    pub async fn get_all(pool: &PgPool) -> SqlResult<Vec<Self>> {
        // language=PostgreSQL
        let data = sqlx::query_as!(Self, r#"SELECT * FROM spotify"#)
            .fetch_all(pool)
            .await?;

        Ok(data)
    }

    pub async fn add(id: &str, token: &AccessTokenResponse, pool: &PgPool) -> SqlResult<()> {
        // language=PostgreSQL
        sqlx::query!(
            r#"
            INSERT INTO spotify (user_id, access_token, refresh_token)
            VALUES ($1, $2, $3)
            "#,
            id,
            token.access_token,
            token.refresh_token
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn update_token(
        id: &str,
        token: &RefreshTokenResponse,
        pool: &PgPool,
    ) -> SqlResult<()> {
        // language=PostgreSQL
        sqlx::query!(
            r#"
            UPDATE spotify
            SET access_token=$2
            WHERE user_id=$1
            "#,
            id,
            token.access_token
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn remove_for_id(id: &str, pool: &PgPool) -> SqlResult<()> {
        // language=PostgreSQL
        sqlx::query!(r#"DELETE FROM spotify WHERE user_id = $1"#, id,)
            .execute(pool)
            .await?;

        Ok(())
    }
}
