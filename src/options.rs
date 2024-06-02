use clap::Parser;
use penumbra_stake::IdentityKey;

#[derive(Parser, Clone, Debug)]
pub struct Options {
    /// Validator identity key to monitor for uptime (can be specified multiple times).
    #[clap(short, long)]
    pub validator: Vec<IdentityKey>,
    /// Fullnode RPC endpoint to monitor for health and use as a primary source for validator uptime
    /// information (can be specified multiple times).
    #[clap(short, long)]
    pub node: Vec<tonic::transport::Uri>,
}
