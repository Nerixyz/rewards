use sqlx::{FromRow, PgPool};
use crate::services::sql::SqlError;
use twitch_api2::twitch_oauth2::{UserToken, AccessToken, RefreshToken, ClientId, ClientSecret};
use std::time::Duration;
use crate::constants::{TWITCH_CLIENT_ID, TWITCH_CLIENT_SECRET};


#[derive(FromRow)]
pub struct User {
    pub id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub scopes: String,
    pub name: String
}

impl User {
    pub async fn get_by_id(id: &str, pool: &PgPool) -> Result<User, SqlError> {
        // language=PostgreSQL
        let user: User = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
            .fetch_one(pool)
            .await?;

        Ok(user)
    }

    pub async fn get_all(pool: &PgPool) -> Result<Vec<User>, SqlError> {
        // language=PostgreSQL
        let users = sqlx::query_as!(User, "SELECT * FROM users")
            .fetch_all(pool)
            .await?;

        Ok(users)
    }

    pub async fn create(&self, pool: &PgPool) -> Result<(), SqlError> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!(r#"
            INSERT
            INTO users (id, access_token, refresh_token, scopes, name)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT(id)
                DO UPDATE SET access_token= $2, refresh_token=$3
                "#, self.id, self.access_token, self.refresh_token, self.scopes, self.name)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn update_refresh(id: &str, access_token: &str, refresh_token: &str, pool: &PgPool) -> Result<(), SqlError> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!(r#"
            UPDATE users
            SET access_token = $2, refresh_token = $3
            WHERE id = $1
            "#, id, access_token, refresh_token)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn delete(id: &str, pool: &PgPool) -> Result<(), SqlError> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!(r#"
            DELETE FROM users WHERE id = $1
                "#, id)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }
}

impl Into<UserToken> for User {
    fn into(self) -> UserToken {
        UserToken::from_existing_unchecked(
            AccessToken::new(self.access_token),
            RefreshToken::new(self.refresh_token),
            ClientId::new(TWITCH_CLIENT_ID.to_string()),
            ClientSecret::new(TWITCH_CLIENT_SECRET.to_string()),
            String::new(),
            self.id,
            None,
            // this isn't used anywhere
            Some(Duration::from_secs(1000))
        )
    }
}
