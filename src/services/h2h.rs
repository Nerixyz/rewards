/// Utility to convert between http 1/0.2 types,
/// currently implemented for StatusCode
pub trait H2hExt {
    type Other;

    fn convert(self) -> Self::Other;
}

impl H2hExt for actix_web::http::StatusCode {
    type Other = reqwest::StatusCode;

    fn convert(self) -> Self::Other {
        // this should never fail
        Self::Other::from_u16(self.as_u16()).unwrap_or_default()
    }
}

impl H2hExt for reqwest::StatusCode {
    type Other = actix_web::http::StatusCode;

    fn convert(self) -> Self::Other {
        // this should never fail
        Self::Other::from_u16(self.as_u16()).unwrap_or_default()
    }
}
