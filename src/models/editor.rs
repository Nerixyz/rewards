use crate::models::user::User;
use errors::sql::SqlResult;
use sqlx::PgPool;

pub struct Editor;

impl Editor {
    pub async fn get_editors(broadcaster_id: &str, pool: &PgPool) -> SqlResult<Vec<String>> {
        let editors = sqlx::query_scalar!(
            // language=PostgreSQL
            "
                SELECT editor_id
                FROM editors
                WHERE broadcaster_id = $1
            ",
            broadcaster_id
        )
        .fetch_all(pool)
        .await?;

        Ok(editors)
    }

    pub async fn get_broadcasters(editor_id: &str, pool: &PgPool) -> SqlResult<Vec<String>> {
        let broadcasters = sqlx::query_scalar!(
            // language=PostgreSQL
            "
                SELECT broadcaster_id
                FROM editors
                WHERE editor_id = $1
            ",
            editor_id
        )
        .fetch_all(pool)
        .await?;

        Ok(broadcasters)
    }

    pub async fn get_broadcaster_for_editor(
        editor_id: &str,
        broadcaster_id: &str,
        pool: &PgPool,
    ) -> SqlResult<User> {
        let user = sqlx::query_as!(
            User,
            // language=PostgreSQL
            "
                SELECT u.id, access_token, refresh_token, scopes, name, eventsub_id
                FROM editors
                    LEFT JOIN users u on u.id = editors.broadcaster_id
                WHERE broadcaster_id = $2 and editor_id = $1
            ",
            editor_id,
            broadcaster_id,
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn add_editor(
        broadcaster_id: &str,
        editor_name: &str,
        pool: &PgPool,
    ) -> SqlResult<()> {
        let _ = sqlx::query!(
            // language=PostgreSQL
            "
            INSERT INTO editors
                (editor_id, broadcaster_id)
             VALUES
                    ((SELECT id from users WHERE name = $1), $2)
            ",
            editor_name,
            broadcaster_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn delete_editor(
        broadcaster_id: &str,
        editor_name: &str,
        pool: &PgPool,
    ) -> SqlResult<()> {
        let mut tx = pool.begin().await?;
        let _ = sqlx::query!(
            // language=PostgreSQL
            "
            DELETE FROM editors
                   WHERE editor_id = (SELECT id from users WHERE name = $2) AND broadcaster_id = $1
            ",
            broadcaster_id,
            editor_name
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
