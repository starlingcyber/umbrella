use clap::Parser;
use penumbra_stake::IdentityKey;
use std::net::SocketAddr;
use tonic::transport::Uri;

use crate::{App, Client, Latest};

#[derive(Parser, Clone, Debug)]
pub struct Options {
    /// Validator identity key to monitor for uptime (can be specified multiple times).
    #[clap(short, long, required = true)]
    pub validator: Vec<IdentityKey>,
    /// Fullnode RPC endpoint to monitor for health and use as a primary source for validator uptime
    /// information (can be specified multiple times).
    #[clap(short, long, required_unless_present("fallback"))]
    pub node: Vec<Uri>,
    /// Fullnode RPC endpoint to use as a backup source for validator uptime information (can be
    /// specified multiple times).
    ///
    /// If all of the primary nodes are unavailable, the client will attempt to connect to the
    /// fallback nodes one at a time in the order they are specified.
    #[clap(short, long, required_unless_present("node"))]
    pub fallback: Vec<Uri>,
    /// Port on which to serve Prometheus metrics.
    #[clap(short, long, default_value = "127.0.0.1:9814")]
    pub bind: SocketAddr,
}

impl Options {
    /// Convert the options into an application which can be run.
    pub fn into_app(self) -> App {
        // List of sets of clients to try to connect to -- first, try all the primary nodes
        // concurrently, then try each fallback node in order:
        let mut node_sets = Vec::with_capacity(1 + self.fallback.len());
        node_sets.push(self.node.into_iter().map(Client::new).collect::<Vec<_>>());
        node_sets.extend(
            self.fallback
                .into_iter()
                .map(Client::new)
                .map(|fallback| vec![fallback]),
        );

        // Make an updateable info cell for each validator:
        let info = self
            .validator
            .into_iter()
            .map(Latest::new)
            .collect::<Vec<_>>();

        App::new(node_sets, info)
    }
}
