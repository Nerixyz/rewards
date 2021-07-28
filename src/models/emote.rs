#[derive(sqlx::Type, Debug)]
#[sqlx(type_name = "slot_platform", rename_all = "snake_case")]
pub enum SlotPlatform {
    Bttv,
    Ffz,
    #[sqlx(rename = "7tv")]
    SevenTv,
}
