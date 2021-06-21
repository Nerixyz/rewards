use reqwest::IntoUrl;
use crate::constants::SEVEN_TV_JWT;
use std::collections::HashMap;
use anyhow::{Result as AnyResult, Error as AnyError};
use serde::de::DeserializeOwned;
use serde::{Serialize, Deserialize};
use std::iter::FromIterator;
use std::array::IntoIter;

#[derive(Serialize)]
struct GqlRequest<'a> {
    query: &'a str,
    variables: HashMap<&'a str, &'a str>,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
struct GqlResponse<T>
where T: DeserializeOwned {
    data: T,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
struct SevenEmoteResponse {
    emote: SevenEmote
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct SevenEmote {
    name: String,
    id: String
}


#[derive(Deserialize, Debug)]
#[non_exhaustive]
struct SevenUserResponse {
    emote: SevenUser
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct SevenUser {
    id: String,
    login: String,
    emote_ids: Vec<String>,
    twitch_id: String,
    emote_slots: usize,
}

pub async fn get_user(user_id: &str) -> AnyResult<SevenUser> {
    let user = seven_tv_post::<GqlResponse<SevenUserResponse>, _>("https://api.7tv.app/v2/gql", &GqlRequest {
        query: "query getUser($id: String!) { user(id: $id) { id, login, emote_ids, twitch_id, emote_slots } }",
        variables: HashMap::<_,_>::from_iter(IntoIter::new([("id", user_id)]))
    }).await?;

    Ok(user.data.user)
}

pub async fn get_emote(emote_id: &str) -> AnyResult<SevenEmote> {
    let emote = seven_tv_post::<GqlResponse<SevenEmoteResponse>, _>("https://api.7tv.app/v2/gql", &GqlRequest {
        query: "query emoteQuery($id: String!){emote(id: $id){id,name, tags}}",
        variables: HashMap::<_,_>::from_iter(IntoIter::new([("id", emote_id)]))
    }).await?;

    Ok(emote.data.emote)
}

pub async fn add_emote(channel_id: &str, emote_id: &str) -> AnyResult<()> {
    seven_tv_post::<GqlResponse<serde_json::Value>, _>("https://api.7tv.app/v2/gql", &GqlRequest {
        query: "mutation AddChannelEmote($ch: String!, $em: String!, $re: String!) {addChannelEmote(channel_id: $ch, emote_id: $em, reason: $re) {emote_ids}}",
        variables: HashMap::<_,_>::from_iter(IntoIter::new([("ch", channel_id), ("em", emote_id), ("re", "")]))
    }).await?;

    Ok(())
}

pub async fn remove_emote(channel_id: &str, emote_id: &str) -> AnyResult<()> {
    seven_tv_post::<GqlResponse<serde_json::Value>, _>("https://api.7tv.app/v2/gql", &GqlRequest {
        query: "mutation RemoveChannelEmote($ch: String!, $em: String!, $re: String!) {removeChannelEmote(channel_id: $ch, emote_id: $em, reason: $re) {emote_ids}}",
        variables: HashMap::<_,_>::from_iter(IntoIter::new([("ch", channel_id), ("em", emote_id), ("re", "")]))
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
        return Err(AnyError::msg(format!("Non OK status: {}", response.status())))
    }

    Ok(response.json().await?)
}