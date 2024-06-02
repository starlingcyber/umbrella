use eyre::Ok;
use parking_lot::RwLock;
use penumbra_proto::core::component::stake::v1::query_service_client::QueryServiceClient as StakeQueryServiceClient;
use std::sync::Arc;
use tonic::transport::{Channel, Uri};

/// A client for the stake query service, which can be disconnected and reconnected in case of failures.
#[derive(Debug, Clone)]
pub struct Client {
    uri: Uri,
    inner: Arc<RwLock<Option<StakeQueryServiceClient<Channel>>>>,
}

impl Client {
    /// Make a new client from a URI.
    ///
    /// The client is initially disconnected and must be connected before use.
    pub fn new(uri: Uri) -> Self {
        Self {
            inner: Arc::new(RwLock::new(None)),
            uri,
        }
    }

    /// Get the URI of the client.
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    /// Connect the client to the server.
    pub async fn connect(&self) -> eyre::Result<()> {
        // If the client is already connected, this is a no-op.
        if self.inner.read().is_none() {
            let client = StakeQueryServiceClient::connect(self.uri.clone()).await?;
            self.inner.write().replace(client);
        }
        Ok(())
    }

    /// Disconnect the client from the server.
    pub fn disconnect(&self) {
        self.inner.write().take();
    }

    /// Get the client, if it is connected.
    ///
    /// This does not attempt to connect the client if it is disconnected.
    pub fn get(&self) -> Option<StakeQueryServiceClient<Channel>> {
        self.inner.read().clone()
    }
}
