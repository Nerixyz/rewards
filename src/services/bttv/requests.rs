use crate::constants::BTTV_JWT;
use anyhow::Result as AnyResult;
use reqwest::IntoUrl;
use serde::de::DeserializeOwned;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BttvEditor {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    pub limits: BttvLimits,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BttvLimits {
    pub shared_emotes: usize,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BttvEmote {
    pub id: String,
    pub code: String,
}

impl PartialEq for BttvEmote {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id || self.code == other.code
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BttvUserInfo {
    pub id: String,
    pub channel_emotes: Vec<BttvEmote>,
    pub shared_emotes: Vec<BttvEmote>,
}

pub async fn get_dashboards() -> AnyResult<Vec<BttvEditor>> {
    bttv_get("https://api.betterttv.net/3/account/dashboards").await
}

pub async fn get_user(bttv_id: &str) -> AnyResult<BttvUserInfo> {
    bttv_get(format!(
        "https://api.betterttv.net/3/users/{}?limited=false&personal=false",
        bttv_id
    ))
    .await
}

pub async fn get_emote(emote_id: &str) -> AnyResult<BttvEmote> {
    bttv_get(format!("https://api.betterttv.net/3/emotes/{}", emote_id)).await
}

pub async fn add_shared_emote(emote_id: &str, user_id: &str) -> AnyResult<String> {
    bttv_put(format!(
        "https://api.betterttv.net/3/emotes/{}/shared/{}",
        emote_id, user_id
    ))
    .await
}

pub async fn delete_shared_emote(emote_id: &str, user_id: &str) -> AnyResult<String> {
    bttv_delete(format!(
        "https://api.betterttv.net/3/emotes/{}/shared/{}",
        emote_id, user_id
    ))
    .await
}

pub async fn get_user_by_twitch_id(id: &str) -> AnyResult<BttvUserInfo> {
    bttv_get(format!(
        "https://api.betterttv.net/3/cached/users/twitch/{}",
        id
    ))
    .await
}

async fn bttv_get<T, U>(url: U) -> AnyResult<T>
where
    T: DeserializeOwned,
    U: IntoUrl,
{
    Ok(reqwest::Client::new()
        .get(url)
        .header("Authorization", format!("Bearer {}", BTTV_JWT))
        .send()
        .await?
        .json()
        .await?)
}

async fn bttv_delete<U>(url: U) -> AnyResult<String>
where
    U: IntoUrl,
{
    Ok(reqwest::Client::new()
        .delete(url)
        .header("Authorization", format!("Bearer {}", BTTV_JWT))
        .send()
        .await?
        .text()
        .await?)
}

async fn bttv_put<U>(url: U) -> AnyResult<String>
where
    U: IntoUrl,
{
    Ok(reqwest::Client::new()
        .put(url)
        .header("Authorization", format!("Bearer {}", BTTV_JWT))
        .send()
        .await?
        .text()
        .await?)
}
