use config::CONFIG;
use errors::sql::SqlResult;
use sqlx::{FromRow, PgPool};
use std::time::Duration;
use twitch_api::{
    twitch_oauth2::{
        AccessToken, ClientId, ClientSecret, RefreshToken, UserToken,
    },
    types::{Nickname, UserId},
};

#[derive(FromRow, Clone)]
pub struct User {
    pub id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub scopes: String,
    pub name: String,
}

#[derive(FromRow)]
pub struct UserBttvData {
    pub id: String,
    pub bttv_id: Option<String>,
}

#[derive(FromRow)]
pub struct UserSevenTvData {
    pub id: String,
    pub name: String,
    pub seventv_id: Option<String>,
}

impl User {
    pub async fn get_by_id(id: &str, pool: &PgPool) -> SqlResult<User> {
        // language=PostgreSQL
        let user: User = sqlx::query_as!(User, "SELECT id, access_token, refresh_token, scopes, name FROM users WHERE id = $1", id)
            .fetch_one(pool)
            .await?;

        Ok(user)
    }

    pub async fn get_all(pool: &PgPool) -> SqlResult<Vec<User>> {
        // language=PostgreSQL
        let users = sqlx::query_as!(
            User,
            "SELECT id, access_token, refresh_token, scopes, name FROM users"
        )
        .fetch_all(pool)
        .await?;

        Ok(users)
    }

    pub async fn get_all_names(pool: &PgPool) -> SqlResult<Vec<String>> {
        let names = sqlx::query_scalar!(
            // language=PostgreSQL
            "SELECT name FROM users"
        )
        .fetch_all(pool)
        .await?;

        Ok(names)
    }

    pub async fn get_all_non_subscribers(
        pool: &PgPool,
        name: &str,
    ) -> SqlResult<Vec<String>> {
        // language=PostgreSQL
        let ids = sqlx::query_scalar!(
            "
                SELECT users.id FROM users
                LEFT JOIN eventsubs e
                    ON users.id = e.user_id
                WHERE e.user_id is null AND e.name = $1
            ",
            name
        )
        .fetch_all(pool)
        .await?;

        Ok(ids)
    }

    pub async fn get_bttv_data(
        user_id: &str,
        pool: &PgPool,
    ) -> SqlResult<UserBttvData> {
        // language=PostgreSQL
        let data = sqlx::query_as!(
            UserBttvData,
            "SELECT id, bttv_id FROM users WHERE id = $1",
            user_id
        )
        .fetch_one(pool)
        .await?;

        Ok(data)
    }

    pub async fn get_seventv_data(
        user_id: &str,
        pool: &PgPool,
    ) -> SqlResult<UserSevenTvData> {
        // language=PostgreSQL
        let data = sqlx::query_as!(
            UserSevenTvData,
            "SELECT id, name, seventv_id FROM users WHERE id = $1",
            user_id
        )
        .fetch_one(pool)
        .await?;

        Ok(data)
    }

    pub async fn create(&self, pool: &PgPool) -> SqlResult<()> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!(
            "
            INSERT
            INTO users (id, access_token, refresh_token, scopes, name)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT(id)
                DO UPDATE SET access_token= $2, refresh_token=$3
                ",
            self.id,
            self.access_token,
            self.refresh_token,
            self.scopes,
            self.name
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn update_refresh(
        id: impl AsRef<str>,
        access_token: &str,
        refresh_token: &str,
        pool: &PgPool,
    ) -> SqlResult<()> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!(
            "UPDATE users SET access_token = $2, refresh_token = $3 WHERE id = $1",
            id.as_ref(),
            access_token,
            refresh_token
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn set_bttv_id(
        user_id: &str,
        bttv_id: &str,
        pool: &PgPool,
    ) -> SqlResult<()> {
        // language=PostgreSQL
        let _ = sqlx::query_scalar!(
            "UPDATE users SET bttv_id = $2 WHERE id = $1",
            user_id,
            bttv_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn set_seventv_id(
        user_id: &str,
        seventv_id: &str,
        pool: &PgPool,
    ) -> SqlResult<()> {
        // language=PostgreSQL
        let _ = sqlx::query_scalar!(
            "UPDATE users SET seventv_id = $2 WHERE id = $1",
            user_id,
            seventv_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(id: &str, pool: &PgPool) -> SqlResult<()> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!("DELETE FROM users WHERE id = $1", id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }
}

impl From<User> for UserToken {
    fn from(u: User) -> Self {
        Self::from_existing_unchecked(
            AccessToken::new(u.access_token),
            RefreshToken::new(u.refresh_token),
            ClientId::new(CONFIG.twitch.client_id.to_string()),
            ClientSecret::new(CONFIG.twitch.client_secret.to_string()),
            Nickname::from(u.name),
            UserId::from(u.id),
            None,
            // this isn't used anywhere
            Some(Duration::from_secs(1000)),
        )
    }
}
