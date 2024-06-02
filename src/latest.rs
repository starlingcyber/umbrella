use parking_lot::RwLock;
use penumbra_stake::{validator, IdentityKey, Uptime};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Latest {
    identity: IdentityKey,
    info: Arc<RwLock<Option<Info>>>,
}

#[derive(Debug, Clone)]
pub struct Info {
    status: validator::Status,
    uptime: Uptime,
    updated: bool,
}

impl Latest {
    pub fn new(identity: IdentityKey) -> Self {
        Self {
            identity,
            info: Arc::new(RwLock::new(None)),
        }
    }

    pub fn identity(&self) -> IdentityKey {
        self.identity
    }

    fn status(&self) -> Option<validator::Status> {
        self.info.read().as_ref().map(|info| info.status.clone())
    }

    pub fn state(&self) -> Option<validator::State> {
        self.status().map(|status| status.state)
    }

    pub fn bonding_state(&self) -> Option<validator::BondingState> {
        self.status().map(|status| status.bonding_state.clone())
    }

    pub fn voting_power(&self) -> Option<u64> {
        self.status().map(|status| {
            status
                .voting_power
                .value()
                .try_into()
                .expect("voting power is too large to fit in u64")
        })
    }

    pub fn uptime(&self) -> Option<Uptime> {
        self.info.read().as_ref().map(|info| info.uptime.clone())
    }

    /// Reset the updated flag to false.
    pub fn reset(&self) {
        let mut info = self.info.write();
        if let Some(info) = info.as_mut() {
            info.updated = false;
        }
    }

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

    pub fn is_updated(&self) -> bool {
        self.info.read().as_ref().map_or(true, |info| info.updated)
    }
}
