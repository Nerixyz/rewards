use dotenv_codegen::dotenv;

pub const TWITCH_CLIENT_ID: &'static str = dotenv!("TWITCH_CLIENT_ID");
pub const TWITCH_CLIENT_SECRET: &'static str = dotenv!("TWITCH_CLIENT_SECRET");
pub const TWITCH_CLIENT_USER_LOGIN: &'static str = dotenv!("TWITCH_CLIENT_USER_LOGIN");
pub const SERVER_URL: &'static str = dotenv!( "SERVER_URL");
pub const DATABASE_URL: &'static str = dotenv!("DATABASE_URL");
pub const JWT_BASE64_SECRET: &'static str = dotenv!("JWT_BASE64_SECRET");
pub const EVENTSUB_BASE64_SECRET: &'static str = dotenv!("EVENTSUB_BASE64_SECRET");

