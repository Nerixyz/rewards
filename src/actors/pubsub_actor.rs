use crate::actors::live_actor::LiveActor;
use crate::actors::messages::live_messages::{LiveMessage, OfflineMessage};
use crate::actors::messages::pubsub_messages::{
    ModMessage, PongMessage, ReconnectMessage, SubAllMessage, SubMessage, VideoPlaybackMessage,
};
use crate::actors::messages::timeout_messages::RemoveTimeoutMessage;
use crate::actors::timeout_actor::TimeoutActor;
use crate::constants::TWITCH_CLIENT_USER_ID;
use crate::log_err;
use crate::models::config::ConfigEntry;
use actix::{Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, WrapFuture};
use anyhow::Result as AnyResult;
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use futures_util::StreamExt;
use rand::distributions::Alphanumeric;
use rand::Rng;
use sqlx::PgPool;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use twitch_api2::pubsub::moderation::{
    ChatModeratorActions, ChatModeratorActionsReply, ModerationActionCommand,
};
use twitch_api2::pubsub::video_playback::{VideoPlaybackById, VideoPlaybackReply};
use twitch_api2::pubsub::{Response, TopicData, Topics};

#[derive(Debug)]
enum InternalPubSubMessage {
    Ping,
    Reconnect,
    /// This is already the listen command at the auth token is needed
    Listen((String, String)),
    /// This is already the listen command at the auth token is needed
    ListenAll(Vec<(String, String)>),
}

pub struct PubSubActor {
    pool: PgPool,

    live_addr: Addr<LiveActor>,
    timeout_handler: Addr<TimeoutActor>,

    sender: UnboundedSender<InternalPubSubMessage>,
    receiver: Option<UnboundedReceiver<InternalPubSubMessage>>,
    last_pong: Option<Instant>,
    last_ping: Option<Instant>,
}

impl PubSubActor {
    pub fn new(
        pool: PgPool,
        live_addr: Addr<LiveActor>,
        timeout_handler: Addr<TimeoutActor>,
    ) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            pool,
            live_addr,
            timeout_handler,

            sender,
            receiver: Some(receiver),
            last_pong: None,
            last_ping: None,
        }
    }

    pub async fn get_access_token_and_id(pool: &PgPool) -> AnyResult<(String, u32)> {
        let conf = ConfigEntry::get_user_token(pool).await?;
        Ok((conf.access_token, TWITCH_CLIENT_USER_ID.parse()?))
    }

    pub fn make_topics_for_channel(
        channel_id: u32,
        this_user_id: u32,
        access_token: &str,
    ) -> String {
        let nonce = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect::<String>();
        twitch_api2::pubsub::listen_command(
            &[
                Topics::VideoPlaybackById(VideoPlaybackById { channel_id }),
                Topics::ChatModeratorActions(ChatModeratorActions {
                    channel_id,
                    user_id: this_user_id,
                }),
            ],
            access_token,
            Some(nonce.as_str()),
        )
        .unwrap_or_else(|_| "{}".to_string())
    }

    pub async fn hydrate_all(topics: &HashSet<String>, pool: &PgPool) -> Vec<String> {
        match Self::get_access_token_and_id(&pool).await {
            Ok((token, id)) => topics
                .iter()
                .map(|chan| Self::make_topics_for_channel(chan.parse().unwrap_or(0), id, &token))
                .collect(),
            Err(e) => {
                log::error!("Could not get access token: {}", e);
                vec![]
            }
        }
    }
}

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(2 * 60);
const TIMEOUT: Duration = Duration::from_secs(15);

