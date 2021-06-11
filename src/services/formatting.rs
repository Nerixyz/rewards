const S_IN_MNTH: u64 = 2628003; // 2628002,88 seconds according to Google

/// This is basically the Short formatting if [`timeago`](https://docs.rs/timeago/0.0.2/src/timeago/lib.rs.html#24-98) without the "ago".
pub fn human_format_duration(duration: &chrono::Duration) -> String {
    let secs = duration.num_seconds();
    if secs > 0 {
        format!("{} ago", format_secs_short(secs as u64))
    } else if secs < 0 {
        let actual_secs = (-secs) as u64;
        format!("in {}", format_secs_short(actual_secs))
    } else {
        "now".into()
    }
}

fn format_secs_short(secs: u64) -> String {
    match secs {
        0 => "0 seconds".into(), // never
        1 => "1 second".into(),
        x if x > 1 && x < 60 => format!("{} seconds", x),
        x if x >= 60 && x < 120 => "1 minute".into(),
        x if x >= 120 && x < 60 * 60 => format!("{} minutes", x / 60),
        x if x >= 60 * 60 && x < 60 * 60 * 2 => "1 hour".into(),
        x if x >= 60 * 60 * 2 && x < 60 * 60 * 24 => format!("{} hours", x / 60 / 60),
        x if x >= 60 * 60 * 24 && x < 60 * 60 * 24 * 2 => "1 day".into(),
        x if x >= 60 * 60 * 24 * 2 && x < S_IN_MNTH => {
            format!("{} days", x / 60 / 60 / 24)
        }
        x if x >= S_IN_MNTH && x < 2 * S_IN_MNTH => "~1 month".into(),
        x if x >= 2 * S_IN_MNTH && x < 12 * S_IN_MNTH => {
            format!("~{} months", x / S_IN_MNTH)
        }
        x if x >= 12 * S_IN_MNTH && x < 12 * 2 * S_IN_MNTH => "~1 year".into(),
        x => format!("~{} years", x / 12 / S_IN_MNTH),
    }
}