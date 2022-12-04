use serde::{Deserialize, Serialize};

#[derive(
    sqlx::Type, Debug, derive_more::Display, Serialize, Deserialize, Clone, Copy,
)]
#[sqlx(type_name = "slot_platform", rename_all = "snake_case")]
pub enum SlotPlatform {
    #[display(fmt = "BTTV")]
    Bttv,
    #[display(fmt = "FFZ")]
    Ffz,
    #[sqlx(rename = "7tv")]
    #[display(fmt = "7TV")]
    SevenTv,
}

impl SlotPlatform {
    pub fn swap_reward_name(&self) -> &'static str {
        match self {
            SlotPlatform::Bttv => "BttvSwap",
            SlotPlatform::Ffz => "FfzSwap",
            SlotPlatform::SevenTv => "SevenTvSwap",
        }
    }
}