impl Actor for PubSubActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        let mut receiver = self.receiver.take().expect("This was set in Self::new");
        let pool = self.pool.clone();

        let (send_tx, mut send_rx) = mpsc::unbounded_channel();

        // This thread sends messages to the websocket.
        // Therefore it has to receive messages from the actor and listen to new clients from the read-thread.
        ctx.spawn(async move {
            let mut topics = HashSet::<String>::new();
            let mut sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message> = send_rx.recv().await.expect("Pubsub sender-channel closed without any sender");
            log::info!("Starting pubsub sender");
            loop {
                tokio::select! {
                    Some(new_sender) = send_rx.recv() => {
                        // The client reconnected; update the sender and listen to the previous topics
                        sender = new_sender;
                        for msg in Self::hydrate_all(&topics, &pool).await {
                            log_err!(sender.send(Message::Text(msg.to_string())).await, "Could not listen to all");
                        }
                    },
                    Some(message) = receiver.recv() => {
                        match message {
                            InternalPubSubMessage::Reconnect => {
                                log_err!(sender.close().await, "Could not close");
                            },
                            InternalPubSubMessage::Ping => {
                                let json = serde_json::json!({
                                    "type": "PING"
                                }).to_string();
                                log_err!(sender.send(Message::Text(json)).await, "Could not send PING");
                            },
                            InternalPubSubMessage::Listen((chan, msg)) => {
                                topics.insert(chan);
                                log_err!(sender.send(Message::Text(msg)).await, "Could not listen");
                            },
                            InternalPubSubMessage::ListenAll(msgs) => {
                                for (chan, msg) in msgs {
                                    topics.insert(chan);
                                    log_err!(sender.send(Message::Text(msg)).await, "Could not listen to all");
                                }
                            }
                        };
                    },
                    else => {
                        log::info!("else");
                        break;
                    }
                }
            }
        }.into_actor(self));
        // This thread (re-)connects to PubSub and emits messages
        ctx.spawn(
            async move {
                // This is the reconnect loop
                loop {
                    match connect_async("wss://pubsub-edge.twitch.tv").await {
                        Ok((stream, _)) => {
                            let (send, mut read) = stream.split();
                            // Send the send-half to the send-thread
                            if let Err(e) = send_tx.send(send) {
                                log::error!("Aborting PubSub: {}", e);
                            }

                            // This is the main read-loop
                            'inner: while let Some(Ok(msg)) = read.next().await {
                                let msg = match msg {
                                    Message::Text(text) => text,
                                    _ => continue 'inner,
                                };
                                let response = match serde_json::from_str::<Response>(&msg) {
                                    Ok(res) => res,
                                    Err(e) => {
                                        log::warn!("Could not read json: {}", e);
                                        continue 'inner;
                                    }
                                };
                                match response {
                                    Response::Pong => {
                                        addr.do_send(PongMessage);
                                    }
                                    Response::Reconnect => {
                                        addr.do_send(ReconnectMessage);
                                    }
                                    Response::Response(res) => {
                                        if let Some(e) = res.error.as_ref() {
                                            if !e.is_empty() {
                                                log::warn!("Could not listen: {:?}", res);
                                            }
                                        }
                                    }
                                    Response::Message {
                                        data:
                                            TopicData::VideoPlaybackById {
                                                reply,
                                                topic: VideoPlaybackById { channel_id },
                                            },
                                    } => {
                                        addr.do_send(VideoPlaybackMessage(
                                            channel_id.to_string(),
                                            *reply,
                                        ));
                                    }
                                    Response::Message {
                                        data:
                                            TopicData::ChatModeratorActions {
                                                reply,
                                                topic: ChatModeratorActions { channel_id, .. },
                                            },
                                    } => {
                                        if let ChatModeratorActionsReply::ModerationAction(act) =
                                            *reply
                                        {
                                            addr.do_send(ModMessage(channel_id.to_string(), act));
                                        };
                                    }
                                    _ => {}
                                }
                            }
                            log::info!("Stream closed");
                        }
                        Err(e) => {
                            log::warn!("Pubsub error: {}", e);
                        }
                    }
                    log::info!("Pubsub reconnect");
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            }
            .into_actor(self),
        );

        ctx.run_interval(HEARTBEAT_INTERVAL, |this, _ctx| {
            if let (Some(last_ping), Some(last_pong)) = (this.last_ping, this.last_pong) {
                if last_pong.duration_since(last_ping) > TIMEOUT {
                    log_err!(
                        this.sender.send(InternalPubSubMessage::Reconnect),
                        "Could not send reconnect-message"
                    );
                    return;
                }
            }
            this.last_ping = Some(Instant::now());
            log_err!(
                this.sender.send(InternalPubSubMessage::Ping),
                "Could not send ping"
            );
        });
    }
}

