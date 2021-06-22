use anyhow::{Error as AnyError, Result as AnyResult};
use lazy_static::lazy_static;
use regex::Regex;

pub mod execute;
mod reply;
pub mod save;
pub mod verify;

fn extract_username(str: &str) -> AnyResult<String> {
    lazy_static! {
        static ref USERNAME_REGEX: Regex = Regex::new("@([\\w_]+)").expect("must compile");
    }

    let str = str.trim();

    if !str.contains(' ') {
        return Ok(str.replace("@", ""));
    }

    USERNAME_REGEX
        .captures(str)
        .map(|m| m.get(0))
        .flatten()
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| AnyError::msg("No user submitted"))
}

fn extract_bttv_id(str: &str) -> AnyResult<&str> {
    lazy_static! {
        static ref BTTV_REGEX: Regex = Regex::new(
            "(?:^| )(?:https?://)?(?:betterttv\\.com/)?(?:emotes/)?([a-f0-9]{24})(?:$| )"
        )
        .expect("must compile");
    }
    BTTV_REGEX
        .captures(str)
        .map(|c| c.iter().nth(1).flatten().map(|m| m.as_str()))
        .flatten()
        .ok_or_else(|| AnyError::msg("Could not find an emote code there!"))
}

fn extract_ffz_id(str: &str) -> AnyResult<&str> {
    lazy_static! {
        static ref FFZ_REGEX: Regex = Regex::new(
            "(?:^| )(?:https?://)?(?:www\\.)?(?:frankerfacez\\.com/)?(?:emoticon/)(\\d+)(?:-[\\w_!]+)?(?:$| )"
        )
        .expect("must compile");
    }
    FFZ_REGEX
        .captures(str)
        .map(|c| c.iter().nth(1).flatten().map(|m| m.as_str()))
        .flatten()
        .ok_or_else(|| AnyError::msg("Could not find an emote there!"))
}

fn extract_seventv_id(str: &str) -> AnyResult<&str> {
    lazy_static! {
        static ref BTTV_REGEX: Regex =
            Regex::new("(?:^| )(?:https?://)?(?:7tv\\.app/)?(?:emotes/)?([a-f0-9]{24})(?:$| )")
                .expect("must compile");
    }
    BTTV_REGEX
        .captures(str)
        .map(|c| c.iter().nth(1).flatten().map(|m| m.as_str()))
        .flatten()
        .ok_or_else(|| AnyError::msg("Could not find an emote code there!"))
}

fn get_duration(duration: &str) -> AnyResult<u64> {
    let duration = duration.trim();

    if let Some(captures) = Regex::new("^rand\\(([^;]+);([^)]+)\\)$")
        .expect("must compile")
        .captures(duration)
    {
        let mut iter = captures
            .iter()
            .skip(1)
            .take(2)
            .flatten()
            .map(|m| humantime::parse_duration(m.as_str().trim()).map(|d| d.as_secs()));
        let (first, second) = (iter.next(), iter.next());

        let (first, second) = match (first, second) {
            (Some(Ok(first)), Some(Ok(second))) => (first, second),
            tuple => {
                return Err(AnyError::msg(format!(
                    "Could not parse duration: {:?}",
                    tuple
                )))
            }
        };

        let (start, diff) = if first < second {
            (first, second - first)
        } else {
            (second, first - second)
        };

        Ok((start as f64 + rand::random::<f64>() * (diff as f64)).floor() as u64)
    } else {
        Ok(humantime::parse_duration(duration)?.as_secs())
    }
}
