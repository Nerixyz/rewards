use anyhow::{anyhow, Result as AnyResult};
use config::CONFIG;
use lazy_static::lazy_static;
use reqwest::{
    header::{self, HeaderMap},
    Client, IntoUrl, StatusCode,
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
#[serde(bound = "V: Serialize")]
struct GqlRequest<'a, V = HashMap<&'a str, &'a str>> {
    query: &'a str,
    variables: V,
}

#[derive(Deserialize, Debug)]
#[serde(bound = "T: DeserializeOwned")]
#[non_exhaustive]
struct GqlResponse<T>
where
    T: DeserializeOwned,
{
    data: T,
    errors: Option<Vec<GqlError>>,
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

#[derive(Serialize, Deserialize, Debug)]
#[non_exhaustive]
pub struct SevenEmote {
    pub name: String,
    pub id: String,
    #[serde(default)]
    pub listed: bool,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct SevenUserResponse {
    pub emote_set: Option<SevenEmoteSet>,
    pub user: SevenUser,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct SevenUser {
    #[serde(default)]
    pub editors: Vec<SevenEditor>,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct SevenEmoteSet {
    pub id: String,
    pub capacity: usize,
    #[serde(default)]
    pub emotes: Vec<SevenEmote>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SevenEditorRelation {
    pub user_id: String,
    pub editor_id: String,
    pub status: SevenEditorStatus,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum SevenEditorStatus {
    Accepted,
    Rejected,
    Pending,
    #[serde(other)]
    Other,
}

#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum SevenEditorUpdateStatus {
    Accept,
    Reject,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GqlEditorUsersWrap {
    pub users: GqlEditorUserWrap,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GqlEditorUserWrap {
    pub editor_for: Vec<SevenEditorRelation>,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct SevenEditor {
    pub id: String,
}

#[derive(Serialize)]
struct GqlIdVars<'a> {
    id: &'a str,
}

#[derive(Serialize)]
struct GqlEmoteInSetVars<'a> {
    set_id: &'a str,
    emote_id: &'a str,
    name: Option<&'a str>,
}

#[derive(Serialize)]
struct GqlEditorUpdateVars<'a> {
    editor_id: &'a str,
    user_id: &'a str,
}

#[derive(Deserialize, Serialize)]
#[non_exhaustive]
struct VoidObject {}

pub async fn get_user(user_id: &str) -> AnyResult<SevenUserResponse> {
    seven_tv_get::<SevenUserResponse>(format!(
        "https://7tv.io/v3/users/twitch/{}",
        CONFIG.debug_overrides.seventv(user_id)
    ))
    .await
}

pub async fn get_emote_set(id: &str) -> AnyResult<SevenEmoteSet> {
    seven_tv_get::<SevenEmoteSet>(format!(
        "https://7tv.io/v3/emote-sets/{}",
        id
    ))
    .await
}

pub async fn get_emote(emote_id: &str) -> AnyResult<SevenEmote> {
    let emote = seven_tv_post::<SevenEmoteResponse>(
        "https://7tv.io/v3/gql",
        &GqlRequest {
            query:
                "query($id: ObjectID!) { emote(id: $id) { id, name, listed } }",
            variables: GqlIdVars { id: emote_id },
        },
    )
    .await?;

    Ok(emote.data.emote)
}

pub async fn add_emote(
    emote_set_id: &str,
    emote_id: &str,
    overwritten_name: Option<&str>,
) -> AnyResult<()> {
    seven_tv_post::<Option<VoidObject>>("https://7tv.io/v3/gql", &GqlRequest {
        query: "mutation($set_id: ObjectID!, $emote_id: ObjectID!, $name: String) { emoteSet(id: $set_id) { emotes(id: $emote_id, action: ADD, name: $name) { id } } }",
        variables: GqlEmoteInSetVars { set_id: emote_set_id, emote_id, name: overwritten_name }
    }).await?;

    Ok(())
}

pub async fn remove_emote(emote_set_id: &str, emote_id: &str) -> AnyResult<()> {
    seven_tv_post::<Option<VoidObject>>("https://7tv.io/v3/gql", &GqlRequest {
        query: "mutation($set_id: ObjectID!, $emote_id: ObjectID!, $name: String) { emoteSet(id: $set_id) { emotes(id: $emote_id, action: REMOVE, name: $name) { id } } }",
        variables: GqlEmoteInSetVars { set_id: emote_set_id, emote_id, name: None }
    }).await?;

    Ok(())
}

pub async fn get_editor_relations() -> AnyResult<Vec<SevenEditorRelation>> {
    let res = seven_tv_post::<GqlEditorUsersWrap>(
        "https://7tv.io/v4/gql",
        &GqlRequest {
            query: r#"
                query ($id: Id!) {
                    users {
                        user(id: $id) {
                            editorFor {
                                userId
                                editorId
                                state
                            }
                        }
                    }
                }
            "#,
            variables: GqlIdVars {
                id: &CONFIG.emotes.seven_tv.user_id,
            },
        },
    )
    .await?;

    Ok(res.data.users.editor_for)
}

pub async fn approve_editor(user_id: &str, editor_id: &str) -> AnyResult<()> {
    seven_tv_post::<VoidObject>(
        "https://7tv.io/v4/gql",
        &GqlRequest {
            query: r#"
                mutation ($user_id: Id!, $editor_id: Id!) {
                    userEditors {
                        editor(userId: $user_id, editorId: $editor_id) {
                            updateState(state: ACCEPT) {
                                state
                            }
                        }
                    }
                }
            "#,
            variables: GqlEditorUpdateVars { editor_id, user_id },
        },
    )
    .await?;

    Ok(())
}

pub async fn logged_in() -> bool {
    #[derive(Deserialize)]
    #[non_exhaustive]
    struct Data {
        actor: Option<VoidObject>,
    }
    seven_tv_post::<Data>(
        "https://7tv.io/v3/gql",
        &GqlRequest {
            query: "query {actor { id } }",
            variables: (),
        },
    )
    .await
    .map(|d| d.data.actor.is_some())
    .unwrap_or(false)
}

async fn seven_tv_post<J>(
    url: impl IntoUrl,
    request: &GqlRequest<'_, impl Serialize>,
) -> AnyResult<GqlResponse<J>>
where
    J: DeserializeOwned,
{
    let response = SEVENTV_CLIENT.post(url).json(request).send().await?;
    let status = response.status();
    let response = response.json().await?;
    match response {
        GqlResponse {
            errors: Some(errors),
            ..
        } if !errors.is_empty() => Err(anyhow!(
            "7TV Error: {} (http-status={})",
            errors[0].message,
            status.as_str()
        )),
        res => Ok(res),
    }
}

async fn seven_tv_get<R>(url: impl IntoUrl) -> AnyResult<R>
where
    R: DeserializeOwned,
{
    let response = SEVENTV_CLIENT.get(url).send().await?;
    let status = response.status();
    match status {
        s if s.is_success() => Ok(response.json().await?),
        StatusCode::NOT_FOUND => Err(anyhow!("7TV error: Not found")),
        _ => Err(anyhow!("7TV error: {}", response.text().await?)),
    }
}
