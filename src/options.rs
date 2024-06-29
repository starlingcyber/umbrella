use clap::Parser;
use penumbra_stake::IdentityKey;
use std::net::SocketAddr;
use tonic::transport::Uri;

use crate::{App, Client, Latest};

/// Umbrella: a Prometheus exporter to monitor on-chain uptime for one or several Penumbra
/// validators.
///
/// Umbrella connects on-demand to one or more Penumbra RPC endpoints, picking uptime data from the
/// node with the highest height. These metrics are served on a local HTTP server at the /metrics
/// endpoint, which can be scraped by Prometheus. In other words, Umbrella is a caching proxy
/// translating Prometheus scraping requests into RPC requests to Penumbra fullnodes, and
/// translating their responses into Prometheus metrics.
///
/// Please be nice to public RPC endpoints: if you're connecting to a public RPC, set it as a
/// fallback node so that you only use its resources if your own fullnodes are all unreachable.
#[derive(Parser, Clone, Debug)]
pub struct Options {
    /// Validator identity key to monitor for uptime (can be specified multiple times).
    #[clap(short = 'v', long, required = true)]
    pub validator: Vec<IdentityKey>,
    /// Fullnode RPC endpoint to monitor for health and use as a primary source for validator uptime
    /// information (can be specified multiple times).
    #[clap(short = 'n', long, required_unless_present("fallback"))]
    pub node: Vec<Uri>,
    /// Fullnode RPC endpoint to use as a backup source for validator uptime information (can be
    /// specified multiple times).
    ///
    /// If all of the primary nodes are unavailable, the client will attempt to connect to the
    /// fallback nodes one at a time in the order they are specified.
    #[clap(short = 'f', long, required_unless_present("node"))]
    pub fallback: Vec<Uri>,
    /// Port on which to serve Prometheus metrics.
    #[clap(short = 'b', long, default_value = "127.0.0.1:1984")]
    pub bind: SocketAddr,
    /// Minimum polling interval for updating the metrics.
    ///
    /// Metrics are updated from fullnodes only on-demand, but the result is cached for this
    /// duration to avoid excessive load on the fullnodes. This option does not usually need to be
    /// altered.
    #[clap(short = 'p', long, default_value = "1s")]
    pub poll_interval: humantime::Duration,
    /// Timeout for connecting to each fullnode.
    ///
    /// If a connection attempt to a fullnode takes longer than this duration, the attempt is
    /// aborted and considered failed. This option does not usually need to be altered.
    #[clap(short = 't', long, default_value = "5s")]
    pub connect_timeout: humantime::Duration,
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

        App::new(
            node_sets,
            info,
            self.poll_interval.into(),
            self.connect_timeout.into(),
        )
    }
}