impl Handler<PongMessage> for PubSubActor {
    type Result = ();

    fn handle(&mut self, _msg: PongMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.last_pong = Some(Instant::now());
    }
}
impl Handler<SubMessage> for PubSubActor {
    type Result = ();

    fn handle(&mut self, msg: SubMessage, ctx: &mut Self::Context) -> Self::Result {
        let pool = self.pool.clone();
        let sender = self.sender.clone();
        ctx.spawn(
            async move {
                match Self::get_access_token_and_id(&pool).await {
                    Ok((token, id)) => {
                        let topic =
                            Self::make_topics_for_channel(msg.0.parse().unwrap_or(0), id, &token);
                        let data = (msg.0, topic);
                        log_err!(
                            sender.send(InternalPubSubMessage::Listen(data)),
                            "Could not send subscribe-message"
                        );
                    }
                    Err(e) => {
                        log::error!("Could not get access token: {}", e);
                    }
                };
            }
            .into_actor(self),
        );
    }
}
impl Handler<SubAllMessage> for PubSubActor {
    type Result = ();

    fn handle(&mut self, msg: SubAllMessage, ctx: &mut Self::Context) -> Self::Result {
        let pool = self.pool.clone();
        let sender = self.sender.clone();
        ctx.spawn(
            async move {
                match Self::get_access_token_and_id(&pool).await {
                    Ok((token, id)) => {
                        let data: Vec<(String, String)> = msg
                            .0
                            .into_iter()
                            .map(|chan| {
                                let topic = Self::make_topics_for_channel(
                                    chan.parse().unwrap_or(0),
                                    id,
                                    &token,
                                );
                                (chan, topic)
                            })
                            .collect();
                        log_err!(
                            sender.send(InternalPubSubMessage::ListenAll(data)),
                            "Could not send subscribe-message"
                        );
                    }
                    Err(e) => {
                        log::error!("Could not get access token: {}", e);
                    }
                };
            }
            .into_actor(self),
        );
    }
}

impl Handler<ReconnectMessage> for PubSubActor {
    type Result = ();

    fn handle(&mut self, _msg: ReconnectMessage, _ctx: &mut Self::Context) -> Self::Result {
        log_err!(
            self.sender.send(InternalPubSubMessage::Reconnect),
            "Could not send reconnect-message"
        );
    }
}
impl Handler<VideoPlaybackMessage> for PubSubActor {
    type Result = ();

    fn handle(&mut self, msg: VideoPlaybackMessage, ctx: &mut Self::Context) -> Self::Result {
        let addr = self.live_addr.clone();
        match msg.1 {
            VideoPlaybackReply::StreamUp { .. } => {
                log::info!("{} is now live", msg.0);
                ctx.spawn(
                    async move {
                        log_err!(
                            addr.send(LiveMessage(msg.0)).await,
                            "Could not send live-message"
                        );
                    }
                    .into_actor(self),
                );
            }
            VideoPlaybackReply::StreamDown { .. } => {
                log::info!("{} is offline", msg.0);
                ctx.spawn(
                    async move {
                        log_err!(
                            addr.send(OfflineMessage(msg.0)).await,
                            "Could not send offline-message"
                        );
                    }
                    .into_actor(self),
                );
            }
            _ => (),
        };
    }
}

impl Handler<ModMessage> for PubSubActor {
    type Result = ();

    fn handle(&mut self, msg: ModMessage, ctx: &mut Self::Context) -> Self::Result {
        if msg.1.moderation_action == ModerationActionCommand::Untimeout {
            log::info!("Untimeout in {} for {}", msg.0, msg.1.target_user_id);
            ctx.spawn(
                self.timeout_handler
                    .send(RemoveTimeoutMessage {
                        channel_id: msg.0,
                        user_id: msg.1.target_user_id,
                        later: Duration::from_secs(0),
                    })
                    .into_actor(self)
                    .map(|res, _, _| {
                        match res {
                            Err(e) => log::warn!("Could not send remove timeout message: {}", e),
                            _ => (),
                        };
                    }),
            );
        };
    }
}
