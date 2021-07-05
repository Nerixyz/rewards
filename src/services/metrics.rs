pub fn register_metrics() {
    metrics::register_counter!("rewards_redemptions", "Redemptions of rewards");
    metrics::register_histogram!("rewards_redemption_execution_duration", "Redemption time");
}
