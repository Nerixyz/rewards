use prometheus::{
    register_histogram, register_int_counter, Histogram, IntCounter,
};

lazy_static::lazy_static! {
    pub static ref REDEMPTIONS: IntCounter = register_int_counter!("rewards_redemptions", "Number of redemptions").unwrap();
    pub static ref EXECUTION_DURATION: Histogram = register_histogram!("rewards_redemption_execution_duration", "Duration of an execution").unwrap();
}
