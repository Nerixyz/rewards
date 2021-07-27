use crate::actors::live_actor::LiveActor;
use crate::actors::messages::live_messages::{LiveMessage, OfflineMessage};
use crate::actors::messages::pubsub_messages::{SubAllMessage, SubMessage};
use crate::actors::messages::timeout_messages::RemoveTimeoutMessage;
use crate::actors::timeout_actor::TimeoutActor;
use crate::constants::TWITCH_CLIENT_USER_ID;
use crate::log_err;
use crate::models::config::ConfigEntry;
use crate::services::errors::json_error::JsonError;
use crate::services::sql::SqlReason;
use actix::{
    Actor, ActorFutureExt, Addr, AsyncContext, Context, ContextFutureSpawner, Handler,
    StreamHandler, WrapFuture,
};
use async_trait::async_trait;
use futures::{future, stream::StreamExt};
use sqlx::PgPool;
use std::time::Duration;
use tokio_stream::wrappers::UnboundedReceiverStream;
use twitch_api2::pubsub::Topic as TopicDef;
use twitch_pubsub::{
    moderation::{
        ChatModeratorActions, ChatModeratorActionsReply, ModerationAction, ModerationActionCommand,
    },
    video_playback::{VideoPlaybackById, VideoPlaybackReply},
    ClientConfig, PubsubClient, ServerMessage, TokenProvider, Topic, TopicData,
};

#[derive(Debug)]
struct PubsubTokenProvider(PgPool);

impl PubsubTokenProvider {
    async fn get_token(&self) -> Result<String, <Self as TokenProvider>::Error> {
        ConfigEntry::get_user_token(&self.0)
            .await
            .map(|conf| conf.access_token)
    }
}

#[async_trait]
impl TokenProvider for PubsubTokenProvider {
    type Error = JsonError<SqlReason>;

    async fn provide_token(&self, _: &Topic) -> Result<Option<String>, Self::Error> {
        Ok(Some(self.get_token().await?))
    }

    async fn provide_many(
        &self,
        topics: Vec<Topic>,
    ) -> Result<Vec<(Vec<Topic>, Option<String>)>, Self::Error> {
        Ok(vec![(topics, Some(self.get_token().await?))])
    }
}

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
        let my_id = TWITCH_CLIENT_USER_ID.parse::<u32>().unwrap_or(0);
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
        let my_id = TWITCH_CLIENT_USER_ID.parse::<u32>().unwrap_or(0);
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
