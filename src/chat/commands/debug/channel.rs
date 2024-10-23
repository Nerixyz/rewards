use super::emotes::EmoteData;
use crate::{services::twitch, PgPool};
use anyhow::Result as AnyResult;
use futures_util::future;
use models::user;
use std::fmt::{Display, Formatter};

pub struct ChannelData {
    pub emote_data: EmoteData,
    pub twitch_auth: bool,
}

impl ChannelData {
    pub async fn get(
        channel_id: &str,
        channel_login: &str,
        pool: &PgPool,
    ) -> AnyResult<Self> {
        let (emote_data, twitch_auth) = future::join(
            EmoteData::get(channel_id, channel_login, pool),
            Self::get_twitch_auth(channel_id, pool),
        )
        .await;
        Ok(Self {
            emote_data,
            twitch_auth: twitch_auth?,
        })
    }

    async fn get_twitch_auth(
        channel_id: &str,
        pool: &PgPool,
    ) -> AnyResult<bool> {
        let user = user::User::get_by_id(channel_id, pool).await?;
        let token = user.into();

        twitch::requests::validate_token(&token).await
    }
}

impl Display for ChannelData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{twitch}7TV({stv}), BTTV({bttv}), FFZ({ffz})",
            twitch = if self.twitch_auth {
                ""
            } else {
                "Twitch Auth: ❌, "
            },
            stv = self.emote_data.seventv,
            bttv = self.emote_data.bttv,
            ffz = self.emote_data.ffz
        )
    }
}
