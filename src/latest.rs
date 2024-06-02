use parking_lot::RwLock;
use penumbra_stake::{validator, IdentityKey, Uptime};
use std::sync::Arc;

/// An updateable cell holding the latest information about a validator.
#[derive(Debug, Clone)]
pub struct Latest {
    /// The identity key of the validator.
    identity: IdentityKey,
    /// The latest information about the validator.
    info: Arc<RwLock<Option<Info>>>,
}

/// The latest information about a validator.
#[derive(Debug, Clone)]
pub struct Info {
    /// The status of the validator.
    status: validator::Status,
    /// The uptime of the validator.
    uptime: Uptime,
    /// Whether the info has been updated since the last time it was reported.
    ///
    /// This is used for determining whether the update process was successful, in the case of
    /// partial errors with connections to nodes.
    updated: bool,
}

impl Latest {
    /// Make a new updateable cell for the given validator identity.
    ///
    /// The cell is initially empty and must be updated before use.
    pub fn new(identity: IdentityKey) -> Self {
        Self {
            identity,
            info: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the identity key of the validator.
    pub fn identity(&self) -> IdentityKey {
        self.identity
    }

    /// Get the status of the validator.
    fn status(&self) -> Option<validator::Status> {
        self.info.read().as_ref().map(|info| info.status.clone())
    }

    /// Get the state of the validator.
    pub fn state(&self) -> Option<validator::State> {
        self.status().map(|status| status.state)
    }

    /// Get the bonding state of the validator.
    pub fn bonding_state(&self) -> Option<validator::BondingState> {
        self.status().map(|status| status.bonding_state.clone())
    }

    /// Get the voting power of the validator.
    pub fn voting_power(&self) -> Option<u64> {
        self.status().map(|status| {
            status
                .voting_power
                .value()
                .try_into()
                .expect("voting power is too large to fit in u64")
        })
    }

    /// Get the uptime of the validator.
    pub fn uptime(&self) -> Option<Uptime> {
        self.info.read().as_ref().map(|info| info.uptime.clone())
    }

    /// Reset the updated flag to false.
    ///
    /// This should be done at the start of each update cycle.
    pub fn reset(&self) {
        let mut info = self.info.write();
        if let Some(info) = info.as_mut() {
            info.updated = false;
        }
    }

    /// Update the info for the validator with the given status and uptime.
    ///
    /// If the uptime reports an older height, no update to either field is made.
    pub fn update(&self, status: validator::Status, uptime: Uptime) {
        // If the uptime is newer or equal, update all the fields; if the uptime is older, do
        // nothing, so that we only progress monotonically through time:
        let mut info = self.info.write();
        if info.as_ref().map_or(true, |info| {
            uptime.as_of_height() >= info.uptime.as_of_height()
        }) {
            *info = Some(Info {
                status,
                uptime,
                updated: true,
            });
        }
    }

    /// Check whether the info is fresh.
    pub fn is_fresh(&self) -> bool {
        self.info.read().as_ref().map_or(true, |info| info.updated)
    }

    /// Check whether the info is stale.
    pub fn is_stale(&self) -> bool {
        !self.is_fresh()
    }
}
