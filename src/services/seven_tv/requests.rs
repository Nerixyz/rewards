use crate::constants::SEVEN_TV_JWT;
use anyhow::{Error as AnyError, Result as AnyResult};
use reqwest::IntoUrl;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::array::IntoIter;
use std::collections::HashMap;

#[derive(Serialize)]
struct GqlRequest<'a> {
    query: &'a str,
    variables: HashMap<&'a str, &'a str>,
}

#[derive(Deserialize, Debug)]
#[serde(bound = "T: DeserializeOwned")]
#[non_exhaustive]
struct GqlResponse<T>
where
    T: DeserializeOwned,
{
    data: T,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
struct SevenEmoteResponse {
    emote: SevenEmote,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct SevenEmote {
    pub name: String,
    pub id: String,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
struct SevenUserResponse {
    user: SevenUser,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
struct SevenUserEditorsResponse {
    user: SevenUserOnlyEditors,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct SevenUserOnlyEditors {
    pub editors: Vec<OnlyName>,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct OnlyName {
    pub login: String,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct SevenUser {
    pub id: String,
    pub login: String,
    pub emotes: Vec<SevenEmote>,
    pub twitch_id: String,
    pub emote_slots: usize,
}

pub async fn get_user(user_id_or_login: &str) -> AnyResult<SevenUser> {
    let user = seven_tv_post::<GqlResponse<SevenUserResponse>, _>("https://api.7tv.app/v2/gql", &GqlRequest {
        query: "query getUser($id: String!) { user(id: $id) { id, login, emotes { id, name }, twitch_id, emote_slots } }",
        variables: IntoIter::new([("id", user_id_or_login)]).collect::<HashMap<_, _>>()
    }).await?;

    Ok(user.data.user)
}

pub async fn get_user_editors(user_id_or_login: &str) -> AnyResult<Vec<OnlyName>> {
    let user = seven_tv_post::<GqlResponse<SevenUserEditorsResponse>, _>(
        "https://api.7tv.app/v2/gql",
        &GqlRequest {
            query: "query getUser($id: String!) { user(id: $id) { editors { login } } }",
            variables: IntoIter::new([("id", user_id_or_login)]).collect::<HashMap<_, _>>(),
        },
    )
    .await?;

    Ok(user.data.user.editors)
}

pub async fn get_emote(emote_id: &str) -> AnyResult<SevenEmote> {
    let emote = seven_tv_post::<GqlResponse<SevenEmoteResponse>, _>(
        "https://api.7tv.app/v2/gql",
        &GqlRequest {
            query: "query emoteQuery($id: String!){emote(id: $id){id,name, tags}}",
            variables: IntoIter::new([("id", emote_id)]).collect::<HashMap<_, _>>(),
        },
    )
    .await?;

    Ok(emote.data.emote)
}

pub async fn add_emote(channel_id: &str, emote_id: &str) -> AnyResult<()> {
    seven_tv_post::<GqlResponse<serde_json::Value>, _>("https://api.7tv.app/v2/gql", &GqlRequest {
        query: "mutation AddChannelEmote($ch: String!, $em: String!, $re: String!) {addChannelEmote(channel_id: $ch, emote_id: $em, reason: $re) {emote_ids}}",
        variables: IntoIter::new([("ch", channel_id), ("em", emote_id), ("re", "")]).collect::<HashMap<_,_>>()
    }).await?;

    Ok(())
}

pub async fn remove_emote(channel_id: &str, emote_id: &str) -> AnyResult<()> {
    seven_tv_post::<GqlResponse<serde_json::Value>, _>("https://api.7tv.app/v2/gql", &GqlRequest {
        query: "mutation RemoveChannelEmote($ch: String!, $em: String!, $re: String!) {removeChannelEmote(channel_id: $ch, emote_id: $em, reason: $re) {emote_ids}}",
        variables: IntoIter::new([("ch", channel_id), ("em", emote_id), ("re", "")]).collect::<HashMap<_,_>>()
    }).await?;

    Ok(())
}

async fn seven_tv_post<J, U>(url: U, request: &GqlRequest<'_>) -> AnyResult<J>
where
    J: DeserializeOwned,
    U: IntoUrl,
{
    let response = reqwest::Client::default()
        .post(url)
        .header("Authorization", format!("Bearer {}", SEVEN_TV_JWT))
        .json(request)
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(AnyError::msg(format!(
            "Non OK status: {}",
            response.status()
        )));
    }

    Ok(response.json().await?)
}
