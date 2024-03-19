use std::time::Duration;

use actix::{
    Actor, ActorFutureExt, Addr, AsyncContext, Context, ContextFutureSpawner,
    Handler, StreamHandler, WrapFuture,
};
use sqlx::PgPool;
use token_provider::PubsubTokenProvider;
use tokio_stream::wrappers::UnboundedReceiverStream;
use twitch_pubsub::{
    moderation::{
        ChatModeratorActions, ChatModeratorActionsReply, ModerationAction,
        ModerationActionCommand,
    },
    video_playback::{VideoPlaybackById, VideoPlaybackReply},
    PubSubEvent, Topic, TopicData, TopicDef,
};
use url::Url;

use crate::{
    actors::{
        live::{LiveActor, LiveMessage, OfflineMessage},
        timeout::{RemoveTimeoutMessage, TimeoutActor},
    },
    log_discord, log_err,
};
use config::CONFIG;

mod messages;
mod token_provider;
pub use messages::*;

pub struct PubSubActor {
    live_addr: Addr<LiveActor>,
    timeout_handler: Addr<TimeoutActor>,

    client: twitch_pubsub::Sender,
}

impl PubSubActor {
    pub fn run(
        pool: PgPool,
        live_addr: Addr<LiveActor>,
        timeout_handler: Addr<TimeoutActor>,
    ) -> Addr<Self> {
        let (client, incoming) = twitch_pubsub::create_manager(
            PubsubTokenProvider(pool),
            Url::parse("wss://pubsub-edge.twitch.tv").unwrap(),
        );

        Self::create(|ctx| {
            ctx.add_stream(UnboundedReceiverStream::new(incoming));

            Self {
                live_addr,
                timeout_handler,
                client,
            }
        })
    }

    fn make_topics(channel_id: u32, this_user_id: u32) -> Vec<Topic> {
        vec![
            VideoPlaybackById { channel_id }.into_topic(),
            ChatModeratorActions {
                channel_id,
                user_id: this_user_id,
            }
            .into_topic(),
        ]
    }
}

impl Actor for PubSubActor {
    type Context = Context<Self>;
}

impl StreamHandler<PubSubEvent<PubsubTokenProvider>> for PubSubActor {
    fn handle(
        &mut self,
        item: PubSubEvent<PubsubTokenProvider>,
        ctx: &mut Self::Context,
    ) {
        match item {
            PubSubEvent::Message(TopicData::ChatModeratorActions {
                topic,
                reply,
            }) => {
                if let ChatModeratorActionsReply::ModerationAction(
                    ModerationAction {
                        moderation_action: ModerationActionCommand::Untimeout,
                        target_user_id,
                        ..
                    },
                ) = *reply
                {
                    self.timeout_handler
                        .send(RemoveTimeoutMessage {
                            channel_id: topic.channel_id.to_string(),
                            user_id: target_user_id.take(),
                            later: Duration::from_secs(0),
                        })
                        .into_actor(self)
                        .map(|res, _, _| {
                            log_err!(res, "Could not send remove timeout");
                        })
                        .spawn(ctx);
                }
            }
            PubSubEvent::Message(TopicData::VideoPlaybackById {
                topic,
                reply,
            }) => match *reply {
                VideoPlaybackReply::StreamUp { .. } => {
                    log::info!("{} is now live", topic.channel_id);

                    let addr = self.live_addr.clone();
                    async move {
                        log_err!(
                            addr.send(LiveMessage(
                                topic.channel_id.to_string()
                            ))
                            .await,
                            "Could not send live message"
                        );
                    }
                    .into_actor(self)
                    .spawn(ctx);
                }
                VideoPlaybackReply::StreamDown { .. } => {
                    log::info!("{} is now offline", topic.channel_id);

                    let addr = self.live_addr.clone();
                    async move {
                        log_err!(
                            addr.send(OfflineMessage(
                                topic.channel_id.to_string()
                            ))
                            .await,
                            "Could not send offline message"
                        );
                    }
                    .into_actor(self)
                    .spawn(ctx);
                }
                _ => (),
            },
            PubSubEvent::SubError { error, topics } => {
                log::warn!(
                    "Couldn't listen on some topics error={} topics={:?}",
                    error,
                    topics
                );
                log_discord!(
                    "Pubsub",
                    "Couldn't listen on some topics",
                    0xffcc4d,
                    "error" = error,
                    "topics" = format!("`{:?}`", topics)
                );
            }
            PubSubEvent::ProvideError(e) => {
                log_discord!(
                    "Pubsub",
                    "Couldn't provide token",
                    0xffce49,
                    "Error" = e.to_string()
                );
            }
            _ => (),
        };
    }
}

impl Handler<SubMessage> for PubSubActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: SubMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let my_id = CONFIG.twitch.user_id.parse::<u32>().unwrap_or(0);
        let target_id = msg.0.parse::<u32>().unwrap_or(0);
        if !self.client.listen(Self::make_topics(target_id, my_id)) {
            log::warn!("Failed to listen to topics");
        }
    }
}

impl Handler<SubAllMessage> for PubSubActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: SubAllMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let my_id = CONFIG.twitch.user_id.parse::<u32>().unwrap_or(0);
        let topics = msg
            .0
            .iter()
            .flat_map(|user| {
                Self::make_topics(user.parse::<u32>().unwrap_or(0), my_id)
            })
            .collect();

        if !self.client.listen(topics) {
            log::warn!("Failed to listen to topics");
        }
    }
}
