use crate::CONFIG;
use actix_web::{dev::Payload, web::BytesMut, FromRequest, HttpRequest};
use chrono::{DateTime, Duration, Utc};
use futures_util::{future::Either, StreamExt};
use hmac::{Hmac, Mac};
use serde::{de::IntoDeserializer, Deserialize};
use sha2::Sha256;
use std::{
    future::{ready, Future, Ready},
    pin::Pin,
};
use twitch_api2::eventsub::{Event, EventType};

type HmacSha256 = Hmac<Sha256>;

pub struct EventsubPayload(pub Event);

impl FromRequest for EventsubPayload {
    type Error = actix_web::Error;
    #[allow(clippy::type_complexity)]
    type Future = Either<
        Ready<Result<Self, Self::Error>>,
        Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>,
    >;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        match read_headers(req) {
            None => Either::Left(ready(Err(errors::ErrorBadRequest(
                "Malformed headers",
            )))),
            Some((headers, mac)) => Either::Right(Box::pin(
                read_verify_payload(headers, mac, payload.take()),
            )),
        }
    }
}

struct PayloadHeaders {
    signature: String,

    version: String,
    event_type: EventType,
    message_type: Vec<u8>,
}

fn read_headers(req: &HttpRequest) -> Option<(PayloadHeaders, HmacSha256)> {
    let mac = init_read_signature(req)?;

    let signature_header = req
        .headers()
        .get("Twitch-Eventsub-Message-Signature")?
        .to_str()
        .ok()
        .filter(|sig| sig.len() > 7 && sig.starts_with("sha256="))?;
    let version_header = req
        .headers()
        .get("Twitch-Eventsub-Subscription-Version")?
        .to_str()
        .ok()?;
    let subscription_type_header = req
        .headers()
        .get("Twitch-Eventsub-Subscription-Type")?
        .to_str()
        .ok()?;
    let message_type_header =
        req.headers().get("Twitch-Eventsub-Message-Type")?;

    Some((
        PayloadHeaders {
            signature: signature_header.to_string(),
            version: version_header.to_string(),
            event_type: EventType::deserialize(
                subscription_type_header.into_deserializer(),
            )
            .map_err(|_: serde::de::value::Error| ())
            .ok()?,
            message_type: message_type_header.as_bytes().to_vec(),
        },
        mac,
    ))
}

fn init_read_signature(req: &HttpRequest) -> Option<HmacSha256> {
    let id_header = req.headers().get("Twitch-Eventsub-Message-Id")?;
    let timestamp_header =
        req.headers().get("Twitch-Eventsub-Message-Timestamp")?;
    let timestamp = timestamp_header
        .to_str()
        .ok()?
        .parse::<DateTime<Utc>>()
        .ok()?;
    if Utc::now() - timestamp > Duration::minutes(10) {
        return None;
    }
    let mut mac =
        HmacSha256::new_from_slice(CONFIG.twitch.eventsub.secret.as_bytes())
            .ok()?;
    mac.update(id_header.as_bytes());
    mac.update(timestamp_header.as_bytes());

    Some(mac)
}

async fn read_verify_payload(
    headers: PayloadHeaders,
    mut mac: HmacSha256,
    mut payload: Payload,
) -> Result<EventsubPayload, actix_web::Error> {
    let mut body = BytesMut::new();
    while let Some(chunk) = payload.next().await {
        // 10Mb
        if body.len() >= 10_000_000 {
            return Err(errors::ErrorImATeapot("yeah no, that's too much"));
        }
        body.extend_from_slice(&chunk?);
    }
    mac.update(body.as_ref());
    let bytes = mac.finalize().into_bytes();

    if hex::encode(bytes) != headers.signature[7..] {
        return Err(errors::ErrorUnauthorized("Bad signature"));
    }

    macro_rules! match_event {
        ($($module:ident::$event:ident);* $(;)?) => {{

            #[deny(unreachable_patterns)]
            match (headers.version.as_str(), headers.event_type) {
                $(  (<twitch_api2::eventsub::$module::$event as twitch_api2::eventsub::EventSubscription>::VERSION, <twitch_api2::eventsub::$module::$event as twitch_api2::eventsub::EventSubscription>::EVENT_TYPE) => {
                    EventsubPayload(twitch_api2::eventsub::Event::$event(twitch_api2::eventsub::Payload::parse_request((&headers.message_type).into(), body.as_ref().into()).map_err(|_| errors::ErrorBadRequest("cannot parse payload"))?))
                }  )*
                (..) => return Err(errors::ErrorBadRequest("not implemented"))
            }
        }}
    }

    Ok(match_event! {
        channel::ChannelUpdateV1;
        channel::ChannelFollowV2;
        channel::ChannelSubscribeV1;
        channel::ChannelCheerV1;
        channel::ChannelBanV1;
        channel::ChannelUnbanV1;
        channel::ChannelPointsCustomRewardAddV1;
        channel::ChannelPointsCustomRewardUpdateV1;
        channel::ChannelPointsCustomRewardRemoveV1;
        channel::ChannelPointsCustomRewardRedemptionAddV1;
        channel::ChannelPointsCustomRewardRedemptionUpdateV1;
        channel::ChannelPollBeginV1;
        channel::ChannelPollProgressV1;
        channel::ChannelPollEndV1;
        channel::ChannelPredictionBeginV1;
        channel::ChannelPredictionProgressV1;
        channel::ChannelPredictionLockV1;
        channel::ChannelPredictionEndV1;
        channel::ChannelRaidV1;
        channel::ChannelSubscriptionEndV1;
        channel::ChannelSubscriptionGiftV1;
        channel::ChannelSubscriptionMessageV1;
        channel::ChannelGoalBeginV1;
        channel::ChannelGoalProgressV1;
        channel::ChannelGoalEndV1;
        channel::ChannelHypeTrainBeginV1;
        channel::ChannelHypeTrainProgressV1;
        channel::ChannelHypeTrainEndV1;
        stream::StreamOnlineV1;
        stream::StreamOfflineV1;
        user::UserUpdateV1;
        user::UserAuthorizationGrantV1;
        user::UserAuthorizationRevokeV1;
    })
}
