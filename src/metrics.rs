use crate::Latest;

/// Emit Prometheus metrics for each piece of validator info.
pub fn metrics(success: bool, info: &[Latest]) {
    success_info(success);
    for latest in info.iter() {
        validator_info(latest);
    }
}

/// Emit a Prometheus metric indicating whether the last round of updates was successful.
fn success_info(success: bool) {
    todo!("emit Prometheus metric for success");
}

/// Emit Prometheus metrics for a single piece of validator info.
fn validator_info(latest: &Latest) {
    let validator = latest.identity();
    let (Some(voting_power), Some(uptime), Some(state), Some(bonding_state)) = (
        latest.voting_power(),
        latest.uptime(),
        latest.state(),
        latest.bonding_state(),
    ) else {
        // If any of the info is missing, don't emit any metrics for this validator:
        warn!(%validator, "missing information for validator, skipping metrics");
        return;
    };

    // Emit the Prometheus metrics for the validator:
    todo!("emit Prometheus metrics for a single piece of validator info");
}
