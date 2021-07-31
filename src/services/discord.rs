use crate::config::CONFIG;
use anyhow::{Error as AnyError, Result as AnyResult};
use lazy_static::lazy_static;
use reqwest::Client;
use serde::Serialize;
use std::borrow::Cow;

lazy_static! {
    static ref DISCORD_CLIENT: Client = Client::builder()
        .user_agent(format!(
            "RewardMore/{} github.com/Nerixyz/rewards",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .unwrap();
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookReq {
    Content(String),
    Embeds(Vec<Embed>),
}

#[derive(Serialize)]
pub struct Embed {
    pub title: Cow<'static, str>,
    pub description: String,
    pub fields: Vec<EmbedField>,
    pub color: u32,
}

#[derive(Serialize)]
pub struct EmbedField {
    name: String,
    value: String,
    inline: bool,
}

impl EmbedField {
    pub fn new<N, V>(name: N, value: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        Self {
            name: name.into(),
            value: value.into(),
            inline: true,
        }
    }
}

pub async fn send_webhook_message(req: &WebhookReq) -> AnyResult<()> {
    if let Some(ref url) = CONFIG.log.webhook_url {
        let res = DISCORD_CLIENT.post(url).json(req).send().await?;
        let status = res.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(AnyError::msg(
                res.text()
                    .await
                    .unwrap_or_else(|_| "Bad response".to_string()),
            ))
        }
    } else {
        Ok(())
    }
}
