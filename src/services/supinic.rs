use anyhow::{Error as AnyError, Result as AnyResult};
use config::CONFIG;
use lazy_static::lazy_static;
use reqwest::{
    header::{HeaderMap, AUTHORIZATION},
    Client,
};

lazy_static! {
    static ref SUPINIC_CLIENT: Client = Client::builder()
        .user_agent(format!(
            "RewardMore/{} github.com/Nerixyz/rewards",
            env!("CARGO_PKG_VERSION")
        ))
        .default_headers({
            let mut map = HeaderMap::with_capacity(1);
            if let Some(supi) = CONFIG.supinic.as_ref() {
                map.insert(
                    AUTHORIZATION,
                    format!("Basic {}:{}", supi.id, supi.key).parse().unwrap(),
                );
            }
            map
        })
        .build()
        .unwrap();
}

pub async fn update_activity() -> AnyResult<()> {
    SUPINIC_CLIENT
        .put("https://supinic.com/api/bot-program/bot/active")
        .send()
        .await
        .map_err(AnyError::from)
        .and_then(|res| {
            if res.status().is_success() {
                Ok(())
            } else {
                Err(AnyError::msg(format!(
                    "Bad response - status={}",
                    res.status()
                )))
            }
        })
}
