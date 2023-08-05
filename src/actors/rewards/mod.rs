mod messages;
use crate::{
    actors::{discord::DiscordActor, irc::IrcActor, timeout::TimeoutActor},
    services::{
        emotes::{bttv::BttvEmotes, ffz::FfzEmotes, seven_tv::SevenTvEmotes},
        rewards::{execute, extract},
    },
    RedisPool,
};
use actix::{Actor, Addr, Context, Handler, ResponseFuture};
use anyhow::Result as AnyResult;
use futures::FutureExt;
pub use messages::*;
use models::{reward::RewardData, timed_mode};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use twitch_api2::twitch_oauth2::AppAccessToken;

pub struct RewardsActor {
    pub irc: Addr<IrcActor>,
    pub db: PgPool,
    pub redis: RedisPool,
    pub app_access_token: Arc<RwLock<AppAccessToken>>,
    pub timeout: Addr<TimeoutActor>,
    pub discord: Addr<DiscordActor>,
}

impl RewardsActor {}

impl Actor for RewardsActor {
    type Context = Context<Self>;
}

impl Handler<ExecuteRewardMessage> for RewardsActor {
    type Result = ResponseFuture<AnyResult<()>>;

    fn handle(
        &mut self,
        msg: ExecuteRewardMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        match msg.reward.data.0 {
            RewardData::Timeout(timeout) => execute::timeout(
                timeout,
                msg.redemption,
                msg.broadcaster,
                (
                    self.irc.clone(),
                    self.app_access_token.clone(),
                    self.timeout.clone(),
                ),
            )
            .boxed(),
            RewardData::SubOnly(duration) => execute::timed_mode(
                timed_mode::Mode::Subonly,
                duration,
                msg.broadcaster,
                msg.redemption,
                self.irc.clone(),
            )
            .boxed(),
            RewardData::EmoteOnly(duration) => execute::timed_mode(
                timed_mode::Mode::Emoteonly,
                duration,
                msg.broadcaster,
                msg.redemption,
                self.irc.clone(),
            )
            .boxed(),
            RewardData::BttvSwap(data) => execute::swap::<BttvEmotes>(
                extract::bttv,
                msg.redemption,
                data,
                (self.db.clone(), self.irc.clone(), self.discord.clone()),
            )
            .boxed(),
            RewardData::FfzSwap(data) => execute::swap::<FfzEmotes>(
                extract::ffz,
                msg.redemption,
                data,
                (self.db.clone(), self.irc.clone(), self.discord.clone()),
            )
            .boxed(),
            RewardData::SevenTvSwap(data) => execute::swap::<SevenTvEmotes>(
                extract::seventv,
                msg.redemption,
                data,
                (self.db.clone(), self.irc.clone(), self.discord.clone()),
            )
            .boxed(),
            RewardData::BttvSlot(slot) => execute::slot::<BttvEmotes>(
                extract::bttv,
                msg.redemption,
                slot,
                (self.db.clone(), self.irc.clone(), self.discord.clone()),
            )
            .boxed(),
            RewardData::FfzSlot(slot) => execute::slot::<FfzEmotes>(
                extract::ffz,
                msg.redemption,
                slot,
                (self.db.clone(), self.irc.clone(), self.discord.clone()),
            )
            .boxed(),
            RewardData::SevenTvSlot(slot) => execute::slot::<SevenTvEmotes>(
                extract::seventv,
                msg.redemption,
                slot,
                (self.db.clone(), self.irc.clone(), self.discord.clone()),
            )
            .boxed(),
            RewardData::SpotifySkip(_) => execute::spotify_skip(
                msg.redemption,
                (self.db.clone(), self.irc.clone()),
            )
            .boxed(),
            RewardData::SpotifyQueue(opts) => execute::spotify_queue(
                opts,
                msg.redemption,
                (self.db.clone(), self.irc.clone()),
            )
            .boxed(),
            RewardData::SpotifyPlay(opts) => execute::spotify_play(
                opts,
                msg.redemption,
                (self.db.clone(), self.irc.clone()),
            )
            .boxed(),
        }
    }
}
