use anyhow::{Error as AnyError, Result as AnyResult};
use config::CONFIG;
use lazy_static::lazy_static;
use reqwest::{
    header::{self, HeaderMap},
    Client, IntoUrl,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

lazy_static! {
    static ref SEVENTV_CLIENT: Client = Client::builder()
        .user_agent(format!(
            "RewardMore/{} github.com/Nerixyz/rewards",
            env!("CARGO_PKG_VERSION")
        ))
        .default_headers({
            let mut map = HeaderMap::with_capacity(1);
            map.insert(
                header::AUTHORIZATION,
                format!("Bearer {}", CONFIG.emotes.seven_tv.jwt)
                    .parse()
                    .unwrap(),
            );
            map
        })
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap();
}

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
struct GqlErrors {
    errors: Vec<GqlError>,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
struct GqlError {
    message: String,
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
        variables: [("id", user_id_or_login)].into_iter().collect::<HashMap<_, _>>()
    }).await?;

    Ok(user.data.user)
}

pub async fn get_user_editors(user_id_or_login: &str) -> AnyResult<Vec<OnlyName>> {
    let user = seven_tv_post::<GqlResponse<SevenUserEditorsResponse>, _>(
        "https://api.7tv.app/v2/gql",
        &GqlRequest {
            query: "query getUser($id: String!) { user(id: $id) { editors { login } } }",
            variables: [("id", user_id_or_login)]
                .into_iter()
                .collect::<HashMap<_, _>>(),
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
            variables: [("id", emote_id)].into_iter().collect::<HashMap<_, _>>(),
        },
    )
    .await?;

    Ok(emote.data.emote)
}

pub async fn add_emote(channel_id: &str, emote_id: &str) -> AnyResult<()> {
    seven_tv_post::<GqlResponse<serde_json::Value>, _>("https://api.7tv.app/v2/gql", &GqlRequest {
        query: "mutation AddChannelEmote($ch: String!, $em: String!, $re: String!) {addChannelEmote(channel_id: $ch, emote_id: $em, reason: $re) {emote_ids}}",
        variables: [("ch", channel_id), ("em", emote_id), ("re", "")].into_iter().collect::<HashMap<_,_>>()
    }).await?;

    Ok(())
}

pub async fn remove_emote(channel_id: &str, emote_id: &str) -> AnyResult<()> {
    seven_tv_post::<GqlResponse<serde_json::Value>, _>("https://api.7tv.app/v2/gql", &GqlRequest {
        query: "mutation RemoveChannelEmote($ch: String!, $em: String!, $re: String!) {removeChannelEmote(channel_id: $ch, emote_id: $em, reason: $re) {emote_ids}}",
        variables: [("ch", channel_id), ("em", emote_id), ("re", "")].into_iter().collect::<HashMap<_,_>>()
    }).await?;

    Ok(())
}

pub async fn logged_in() -> bool {
    seven_tv_post::<GqlResponse<serde_json::Value>, _>(
        "https://api.7tv.app/v2/gql",
        &GqlRequest {
            query: "query GetUser($id: String!) {user(id: $id) {id}}",
            variables: [("id", "@me")].into_iter().collect::<HashMap<_, _>>(),
        },
    )
    .await
    .is_ok()
}

async fn seven_tv_post<J, U>(url: U, request: &GqlRequest<'_>) -> AnyResult<J>
where
    J: DeserializeOwned,
    U: IntoUrl,
{
    let response = SEVENTV_CLIENT.post(url).json(request).send().await?;
    if !response.status().is_success() {
        return Err(AnyError::msg(format!(
            "Non OK status: {} - Error: {}",
            response.status(),
            response
                .json::<GqlErrors>()
                .await
                .map(|e| e
                    .errors
                    .into_iter()
                    .next()
                    .map(|e| e.message)
                    .unwrap_or_else(|| "<no error>?".to_string()))
                .unwrap_or_else(|e| e.to_string())
        )));
    }

    Ok(response.json().await?)
}
