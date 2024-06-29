use parking_lot::Mutex;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::time::Instant;

use crate::{report, update, Client, Latest};

/// The main application state.
#[derive(Debug, Clone)]
pub struct App {
    /// The sets of nodes to use to update the info for each validator.
    ///
    /// Each set of nodes is tried in order, with all the nodes in each set tried concurrently. Once
    /// all validators have been updated, no more nodes are tried.
    node_sets: Vec<Vec<Client>>,
    /// The latest info for each validator.
    info: Vec<Latest>,
    /// The time of the last update.
    last_update: Arc<Mutex<Option<Instant>>>,
    /// The last update success.
    last_success: Arc<AtomicBool>,
    /// The minimum polling interval.
    poll_interval: Duration,
    /// The timeout for connecting to each fullnode.
    connect_timeout: Duration,
}

impl App {
    /// Make a new application with the given sets of nodes and latest info.
    pub fn new(
        node_sets: Vec<Vec<Client>>,
        info: Vec<Latest>,
        poll_interval: Duration,
        connect_timeout: Duration,
    ) -> Self {
        Self {
            node_sets,
            info,
            last_update: Arc::new(Mutex::new(None)),
            last_success: Arc::new(AtomicBool::new(true)),
            poll_interval,
            connect_timeout,
        }
    }

    /// Run the application forever, updating the info for each validator at the polling interval.
    pub async fn update(&self) {
        // The locking here prevents multiple updates from happening concurrently within the same
        // polling interval, by atomically bumping the last update time to the current time.
        let (needs_update, now) = {
            // Check if an update is required
            let mut last_update = self.last_update.lock();
            let needs_update = match *last_update {
                None => true,
                Some(last_update) => last_update.elapsed() >= self.poll_interval,
            };

            if needs_update {
                *last_update = Some(Instant::now());
            }

            (needs_update, *last_update)
        };

        if needs_update {
            self.last_success.store(
                update(&self.node_sets, &self.info, self.connect_timeout).await,
                Ordering::SeqCst,
            );
        }

        // Emit metrics unconditionally, even if the update was not performed (this ensures that the
        // staleness metric is updated)
        report(self.last_success.load(Ordering::SeqCst), now, &self.info);
    }
}
