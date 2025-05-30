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

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct ModVips {
    #[serde(default)]
    pub vips: Vec<User>,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct User {
    pub id: String,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
struct IvrErrorResponse {
    error: IvrError,
}

#[derive(Deserialize, Debug)]
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
        _ => match response.json::<IvrErrorResponse>().await {
            Ok(json) => {
                Err(anyhow!("IVR Error - {} ({status})", json.error.message))
            }
            Err(_) => Err(anyhow!(
                "IVR error {status} + failed to decode error response"
            )),
        },
    }
}
