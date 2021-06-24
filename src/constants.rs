use dotenv_codegen::dotenv;

pub const TWITCH_CLIENT_ID: &str = dotenv!("TWITCH_CLIENT_ID");
pub const TWITCH_CLIENT_SECRET: &str = dotenv!("TWITCH_CLIENT_SECRET");
pub const TWITCH_CLIENT_USER_LOGIN: &str = dotenv!("TWITCH_CLIENT_USER_LOGIN");
pub const TWITCH_CLIENT_USER_ID: &str = dotenv!("TWITCH_CLIENT_USER_ID");
pub const BTTV_JWT: &str = dotenv!("BTTV_JWT");
pub const FFZ_SESSION: &str = dotenv!("FFZ_SESSION");
pub const SEVEN_TV_JWT: &str = dotenv!("SEVEN_TV_JWT");
pub const SERVER_URL: &str = dotenv!("SERVER_URL");
pub const DATABASE_URL: &str = dotenv!("DATABASE_URL");
pub const JWT_BASE64_SECRET: &str = dotenv!("JWT_BASE64_SECRET");
pub const EVENTSUB_BASE64_SECRET: &str = dotenv!("EVENTSUB_BASE64_SECRET");
pub const SPOTIFY_CLIENT_ID: &str = dotenv!("SPOTIFY_CLIENT_ID");
pub const SPOTIFY_CLIENT_SECRET: &str = dotenv!("SPOTIFY_CLIENT_SECRET");
