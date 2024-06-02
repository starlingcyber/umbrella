use std::fmt::{self, Display, Formatter};

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
    info!(success, "last update status");

    // TODO: actually emit Prometheus metrics
}

/// Emit Prometheus metrics for a single piece of validator info.
fn validator_info(latest: &Latest) {
    let validator = latest.identity();
    let (Some(voting_power), Some(uptime), Some(state)) =
        (latest.voting_power(), latest.uptime(), latest.state())
    else {
        // If any of the info is missing, don't emit any metrics for this validator:
        warn!(%validator, "missing information for validator, skipping metrics");
        return;
    };

    let downtime_fraction =
        uptime.num_missed_blocks() as f64 / uptime.missed_blocks_window() as f64;
    let uptime_fraction = 1.0 - downtime_fraction;
    info!(
        %validator,
        %state,
        uptime = %Percent(uptime_fraction),
        %voting_power,
        "validator status",
    );

    // TODO: actually emit Prometheus metrics
}

struct Percent(f64);

impl Display for Percent {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:.2}%", self.0)
    }
}
