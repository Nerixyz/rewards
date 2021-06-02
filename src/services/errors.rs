#[macro_export]
macro_rules! log_err {
    ($result:expr, $format:literal) => {
            if let Err(__e) = $result {
                log::warn!("{}: {}", $format, __e);
            };
    };
    ($result:expr, $format:literal, $($arg:tt)+) => {
            if let Err(e) = $result {
                log::warn!($format, $($arg)+, e);
            };
    };
}
