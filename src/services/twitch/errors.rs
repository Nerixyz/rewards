use crate::services::h2h::H2hExt as _;
use actix_web::{error, http::StatusCode, HttpResponse};
use derive_more::Display;
use errors::json_error::JsonError;
use twitch_api::helix::{
    ClientRequestError, CreateRequestError, HelixRequestDeleteError,
    HelixRequestGetError, HelixRequestPatchError, HelixRequestPostError,
    HelixRequestPutError,
};

#[derive(Display, Debug)]
pub enum TwitchApiError {
    #[display(fmt = "Reqwest Error")]
    ReqwestError,
    #[display(fmt = "Http crate error")]
    HttpCrateError,
    #[display(fmt = "No Page")]
    NoPage,
    #[display(fmt = "Invalid Uri")]
    InvalidUri,
    #[display(fmt = "Invalid Utf8 received")]
    Utf8,
    #[display(fmt = "Serde error")]
    Serde,
    #[display(fmt = "TwitchError: {} - {}", _0, _1)]
    Response(StatusCode, String),
    #[display(fmt = "Some custom error")]
    Custom,
    #[display(fmt = "Unknown and unexpected Twitch error (no further info)")]
    Unknown,
    #[display(fmt = "{}", _0)]
    Other(String),
}

impl std::error::Error for TwitchApiError {}
impl error::ResponseError for TwitchApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Response(status, _) => *status,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        JsonError::new(format!("{}", self), self.status_code()).error_response()
    }
}

impl From<ClientRequestError<reqwest::Error>> for TwitchApiError {
    fn from(e: ClientRequestError<reqwest::Error>) -> Self {
        // Although `ClientRequestError` is Display and Debug, it's too much to expose, so here we're converting...
        // Still. It should be improved.
        match e {
            ClientRequestError::RequestError(_) => Self::ReqwestError,
            ClientRequestError::NoPage => Self::NoPage,
            ClientRequestError::CreateRequestError(e) => match e {
                CreateRequestError::HttpError(_) => Self::HttpCrateError,
                CreateRequestError::SerializeError(_) => Self::Serde,
                CreateRequestError::InvalidUri(_) => Self::InvalidUri,
                CreateRequestError::Custom(_) => Self::Custom,
                _ => Self::Unknown,
            },
            ClientRequestError::HelixRequestGetError(e) => match e {
                HelixRequestGetError::Error {
                    status, message, ..
                } => Self::Response(status.convert(), message),
                HelixRequestGetError::Utf8Error(_, _, _) => Self::Utf8,
                HelixRequestGetError::DeserializeError(_, _, _, _) => {
                    Self::Serde
                }
                HelixRequestGetError::InvalidResponse {
                    status,
                    reason,
                    ..
                } => Self::Response(status.convert(), reason.to_string()),
                _ => Self::Unknown,
            },
            ClientRequestError::HelixRequestPutError(e) => match e {
                HelixRequestPutError::Error {
                    status, message, ..
                } => Self::Response(status.convert(), message),
                HelixRequestPutError::Utf8Error(_, _, _) => Self::Utf8,
                HelixRequestPutError::DeserializeError(_, _, _, _) => {
                    Self::Serde
                }
                HelixRequestPutError::InvalidResponse {
                    status,
                    reason,
                    ..
                } => Self::Response(status.convert(), reason.to_string()),
                _ => Self::Unknown,
            },
            ClientRequestError::HelixRequestPostError(e) => match e {
                HelixRequestPostError::Error {
                    status, message, ..
                } => Self::Response(status.convert(), message),
                HelixRequestPostError::Utf8Error(_, _, _) => Self::Utf8,
                HelixRequestPostError::DeserializeError(_, _, _, _) => {
                    Self::Serde
                }
                HelixRequestPostError::InvalidResponse {
                    status,
                    reason,
                    ..
                } => Self::Response(status.convert(), reason.to_string()),
                _ => Self::Unknown,
            },
            ClientRequestError::HelixRequestPatchError(e) => match e {
                HelixRequestPatchError::Error {
                    status, message, ..
                } => Self::Response(status.convert(), message),
                HelixRequestPatchError::Utf8Error(_, _, _) => Self::Utf8,
                HelixRequestPatchError::DeserializeError(_, _, _, _) => {
                    Self::Serde
                }
                HelixRequestPatchError::InvalidResponse {
                    status,
                    reason,
                    ..
                } => Self::Response(status.convert(), reason.to_string()),
                _ => Self::Unknown,
            },
            ClientRequestError::HelixRequestDeleteError(e) => match e {
                HelixRequestDeleteError::Error {
                    status, message, ..
                } => Self::Response(status.convert(), message),
                HelixRequestDeleteError::Utf8Error(_, _, _) => Self::Utf8,
                HelixRequestDeleteError::InvalidResponse {
                    status,
                    reason,
                    ..
                } => Self::Response(status.convert(), reason.to_string()),
                _ => Self::Unknown,
            },
            ClientRequestError::Custom(_) => Self::Custom,
            ClientRequestError::HyperError(_) => Self::ReqwestError,
            _ => Self::Unknown,
        }
    }
}
