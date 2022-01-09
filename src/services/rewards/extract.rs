use anyhow::{Error as AnyError, Result as AnyResult};
use lazy_static::lazy_static;
use regex::Regex;

pub fn username(str: &str) -> AnyResult<String> {
    lazy_static! {
        static ref USERNAME_REGEX: Regex = Regex::new("@([\\w_]+)").expect("must compile");
    }

    let str = str.trim();

    if !str.contains(' ') {
        return Ok(str.replace('@', ""));
    }

    USERNAME_REGEX
        .captures(str)
        .and_then(|m| m.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| AnyError::msg("No user submitted"))
}

pub fn bttv_id(str: &str) -> AnyResult<&str> {
    lazy_static! {
        static ref BTTV_REGEX: Regex = Regex::new(
            "(?:^| )(?:https?://)?(?:betterttv\\.com/)?(?:emotes/)?([a-f0-9]{24})(?:$| )"
        )
        .expect("must compile");
    }
    BTTV_REGEX
        .captures(str)
        .and_then(|c| c.iter().nth(1).flatten().map(|m| m.as_str()))
        .ok_or_else(|| AnyError::msg("Could not find an emote code there!"))
}

pub fn ffz_id(str: &str) -> AnyResult<&str> {
    lazy_static! {
        static ref FFZ_REGEX: Regex = Regex::new(
            "(?:^| )(?:https?://)?(?:www\\.)?(?:frankerfacez\\.com/)?(?:emoticon/)(\\d+)(?:-[\\w_!]+)?(?:$| )"
        )
        .expect("must compile");
    }
    FFZ_REGEX
        .captures(str)
        .and_then(|c| c.iter().nth(1).flatten().map(|m| m.as_str()))
        .ok_or_else(|| AnyError::msg("Could not find an emote there!"))
}

pub fn seventv_id(str: &str) -> AnyResult<&str> {
    lazy_static! {
        static ref BTTV_REGEX: Regex =
            Regex::new("(?:^| )(?:https?://)?(?:7tv\\.app/)?(?:emotes/)?([a-f0-9]{24})(?:$| )")
                .expect("must compile");
    }
    BTTV_REGEX
        .captures(str)
        .and_then(|c| c.iter().nth(1).flatten().map(|m| m.as_str()))
        .ok_or_else(|| AnyError::msg("Could not find an emote code there!"))
}

pub fn duration(duration: &str) -> AnyResult<u64> {
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
