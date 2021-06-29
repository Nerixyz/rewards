use std::fmt::{Debug, Display, Formatter};

use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct JsonError<T>
where
    T: Display + Debug + Serialize,
{
    pub error: T,
    #[serde(skip)]
    pub status: StatusCode,
}

impl<T: Debug + Display + Serialize> std::error::Error for JsonError<T> {}
impl<T: Display + Debug + Serialize> JsonError<T> {
    pub fn new(error: T, status: StatusCode) -> Self {
        Self { error, status }
    }
}

impl<T: Debug + Display + Serialize> Display for JsonError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
        )
    }
}

impl<T: Debug + Display + Serialize> ResponseError for JsonError<T> {
    fn status_code(&self) -> StatusCode {
        self.status
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(("content-type", "application/json"))
            .body(
                serde_json::to_string(self)
                    .unwrap_or_else(|_| "{\"error\":\"serde error\"}".to_string()),
            )
    }
}
