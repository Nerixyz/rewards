use crate::services::twitch::errors::ErrorResponse;
use twitch_api2::HelixClient;

pub mod errors;
pub mod eventsub;
pub mod requests;

pub type RHelixClient<'a> = HelixClient<'a, reqwest::Client>;
pub type HelixResult<T> = Result<T, ErrorResponse>;
