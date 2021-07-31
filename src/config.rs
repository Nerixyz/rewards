use anyhow::Result as AnyResult;
use lazy_static::lazy_static;
use serde::Deserialize;
use sqlx::postgres::PgConnectOptions;
use std::{convert::TryFrom, str::FromStr};

#[derive(Deserialize)]
pub struct Config {
    pub db: DbConfig,
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub twitch: TwitchConfig,
    pub emotes: EmoteConfig,
    pub spotify: SpotifyConfig,
}

#[derive(Deserialize)]
pub struct DbConfig {
    pub url: String,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub url: String,
    pub port: u16,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AuthConfig {
    pub jwt_secret: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TwitchConfig {
    pub client_id: String,
    pub client_secret: String,
    pub login: String,
    pub user_id: String,
    pub eventsub: EventSubConfig,
}

#[derive(Deserialize)]
pub struct EventSubConfig {
    pub secret: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct EmoteConfig {
    pub bttv: BttvConfig,
    pub ffz: FfzConfig,
    pub seven_tv: SevenTvConfig,
}

#[derive(Deserialize)]
pub struct BttvConfig {
    pub jwt: String,
}

#[derive(Deserialize)]
pub struct FfzConfig {
    pub session: String,
    pub remember: String,
}

#[derive(Deserialize)]
pub struct SevenTvConfig {
    pub jwt: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SpotifyConfig {
    pub client_id: String,
    pub client_secret: String,
}

impl TryFrom<&DbConfig> for PgConnectOptions {
    type Error = <Self as FromStr>::Err;

    fn try_from(c: &DbConfig) -> Result<Self, Self::Error> {
        Self::from_str(&c.url)
    }
}

lazy_static! {
    pub static ref CONFIG: Config = read_config().unwrap();
}

/// This blocks!
fn read_config() -> AnyResult<Config> {
    let config = toml::from_slice(&std::fs::read("config.toml")?)?;
    Ok(config)
}
