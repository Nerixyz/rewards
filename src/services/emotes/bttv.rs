use crate::{
    models::{
        slot::SlotPlatform,
        user::{User, UserBttvData},
    },
    services::{
        bttv::{get_or_fetch_id, requests as bttv},
        emotes::{Emote, EmoteEnvData, EmoteId, EmoteInitialData, EmoteRW},
    },
};
use anyhow::{Error as AnyError, Result as AnyResult};
use async_trait::async_trait;
use futures::TryFutureExt;
use sqlx::PgPool;

pub struct BttvEmotes {
    _private: usize,
}

impl BttvEmotes {
    async fn get_data_and_id(
        broadcaster_id: &str,
        pool: &PgPool,
    ) -> AnyResult<(UserBttvData, String)> {
        let this_user = User::get_bttv_data(broadcaster_id, pool)
            .await
            .map_err(|_| AnyError::msg("No internal user"))?;
        let bttv_id = if let Some(id) = &this_user.bttv_id {
            id.to_string()
        } else {
            fetch_save_bttv_id(broadcaster_id, pool)
                .await
                .map_err(|_| AnyError::msg("No such user"))?
        };

        Ok((this_user, bttv_id))
    }
}

impl Emote<String> for bttv::BttvEmote {
    fn id(&self) -> &String {
        &self.id
    }

    fn name(self) -> String {
        self.code
    }
}

impl EmoteId for String {
    fn from_db(db: &str) -> AnyResult<Self> {
        Ok(db.to_owned())
    }
}

#[async_trait]
impl EmoteRW for BttvEmotes {
    type PlatformId = String;
    type Emote = bttv::BttvEmote;
    type EmoteId = String;

    fn platform() -> SlotPlatform {
        SlotPlatform::Bttv
    }

    async fn get_check_initial_data(
        broadcaster_id: &str,
        emote_id: &str,
        pool: &PgPool,
    ) -> AnyResult<EmoteInitialData<String, bttv::BttvEmote, String>> {
        let (this_user, bttv_id) = Self::get_data_and_id(broadcaster_id, pool).await?;

        // get the data in parallel
        let (bttv_user, user_limits, emote_data) = futures::future::try_join3(
            bttv::get_user(&bttv_id).map_err(|_| AnyError::msg("No such user.")),
            get_user_limits(&bttv_id).map_err(|_| AnyError::msg("I'm not added as an editor.")),
            bttv::get_emote(emote_id).map_err(|_| AnyError::msg("This emote doesn't exist.")),
        )
        .await?;

        // check if there's already an emote with the same name or id
        // If the added emote will replace an emote with the same name it will never work!
        if bttv_user
            .shared_emotes
            .iter()
            .any(|e| e.id == emote_id || e.code == emote_data.code)
        {
            return Err(AnyError::msg("The emote already exists as a shared emote"));
        }
        if bttv_user
            .channel_emotes
            .iter()
            .any(|e| e.id == emote_id || e.code == emote_data.code)
        {
            return Err(AnyError::msg("The emote already exists as a channel emote"));
        }
        Ok(EmoteInitialData {
            max_emotes: user_limits.shared_emotes,
            current_emotes: bttv_user.shared_emotes.len(),
            history: this_user.bttv_history.0,
            platform_id: bttv_id,
            emote: emote_data,
            emotes: bttv_user.shared_emotes,
        })
    }

    async fn get_emote_env_data(
        _broadcaster_id: &str,
        platform_id: &String,
    ) -> AnyResult<EmoteEnvData> {
        let (bttv_limits, shared_emotes) =
            futures::future::try_join(get_user_limits(platform_id), bttv::get_user(platform_id))
                .await?;

        Ok(EmoteEnvData {
            max_emotes: bttv_limits.shared_emotes,
            current_emotes: shared_emotes.shared_emotes.len(),
        })
    }

    async fn get_history_and_platform_id(
        broadcaster_id: &str,
        pool: &PgPool,
    ) -> AnyResult<(Vec<Self::EmoteId>, Self::PlatformId)> {
        let (this_user, id) = Self::get_data_and_id(broadcaster_id, pool).await?;
        Ok((this_user.bttv_history.0, id))
    }

    async fn get_emote_by_id(emote_id: &String) -> AnyResult<bttv::BttvEmote> {
        bttv::get_emote(emote_id).await
    }

    async fn remove_emote(platform_id: &String, emote_id: &String) -> AnyResult<()> {
        bttv::delete_shared_emote(emote_id, platform_id).await?;
        Ok(())
    }

    async fn add_emote(platform_id: &String, emote_id: &String) -> AnyResult<()> {
        bttv::add_shared_emote(emote_id, platform_id).await?;
        Ok(())
    }

    async fn save_history(
        broadcaster_id: &str,
        history: Vec<String>,
        pool: &PgPool,
    ) -> AnyResult<()> {
        User::set_bttv_history(broadcaster_id, history, pool).await?;
        Ok(())
    }

    async fn remove_emote_from_broadcaster(
        broadcaster_id: &str,
        emote_id: &str,
        pool: &PgPool,
    ) -> AnyResult<String> {
        let bttv_id = match get_or_fetch_id(broadcaster_id, pool).await {
            Ok(id) => id,
            Err(e) => {
                log::warn!("No user-id? broadcaster={} error={}", broadcaster_id, e);
                return Err(e);
            }
        };

        let (_, emote) = futures::future::try_join(
            bttv::delete_shared_emote(emote_id, &bttv_id),
            bttv::get_emote(emote_id),
        )
        .await?;

        Ok(emote.code)
    }
}

async fn get_user_limits(bttv_id: &str) -> AnyResult<bttv::BttvLimits> {
    bttv::get_dashboards()
        .await?
        .into_iter()
        .find(|d| d.id == bttv_id)
        .map(|u| u.limits)
        .ok_or_else(|| AnyError::msg("User isn't an editor"))
}

async fn fetch_save_bttv_id(user_id: &str, pool: &PgPool) -> AnyResult<String> {
    let user = bttv::get_user_by_twitch_id(user_id).await?;
    User::set_bttv_id(user_id, &user.id, pool).await?;

    Ok(user.id)
}
