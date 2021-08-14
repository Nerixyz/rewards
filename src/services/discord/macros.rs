#[macro_export]
macro_rules! send_discord {
    ($discord:expr, $user_id:expr, $embed:expr) => {{
        let msg = crate::actors::discord::LogToDiscordMessage {
            user_id: $user_id,
            embed: $embed,
        };
        tokio::spawn(async move {
            crate::log_err!($discord.send(msg).await, "Could not send");
        });
    };};
}

#[macro_export]
macro_rules! embed_builder {
    ($title:literal, $description:expr, $color:expr, $($name:literal = $value:expr),* $(,)?) => {
        {
            crate::embed_builder!($title, $description, $color, $($name = $value),*; image = None, url = None)
        }
    };
    ($title:literal, $description:expr, $color:expr, $($name:literal = $value:expr),*; image = $image:expr, url = $url:expr $(,)?) => {
        {
            crate::services::discord::Embed {
                title: $title.into(),
                description: $description.into(),
                fields: vec![$(crate::services::discord::EmbedField::new($name, $value),)*],
                color: (($color) as u32),
                image: $image.map(|url: String| crate::services::discord::EmbedImage { url }),
                url: $url.into(),
            }
        }
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
            let msg = crate::services::discord::WebhookReq::Embeds(vec![crate::embed_builder!($title, $description, $color, $($name = $value),*)]);
            tokio::spawn(async move {
                crate::log_err!(crate::services::discord::send_webhook_message(&msg).await, "Could not send webhook message");
            });
        };}
    };
}
