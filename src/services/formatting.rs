const S_IN_MNTH: u64 = 2628003; // 2628002,88 seconds according to Google

/// This is basically the Short formatting if [`timeago`](https://docs.rs/timeago/0.0.2/src/timeago/lib.rs.html#24-98) without the "ago".
pub fn human_format_duration(duration: &chrono::Duration) -> String {
    match duration.num_seconds() {
        secs if secs > 0 => format!("{} ago", format_secs_short(secs as u64)),
        secs if secs < 0 => {
            let actual_secs = (-secs) as u64;
            format!("in {}", format_secs_short(actual_secs))
        }
        _ => "now".into(),
    }
}

fn format_secs_short(secs: u64) -> String {
    match secs {
        0 => "0 seconds".into(), // never
        1 => "1 second".into(),
        x if x > 1 && x < 60 => format!("{} seconds", x),
        x if (60..120).contains(&x) => "1 minute".into(),
        x if (120..60 * 60).contains(&x) => format!("{} minutes", x / 60),
        x if (60 * 60..60 * 60 * 2).contains(&x) => "1 hour".into(),
        x if (60 * 60 * 2..60 * 60 * 24).contains(&x) => {
            format!("{} hours", x / 60 / 60)
        }
        x if (60 * 60 * 24..60 * 60 * 24 * 2).contains(&x) => "1 day".into(),
        x if (60 * 60 * 24 * 2..S_IN_MNTH).contains(&x) => {
            format!("{} days", x / 60 / 60 / 24)
        }
        x if (S_IN_MNTH..2 * S_IN_MNTH).contains(&x) => "~1 month".into(),
        x if (2 * S_IN_MNTH..12 * S_IN_MNTH).contains(&x) => {
            format!("~{} months", x / S_IN_MNTH)
        }
        x if (12 * S_IN_MNTH..12 * 2 * S_IN_MNTH).contains(&x) => {
            "~1 year".into()
        }
        x => format!("~{} years", x / 12 / S_IN_MNTH),
    }
}
