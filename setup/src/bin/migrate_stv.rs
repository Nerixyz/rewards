use std::collections::HashSet;

use anyhow::Result as AnyResult;
use serde::Deserialize;
use tokio_postgres::NoTls;

#[derive(Deserialize)]
#[non_exhaustive]
struct SimpleMainConfig {
    db: SimpleDbConfig,
}

#[derive(Deserialize)]
#[non_exhaustive]
struct SimpleDbConfig {
    url: String,
}

#[derive(Deserialize)]
#[non_exhaustive]
struct JustAnId {
    id: String,
}

async fn resolve_id(id: &str) -> AnyResult<String> {
    Ok(reqwest::get(format!("https://7tv.io/v3/emotes/{id}"))
        .await?
        .json::<JustAnId>()
        .await?
        .id)
}

async fn fix_table(
    pg: &mut tokio_postgres::Client,
    name: &str,
) -> AnyResult<()> {
    eprintln!("-> table: {name}");
    let rows = pg.query(&format!("select emote_id from {name} where platform = '7tv' and length(emote_id) = 24"), &[]).await?;
    let ids = rows
        .iter()
        .map(|r| r.get::<_, &str>(0))
        .collect::<HashSet<_>>();

    let mut resolved = Vec::with_capacity(ids.len());
    for id in ids {
        eprintln!("{id}");
        match resolve_id(id).await {
            Ok(res) => resolved.push((id, res)),
            Err(e) => eprintln!("Failed to resolve {id} - {e}"),
        }
    }

    eprintln!("applying...");
    let tx = pg.transaction().await?;
    for (id, resolved) in resolved {
        tx.execute(&format!("update {name} set emote_id = $2 where platform = '7tv' and emote_id = $1"), &[&id, &resolved]).await?;
    }
    tx.commit().await?;
    eprintln!("<- done");

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> AnyResult<()> {
    dotenv::dotenv().ok();
    let config = toml::from_str::<SimpleMainConfig>(
        &tokio::fs::read_to_string("config.toml").await?,
    )?;

    eprintln!("connecting to db...");
    let (mut pg, pg_connection) =
        tokio_postgres::connect(&config.db.url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = pg_connection.await {
            eprintln!("Postgres connection error: {}", e);
        }
    });
    eprintln!("connected");

    for db in ["banned_emotes", "slots", "swap_emotes"] {
        if let Err(e) = fix_table(&mut pg, db).await {
            eprintln!("failed to update {db}: {e}");
        }
    }

    Ok(())
}
