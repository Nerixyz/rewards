use crate::{
    log_discord,
    services::{bttv, ffz, seven_tv},
};
use anyhow::Result as AnyResult;
use futures_util::future;
use std::fmt::{Display, Formatter};

pub struct Platforms {
    seventv: bool,
    ffz: bool,
    bttv: bool,
}

impl Platforms {
    pub async fn get() -> AnyResult<Self> {
        let (bttv, ffz, seventv) = future::join3(get_bttv(), get_ffz(), get_seventv()).await;
        let ffz = ffz?;
        Ok(Self { bttv, ffz, seventv })
    }
}

impl Display for Platforms {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let platform = |ok: bool| if ok { "✅" } else { "❌" };

        write!(
            f,
            "7tv={},ffz={},bttv={}",
            platform(self.seventv),
            platform(self.ffz),
            platform(self.bttv)
        )
    }
}

async fn get_bttv() -> bool {
    match bttv::requests::get_dashboards().await {
        Ok(_) => true,
        Err(e) => {
            log_discord!("Bttv error", e.to_string(),);
            false
        }
    }
}

async fn get_ffz() -> AnyResult<bool> {
    ffz::requests::logged_in().await
}

async fn get_seventv() -> bool {
    seven_tv::requests::logged_in().await
}
