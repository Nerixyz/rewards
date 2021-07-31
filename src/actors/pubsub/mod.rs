use std::time::Duration;

use actix::{
    Actor, ActorFutureExt, Addr, AsyncContext, Context, ContextFutureSpawner, Handler,
    StreamHandler, WrapFuture,
};
use futures::{future, stream::StreamExt};
use sqlx::PgPool;
use token_provider::PubsubTokenProvider;
use tokio_stream::wrappers::UnboundedReceiverStream;
use twitch_pubsub::{
    moderation::{
        ChatModeratorActions, ChatModeratorActionsReply, ModerationAction, ModerationActionCommand,
    },
    video_playback::{VideoPlaybackById, VideoPlaybackReply},
    ClientConfig, PubsubClient, ServerMessage, Topic, TopicData, TopicDef,
};

use crate::{
    actors::{
        live::{LiveActor, LiveMessage, OfflineMessage},
        timeout::TimeoutActor,
    },
    config::CONFIG,
    log_err,
};

mod messages;
mod token_provider;
use crate::actors::timeout::RemoveTimeoutMessage;
pub use messages::*;

pub struct PubSubActor {
    live_addr: Addr<LiveActor>,
    timeout_handler: Addr<TimeoutActor>,

    client: PubsubClient<PubsubTokenProvider>,
}

impl PubSubActor {
    pub fn run(
        pool: PgPool,
        live_addr: Addr<LiveActor>,
        timeout_handler: Addr<TimeoutActor>,
    ) -> Addr<Self> {
        let config = ClientConfig::new(PubsubTokenProvider(pool));
        let (incoming, client) = PubsubClient::new(config);

        Self::create(|ctx| {
            let stream = UnboundedReceiverStream::new(incoming).filter_map(|s| {
                future::ready(match s {
                    ServerMessage::Message { data } => Some(data),
                    _ => None,
                })
            });
            ctx.add_stream(stream);

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

impl StreamHandler<TopicData> for PubSubActor {
    fn handle(&mut self, item: TopicData, ctx: &mut Self::Context) {
        match item {
            TopicData::ChatModeratorActions { topic, reply } => {
                if let ChatModeratorActionsReply::ModerationAction(ModerationAction {
                    moderation_action: ModerationActionCommand::Untimeout,
                    target_user_id,
                    ..
                }) = *reply
                {
                    self.timeout_handler
                        .send(RemoveTimeoutMessage {
                            channel_id: topic.channel_id.to_string(),
                            user_id: target_user_id.into_string(),
                            later: Duration::from_secs(0),
                        })
                        .into_actor(self)
                        .map(|res, _, _| {
                            log_err!(res, "Could not send remove timeout");
                        })
                        .spawn(ctx);
                }
            }
            TopicData::VideoPlaybackById { topic, reply } => match *reply {
                VideoPlaybackReply::StreamUp { .. } => {
                    log::info!("{} is now live", topic.channel_id);

                    let addr = self.live_addr.clone();
                    async move {
                        log_err!(
                            addr.send(LiveMessage(topic.channel_id.to_string())).await,
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
                            addr.send(OfflineMessage(topic.channel_id.to_string()))
                                .await,
                            "Could not send offline message"
                        );
                    }
                    .into_actor(self)
                    .spawn(ctx);
                }
                _ => (),
            },
            _ => (),
        };
    }
}

impl Handler<SubMessage> for PubSubActor {
    type Result = ();

    fn handle(&mut self, msg: SubMessage, ctx: &mut Self::Context) -> Self::Result {
        let my_id = CONFIG.twitch.user_id.parse::<u32>().unwrap_or(0);
        let target_id = msg.0.parse::<u32>().unwrap_or(0);
        let client = self.client.clone();
        async move {
            log_err!(
                client
                    .listen_many(Self::make_topics(target_id, my_id))
                    .await,
                "Could not listen"
            );
        }
        .into_actor(self)
        .spawn(ctx);
    }
}

impl Handler<SubAllMessage> for PubSubActor {
    type Result = ();

    fn handle(&mut self, msg: SubAllMessage, ctx: &mut Self::Context) -> Self::Result {
        let my_id = CONFIG.twitch.user_id.parse::<u32>().unwrap_or(0);
        let topics = msg
            .0
            .iter()
            .map(|user| Self::make_topics(user.parse::<u32>().unwrap_or(0), my_id))
            .flatten()
            .collect();

        let client = self.client.clone();
        async move {
            log_err!(client.listen_many(topics).await, "Could not listen");
        }
        .into_actor(self)
        .spawn(ctx);
    }
}
