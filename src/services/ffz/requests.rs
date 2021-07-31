use crate::constants::{FFZ_REMEMBER, FFZ_SESSION};
use anyhow::{Error as AnyError, Result as AnyResult};
use futures::TryFutureExt;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use reqwest::{cookie::Jar, Client, IntoUrl, Response, Url};
use serde::{de::DeserializeOwned, Deserialize};
use std::{collections::HashMap, fmt::Display, sync::Arc};

lazy_static! {
    static ref FFZ_CLIENT: Client = Client::builder()
        .user_agent(format!(
            "RewardMore/{} github.com/Nerixyz/rewards",
            env!("CARGO_PKG_VERSION")
        ))
        .cookie_store(true)
        .cookie_provider(Arc::new({
            let jar = Jar::default();

            let url = "https://frankerfacez.com".parse::<Url>().unwrap();
            jar.add_cookie_str(
                &format!("session={}; Domain=frankerfacez.com", FFZ_SESSION),
                &url,
            );
            jar.add_cookie_str(
                &format!(
                    "remember_token={}; Domain=www.frankerfacez.com",
                    FFZ_REMEMBER
                ),
                &url,
            );

            jar
        }))
        .build()
        .unwrap();
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct FfzEmote {
    pub id: usize,
    pub name: String,
}

impl PartialEq for FfzEmote {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id || self.name == other.name
    }
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
struct FfzEmoteReply {
    emote: FfzEmote,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct FfzEmoteSet {
    pub id: usize,
    pub emoticons: Vec<FfzEmote>,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct FfzRoomData {
    pub sets: HashMap<String, FfzEmoteSet>,
    pub room: FfzRoom,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct FfzRoom {
    pub _id: usize,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct FfzUser {
    pub max_emoticons: usize,
    pub id: usize,
    pub name: String,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
struct FfzUserReply {
    user: FfzUser,
}

pub async fn get_emote<T: Display>(id: T) -> AnyResult<FfzEmote> {
    ffz_get_json::<FfzEmoteReply, _>(format!("https://api.frankerfacez.com/v1/emote/{}", id))
        .await
        .map(|e| e.emote)
}

pub async fn get_room(id: &str) -> AnyResult<FfzRoomData> {
    ffz_get_json(format!("https://api.frankerfacez.com/v1/room/id/{}", id)).await
}

pub async fn get_user(id: &str) -> AnyResult<FfzUser> {
    ffz_get_json::<FfzUserReply, _>(format!("https://api.frankerfacez.com/v1/user/id/{}", id))
        .await
        .map(|u| u.user)
}

pub async fn get_channels() -> AnyResult<Vec<String>> {
    lazy_static! {
        static ref CHANNEL_REGEX: Regex =
            Regex::new("<li><a href=\"/channel/([\\w_]+)\">[^My ]{3}").expect("must compile");
    }

    let text = ffz_get_text("https://www.frankerfacez.com/").await?;
    Ok(CHANNEL_REGEX
        .captures_iter(&text)
        .map(|c: Captures| c.iter().nth(1).flatten().map(|m| m.as_str().to_string()))
        .flatten()
        .collect())
}

pub async fn add_emote(channel_id: usize, emote_id: usize) -> AnyResult<()> {
    lazy_static! {
        static ref SUCCESS_REGEX: Regex =
            Regex::new("Added the emote [^ ]+ to the channel").expect("must compile");
        static ref REASON_REGEX: Regex =
            Regex::new("&times;</span></button>\\n([^<][^\\n]+)\\n</div>").expect("must compile");
    }

    let text = ffz_get_text_auth(
        format!(
            "https://www.frankerfacez.com/emoticons/channel/True?channels={}&ids={}&unlock_code=",
            channel_id, emote_id
        ),
        &emote_id.to_string(),
    )
    .await?;
    check_for_success(&text, &SUCCESS_REGEX, &REASON_REGEX)
}

pub async fn delete_emote(channel_id: usize, emote_id: usize) -> AnyResult<()> {
    lazy_static! {
        static ref SUCCESS_REGEX: Regex =
            Regex::new("Removed the emote [^ ]+ from the channel").expect("must compile");
        static ref REASON_REGEX: Regex =
            Regex::new("&times;</span></button>\\n([^<][^\\n]+)\\n</div>").expect("must compile");
    }

    let text = ffz_get_text_auth(
        format!(
            "https://www.frankerfacez.com/emoticons/channel/False?channels={}&ids={}&unlock_code=",
            channel_id, emote_id
        ),
        &emote_id.to_string(),
    )
    .await?;
    check_for_success(&text, &SUCCESS_REGEX, &REASON_REGEX)
}

fn check_for_success(text: &str, success: &Regex, reason: &Regex) -> AnyResult<()> {
    if success.is_match(text) {
        Ok(())
    } else {
        let reason = reason
            .captures(&text)
            .map(|c| c.iter().nth(1).flatten().map(|m| m.as_str().to_string()))
            .flatten()
            .unwrap_or_else(|| "No reason found".to_string());

        Err(AnyError::msg(reason))
    }
}

async fn ffz_get_json<T, U>(url: U) -> AnyResult<T>
where
    T: DeserializeOwned,
    U: IntoUrl,
{
    Ok(FFZ_CLIENT.get(url).send().and_then(Response::json).await?)
}

async fn ffz_get_text<U>(url: U) -> AnyResult<String>
where
    U: IntoUrl,
{
    Ok(FFZ_CLIENT.get(url).send().and_then(Response::text).await?)
}

async fn ffz_get_text_auth<U>(url: U, referer: &str) -> AnyResult<String>
where
    U: IntoUrl,
{
    Ok(FFZ_CLIENT
        .get(url)
        .header(
            "Referer",
            format!("https://www.frankerfacez.com/emoticon/{}", referer),
        )
        .send()
        .and_then(Response::text)
        .await?)
}
