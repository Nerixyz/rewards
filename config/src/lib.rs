use anyhow::Result as AnyResult;
use lazy_static::lazy_static;
use serde::Deserialize;
use sqlx::postgres::PgConnectOptions;
use std::collections::HashMap;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    // keep in sync with /setup
    pub db: DbConfig,
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub auth: AuthConfig,
    pub twitch: TwitchConfig,
    pub emotes: EmoteConfig,
    pub spotify: SpotifyConfig,
    pub bot: BotConfig,
    #[serde(default)]
    pub log: LogConfig,
    #[serde(default)]
    pub supinic: Option<SupinicConfig>,
    pub owner: OwnerConfig,
    #[serde(default)]
    pub debug_overrides: DebugOverrides,
}

#[derive(Deserialize)]
pub struct DbConfig {
    // keep in sync with /setup
    pub url: String,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub url: String,
    #[serde(default = "default_bind_addr")]
    pub bind_addr: String,
}

fn default_bind_addr() -> String {
    "127.0.0.1:8082".to_string()
}

#[derive(Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AuthConfig {
    pub jwt_secret: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TwitchConfig {
    // keep in sync with /setup
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
#[serde(rename_all = "kebab-case")]
pub struct SevenTvConfig {
    pub jwt: String,
    pub user_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SpotifyConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Deserialize)]
pub struct BotConfig {
    #[serde(default = "default_prefix")]
    pub prefix: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "kebab-case", default)]
pub struct LogConfig {
    pub webhook_url: Option<String>,
    pub announce: Option<AnnounceConfig>,
}

fn default_prefix() -> String {
    "::".to_string()
}

#[derive(Deserialize, Default)]
pub struct AnnounceConfig {
    pub discord: bool,
    pub twitch: Option<AnnounceTwitchConfig>,
}

#[derive(Deserialize)]
pub struct AnnounceTwitchConfig {
    pub channel: String,
    #[serde(default)]
    pub prefix: Option<String>,
}

#[derive(Deserialize)]
pub struct SupinicConfig {
    pub id: u64,
    pub key: String,
}

#[derive(Deserialize)]
pub struct OwnerConfig {
    pub id: String,
    pub username: String,
}

#[derive(Deserialize, Default)]
pub struct DebugOverrides {
    #[serde(default)]
    pub seventv: HashMap<String, String>,
    #[serde(default)]
    pub twitch: HashMap<String, String>,
}

impl DebugOverrides {
    pub fn seventv<'a>(&'a self, id: &'a str) -> &'a str {
        self.seventv.get(id).map(|k| k.as_str()).unwrap_or(id)
    }

    pub fn twitch<'a>(&'a self, id: &'a str) -> &'a str {
        self.twitch.get(id).map(|k| k.as_str()).unwrap_or(id)
    }
}

lazy_static! {
    pub static ref CONFIG: Config = read_config().unwrap();
}

/// This blocks!
fn read_config() -> AnyResult<Config> {
    let config = toml::from_str(&std::fs::read_to_string("config.toml")?)?;
    Ok(config)
}
