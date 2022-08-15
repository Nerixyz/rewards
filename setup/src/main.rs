use chrono::{DateTime, Duration, Utc};
use dialoguer::{console::style, Confirm, Input, Password};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::Add};
use tokio_postgres::{types::Json, NoTls};
use twitch_oauth2::{
    tokens::UserTokenBuilder, url::Url, ClientId, ClientSecret, Scope, TwitchToken,
};

#[derive(Serialize, Debug)]
#[serde(tag = "type", content = "data")]
enum ConfigValue<'a> {
    UserToken(&'a UserAccessToken<'a>),
}

#[derive(Serialize, Debug)]
struct UserAccessToken<'a> {
    access_token: &'a str,
    refresh_token: &'a str,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[non_exhaustive]
struct SimpleMainConfig {
    twitch: SimpleTwitchConfig,
    db: SimpleDbConfig,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
struct SimpleTwitchConfig {
    client_id: String,
    client_secret: String,
}

#[derive(Deserialize)]
#[non_exhaustive]
struct SimpleDbConfig {
    url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let maybe_config = tokio::fs::read("config.toml")
        .await
        .ok()
        .and_then(|bytes| toml::from_slice::<SimpleMainConfig>(&bytes).ok());

    println!("Let's get some configuration upfront. If you run this in the main directory with a config.toml file this will be automatic.");

    let (client_id, client_secret, database_url) = match maybe_config {
        Some(config) => (
            config.twitch.client_id,
            config.twitch.client_secret,
            config.db.url,
        ),
        None => (
            Input::<String>::new()
                .with_prompt("ClientId")
                .interact_text()?,
            Password::new().with_prompt("ClientSecret").interact()?,
            Password::new().with_prompt("DatabaseUrl").interact()?,
        ),
    };

    let (pg, pg_connection) = tokio_postgres::connect(&database_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = pg_connection.await {
            eprintln!("Postgres connection error: {}", e);
        }
    });
    // language=PostgreSQL
    if pg
        .query_opt("SELECT key FROM config WHERE key = 'user_token'", &[])
        .await?
        .is_some()
    {
        // language=none
        if !Confirm::new().with_prompt("There's already a user-token in the database. Are you sure, you want to replace it?").default(false).interact()? {
            std::process::exit(1);
        }
    }

    let tw_client_id = ClientId::new(client_id);
    let tw_client_secret = ClientSecret::new(client_secret);

    println!(
        "SeemsGood. Make sure you have {} added as a redirect-url in the dev-console!\n",
        style("http://localhost").cyan()
    );
    println!(
        "Next, authenticate the app and paste either the {} or the {} here.",
        style("code").green(),
        style("full url").red()
    );

    let mut builder = UserTokenBuilder::new(
        tw_client_id,
        tw_client_secret,
        Url::parse("http://localhost")?,
    )
    .set_scopes(vec![
        Scope::ChatEdit,
        Scope::ChatRead,
        Scope::ChannelModerate,
        Scope::ModeratorManageAutoMod,
        Scope::ModerationRead,
        Scope::UserManageBlockedUsers,
        Scope::UserReadBlockedUsers,
        Scope::UserEditFollows,
        Scope::UserReadFollows,
        Scope::ChannelReadRedemptions,
        Scope::ChannelManageRedemptions,
        Scope::WhispersEdit,
        Scope::WhispersRead,
        Scope::ChannelEditCommercial,
        Scope::ChannelManageBroadcast,
    ]);
    let (url, csrf) = builder.generate_url();

    println!("Go to this url: {}\n", style(url.to_string()).cyan());

    let input = Input::<String>::new()
        .with_prompt("Code or Url")
        .interact()?;
    let code = if input.starts_with("http") {
        let url = Url::parse(&input)?;
        let pairs: HashMap<_, _> = url.query_pairs().collect();
        pairs.get("code").expect("Could not get code").to_string()
    } else {
        input
    };

    let token = builder
        .get_user_token(&reqwest::Client::new(), csrf.secret(), &code)
        .await?;

    let token_duration = Duration::from_std(token.expires_in()).expect("Should be in range");
    let refresh_token = token.refresh_token.expect("Could not get refresh_token");
    let token = UserAccessToken {
        access_token: token.access_token.secret(),
        refresh_token: refresh_token.secret(),
        created_at: Utc::now(),
        expires_at: Utc::now().add(token_duration),
    };
    let config_value = ConfigValue::UserToken(&token);

    // language=PostgreSQL
    pg.execute(
        "
            INSERT INTO config (key, value)
            VALUES ('user_token', $1)
             ON CONFLICT (key)
                 DO UPDATE SET value = $1
     ",
        &[&Json(config_value)],
    )
    .await?;

    println!("{}", style("Added your token to the database!").green());

    Ok(())
}
