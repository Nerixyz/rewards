use errors::sql::SqlResult;
use sqlx::PgPool;
use twitch_api::types::EventSubIdRef;

pub async fn add(
    id: &EventSubIdRef,
    user_id: &str,
    name: &str,
    pool: &PgPool,
) -> SqlResult<()> {
    sqlx::query_scalar!(
        "
            INSERT INTO eventsubs (id, user_id, name) VALUES ($1, $2, $3) 
                ON CONFLICT (user_id, name) 
                    DO UPDATE SET id = $1
        ",
        id.as_str(),
        user_id,
        name
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn remove(id: &EventSubIdRef, pool: &PgPool) -> SqlResult<()> {
    sqlx::query_scalar!("DELETE FROM eventsubs WHERE id = $1", id.as_str(),)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn all_for_user(
    user_id: &str,
    pool: &PgPool,
) -> SqlResult<Vec<String>> {
    sqlx::query_scalar!(
        "
            select id from eventsubs where user_id = $1
        ",
        user_id,
    )
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}
