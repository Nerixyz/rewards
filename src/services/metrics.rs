pub fn register_metrics() {
    metrics::register_counter!("rewards_redemptions");
    metrics::register_histogram!("rewards_redemption_execution_duration");
}
