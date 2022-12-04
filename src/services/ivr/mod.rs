use anyhow::{anyhow, Result as AnyResult};
use lazy_static::lazy_static;
use reqwest::{Client, IntoUrl};
use serde::{de::DeserializeOwned, Deserialize};
use std::time::Duration;

lazy_static! {
    static ref IVR_CLIENT: Client = Client::builder()
        .user_agent(format!(
            "RewardMore/{} github.com/Nerixyz/rewards",
            env!("CARGO_PKG_VERSION")
        ))
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap();
}

#[derive(Deserialize)]
#[non_exhaustive]
pub struct ModVips {
    pub vips: Vec<User>,
}

#[derive(Deserialize)]
#[non_exhaustive]
pub struct User {
    pub id: String,
}

#[derive(Deserialize)]
#[non_exhaustive]
struct IvrErrorResponse {
    error: IvrError,
}

#[derive(Deserialize)]
#[non_exhaustive]
struct IvrError {
    message: String,
}

pub async fn modvips(username: &str) -> AnyResult<ModVips> {
    ivr_get(format!("https://api.ivr.fi/v2/twitch/modvip/{username}")).await
}

async fn ivr_get<R>(url: impl IntoUrl) -> AnyResult<R>
where
    R: DeserializeOwned,
{
    let response = IVR_CLIENT.get(url).send().await?;
    let status = response.status();
    match status {
        s if s.is_success() => Ok(response.json().await?),
        _ => {
            let json: IvrErrorResponse = response.json().await?;
            Err(anyhow!(
                "IVR Error - {} (http-status={})",
                json.error.message,
                status
            ))
        }
    }
}
