use crate::constants::{TWITCH_CLIENT_ID, TWITCH_CLIENT_SECRET};
use crate::services::sql::SqlError;
use sqlx::{FromRow, PgPool};
use std::time::Duration;
use twitch_api2::twitch_oauth2::{AccessToken, ClientId, ClientSecret, RefreshToken, UserToken};

#[derive(FromRow)]
pub struct User {
    pub id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub scopes: String,
    pub name: String,
    pub eventsub_id: Option<String>,
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

    pub async fn get_all_names(pool: &PgPool) -> Result<Vec<String>, SqlError> {
        let names = sqlx::query_scalar!(
            // language=PostgreSQL
            r#"
                SELECT name
                FROM users
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(names)
    }

    pub async fn get_all_non_subscribers(pool: &PgPool) -> Result<Vec<String>, SqlError> {
        // language=PostgreSQL
        let ids = sqlx::query_scalar!("SELECT id FROM users WHERE eventsub_id IS null")
            .fetch_all(pool)
            .await?;

        Ok(ids)
    }

    pub async fn create(&self, pool: &PgPool) -> Result<(), SqlError> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!(
            r#"
            INSERT
            INTO users (id, access_token, refresh_token, scopes, name)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT(id)
                DO UPDATE SET access_token= $2, refresh_token=$3
                "#,
            self.id,
            self.access_token,
            self.refresh_token,
            self.scopes,
            self.name
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn update_refresh(
        id: &str,
        access_token: &str,
        refresh_token: &str,
        pool: &PgPool,
    ) -> Result<(), SqlError> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!(
            r#"
            UPDATE users
            SET access_token = $2, refresh_token = $3
            WHERE id = $1
            "#,
            id,
            access_token,
            refresh_token
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn delete(id: &str, pool: &PgPool) -> Result<(), SqlError> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!(
            r#"
            DELETE FROM users WHERE id = $1
                "#,
            id
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn set_eventsub_id(
        user_id: &str,
        eventsub_id: &str,
        pool: &PgPool,
    ) -> Result<(), SqlError> {
        // language=PostgreSQL
        let _ = sqlx::query!(
            r#"
            UPDATE users
            SET eventsub_id = $2
            WHERE id = $1
            "#,
            user_id,
            eventsub_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn clear_eventsub_id(eventsub_id: &str, pool: &PgPool) -> Result<(), SqlError> {
        // language=PostgreSQL
        let _ = sqlx::query!(
            r#"
            UPDATE users
            SET eventsub_id = null
            WHERE eventsub_id = $1
            "#,
            eventsub_id
        )
        .execute(pool)
        .await?;
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
            Some(Duration::from_secs(1000)),
        )
    }
}
