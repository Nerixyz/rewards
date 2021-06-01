#[macro_export]
macro_rules! log_err {
    ($result:expr, $format:literal) => {
            if let Err(__e) = $result {
                log::warn!("{}: {}", $format, __e);
            };
    };
}

