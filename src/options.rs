use clap::Parser;
use penumbra_stake::IdentityKey;

#[derive(Parser, Clone, Debug)]
pub(crate) struct Options {
    /// Validator identity key to monitor (can be specified multiple times).
    #[clap(long)]
    validator: Vec<IdentityKey>,
    /// Fullnode RPC endpoint to monitor (can be specified multiple times).
    #[clap(long)]
    node: Vec<tonic::transport::Uri>,
    /// Fallback fullnode RPC endpoint (can be specified multiple times, only used when no `--node`
    /// can be reached).
    #[clap(long)]
    fallback: Vec<tonic::transport::Uri>,
    /// Whenever something is wrong, run this alert script (can be specified multiple times).
    #[clap(long)]
    alert: Vec<String>,
    /// Polling interval for uptime and health checks.
    #[clap(long, default_value = "5s")]
    poll: humantime::Duration,
    /// Every `--heartrate`, emit a heartbeat by running this script (can be specified multiple
    /// times, requires `--heartrate`).
    #[clap(long, requires("heartrate"))]
    heartbeat: Vec<String>,
    /// Run every `--heartbeat` script at this interval (requires `--heartbeat`).
    #[clap(long, requires("heartbeat"))]
    heartrate: Option<humantime::Duration>,
}
