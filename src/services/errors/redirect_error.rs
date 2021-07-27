use std::fmt::{Debug, Display, Formatter};

use actix_web::{body::Body, http::StatusCode, HttpResponse, ResponseError};

#[derive(Debug)]
pub struct RedirectError<L, D>
where
    L: Display + Debug,
    D: Debug + Display,
{
    pub location: L,
    pub data: Option<D>,
}

impl<L: Display + Debug, D: Debug + Display> std::error::Error for RedirectError<L, D> {}
impl<L: Display + Debug, D: Debug + Display> RedirectError<L, D> {
    pub fn new(location: L, data: Option<D>) -> Self {
        Self { location, data }
    }
    pub fn simple(location: L) -> Self {
        Self {
            location,
            data: None,
        }
    }
}

impl<L: Display + Debug, D: Debug + Display> Display for RedirectError<L, D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Redirect: {} Data: {:?}", self.location, self.data)
    }
}

impl<L: Display + Debug, D: Debug + Display> ResponseError for RedirectError<L, D> {
    fn status_code(&self) -> StatusCode {
        StatusCode::FOUND
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header((
                "location",
                match &self.data {
                    Some(data) => format!("{}#{}", self.location, data),
                    None => format!("{}", self.location),
                },
            ))
            .body(Body::None)
    }
}
