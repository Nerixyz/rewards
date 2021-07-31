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

#[macro_export]
macro_rules! log_discord {
    ($message:expr) => {
        {if crate::config::CONFIG.log.webhook_url.is_some() {
            let msg = crate::services::discord::WebhookReq::Content($message.into());
            tokio::spawn(async move {
                crate::log_err!(crate::services::discord::send_webhook_message(&msg).await, "Could not send webhook message");
            });
        };}
    };
    ($title:literal, $description:expr, $($name:literal = $value:expr),*) => {
        crate::log_discord!($title, $description, 0xe91916, $($name = $value),*)
    };
    ($title:literal, $description:expr, $color:expr, $($name:literal = $value:expr),*) => {
        {if crate::config::CONFIG.log.webhook_url.is_some() {
            let msg = crate::services::discord::WebhookReq::Embeds(vec![crate::services::discord::Embed {
                title: $title.into(),
                description: $description.into(),
                fields: vec![$(crate::services::discord::EmbedField::new($name, $value),)*],
                color: (($color) as u32),
            }]);
            tokio::spawn(async move {
                crate::log_err!(crate::services::discord::send_webhook_message(&msg).await, "Could not send webhook message");
            });
        };}
    };
}
