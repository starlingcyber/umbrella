use metrics::Unit;
use penumbra_stake::validator;
use tokio::time::Instant;

use crate::Latest;

/// Emit Prometheus metrics for each piece of validator info.
pub fn report(success: bool, last_update: Option<Instant>, info: &[Latest]) {
    gauge!("update_success").set(u8::from(success));
    describe_gauge!(
        "update_success",
        "Whether the last update was successful (1) or not (0)",
    );

    let elapsed = last_update
        .unwrap_or_else(Instant::now)
        .elapsed()
        .as_secs_f64();
    gauge!("update_staleness").set(elapsed);
    describe_gauge!(
        "update_staleness",
        Unit::Seconds,
        "Time elapsed in seconds since the last attempted update, whether or not it was successful",
    );

    for latest in info.iter() {
        validator_info(latest);
    }
}

/// Emit Prometheus metrics for a single piece of validator info.
fn validator_info(latest: &Latest) {
    let validator = latest.identity();
    let (Some(uptime), Some(state)) = (latest.uptime(), latest.state()) else {
        // If any of the info is missing, don't emit any metrics for this validator:
        warn!(%validator, "missing information");
        return;
    };

    let state_number = {
        use validator::State::*;
        match state {
            Defined => 0,
            Disabled => 1,
            Inactive => 2,
            Active => 3,
            Jailed => 4,
            Tombstoned => 5,
        }
    };
    gauge!("state", "validator" => validator.to_string()).set(state_number);
    describe_gauge!(
        "state",
        "Validator state (0=Defined, 1=Disabled, 2=Inactive, 3=Active, 4=Jailed, 5=Tombstoned)",
    );

    let uptime_percent = {
        let downtime_fraction =
            uptime.num_missed_blocks() as f64 / uptime.missed_blocks_window() as f64;
        let uptime_fraction = 1.0 - downtime_fraction;
        uptime_fraction * 100.0
    };
    gauge!("uptime", "validator" => validator.to_string()).set(uptime_percent);
    describe_gauge!(
        "uptime",
        Unit::Percent,
        "Validator uptime as a percentage, computed over the block window considered for on-chain uptime calculation",
    );

    let consecutive_missed_blocks = {
        let mut block = uptime.as_of_height() + 1;
        uptime
            .missed_blocks()
            .rev()
            .take_while(|&b| {
                let consecutive = b + 1 == block;
                block = b;
                consecutive
            })
            .count()
    };
    gauge!("consecutive_missed_blocks", "validator" => validator.to_string())
        .set(consecutive_missed_blocks as f64);
    describe_gauge!(
        "consecutive_missed_blocks",
        Unit::Count,
        "Number of most-recent consecutive blocks missed by the validator (resets to 0 on a signed block)",
    );

    info!(
        %validator,
        %state,
        uptime = %format!("{:.2}%", uptime_percent),
    );
}
