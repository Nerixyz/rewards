use crate::services::text::first_capture;
use anyhow::{Error as AnyError, Result as AnyResult};
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use regex::Regex;

pub struct EmoteSpec<'a> {
    pub id: &'a str,
    pub override_name: Option<&'a str>,
}

pub fn username(str: &str) -> AnyResult<String> {
    lazy_static! {
        static ref USERNAME_REGEX: Regex =
            Regex::new("@([\\w_]+)").expect("must compile");
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

pub fn bttv(s: &str) -> AnyResult<EmoteSpec<'_>> {
    bttv_id(s).map(|id| EmoteSpec {
        id,
        override_name: None,
    })
}
pub fn ffz(s: &str) -> AnyResult<EmoteSpec<'_>> {
    ffz_id(s).map(|id| EmoteSpec {
        id,
        override_name: None,
    })
}
pub fn seventv(s: &str) -> AnyResult<EmoteSpec<'_>> {
    seventv_id(s).map(|id| EmoteSpec {
        id,
        override_name: parse_overridden(s),
    })
}

pub fn bttv_id(str: &str) -> AnyResult<&str> {
    lazy_static! {
        static ref BTTV_REGEX: Regex = Regex::new(
            "(?:^| )(?:https?://)?(?:betterttv\\.com/)?(?:emotes/)?([a-f0-9]{24})(?:$| )"
        )
        .expect("must compile");
    }
    first_capture(str, &BTTV_REGEX)
        .ok_or_else(|| AnyError::msg("Could not find an emote code there!"))
}

pub fn ffz_id(str: &str) -> AnyResult<&str> {
    lazy_static! {
        static ref FFZ_REGEX: Regex = Regex::new(
            "(?:^| )(?:https?://)?(?:www\\.)?(?:frankerfacez\\.com/)?(?:emoticon/)(\\d+)(?:-[\\w_!]+)?(?:$| )"
        )
        .expect("must compile");
    }
    first_capture(str, &FFZ_REGEX)
        .ok_or_else(|| AnyError::msg("Could not find an emote there!"))
}

pub fn seventv_id(str: &str) -> AnyResult<&str> {
    lazy_static! {
        static ref SEVENTV_REGEX: Regex = Regex::new(
            "(?:^| )(?:https?://)?(?:7tv\\.app/)?(?:emotes/)?([0-7][0-9A-HJKMNP-TV-Z]{25})(?:$| )"
        )
        .expect("must compile");
    }
    first_capture(str, &SEVENTV_REGEX)
        .ok_or_else(|| AnyError::msg("Could not find an emote code there!"))
}

/// in seconds
pub fn duration(duration: &str) -> AnyResult<u64> {
    let duration = duration.trim();

    if let Some(captures) = Regex::new("^rand\\(([^;]+);([^)]+)\\)$")
        .expect("must compile")
        .captures(duration)
    {
        let mut iter = captures.iter().skip(1).take(2).flatten().map(|m| {
            humantime::parse_duration(m.as_str().trim()).map(|d| d.as_secs())
        });
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

        Ok(
            (start as f64 + rand::random::<f64>() * (diff as f64)).floor()
                as u64,
        )
    } else {
        Ok(humantime::parse_duration(duration)?.as_secs())
    }
}

fn parse_overridden(s: &str) -> Option<&str> {
    static OVERRIDE_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new("(?: |^)as=([-_A-Za-z(!?&)$+:0-9]{2,100})\\b")
            .expect("must compile")
    });
    first_capture(s, &OVERRIDE_REGEX)
}
