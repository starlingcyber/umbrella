use std::convert::Infallible;

use tokio::time::{interval, MissedTickBehavior};

use crate::{metrics, update, Client, Latest, POLL_INTERVAL};

/// The main application state.
pub struct App {
    /// The sets of nodes to use to update the info for each validator.
    ///
    /// Each set of nodes is tried in order, with all the nodes in each set tried concurrently. Once
    /// all validators have been updated, no more nodes are tried.
    node_sets: Vec<Vec<Client>>,
    /// The latest info for each validator.
    info: Vec<Latest>,
}

impl App {
    /// Make a new application with the given sets of nodes and latest info.
    pub fn new(node_sets: Vec<Vec<Client>>, info: Vec<Latest>) -> Self {
        Self { node_sets, info }
    }

    /// Run the application forever, updating the info for each validator at the polling interval.
    pub async fn run(&self) -> Infallible {
        // Every polling interval, update the info for each validator using the clients:
        let mut interval = interval(POLL_INTERVAL);
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            interval.tick().await;
            let success = update(&self.node_sets, &self.info).await;
            metrics(success, &self.info);
        }
    }
}
