use parking_lot::RwLock;
use penumbra_proto::core::component::stake::v1::query_service_client::QueryServiceClient as StakeQueryServiceClient;
use std::sync::Arc;
use tonic::transport::{Channel, Uri};

#[derive(Debug, Clone)]
pub struct Client {
    inner: Arc<RwLock<Node>>,
}

impl Client {
    pub fn new(uri: Uri) -> Self {
        Self {
            inner: Arc::new(RwLock::new(Node::Disconnected { uri })),
        }
    }

    pub fn uri(&self) -> Uri {
        self.inner.read().uri().clone()
    }

    pub async fn get(&self) -> eyre::Result<StakeQueryServiceClient<Channel>> {
        let node = self.inner.read().clone();
        match node {
            Node::Connected { client, .. } => Ok(client.clone()),
            Node::Disconnected { uri } => {
                let client = StakeQueryServiceClient::connect(uri.clone()).await?;
                // TODO: This is a race condition which will hit the server with a new connection
                // for every validator every time the client is diconnected. This is not ideal if
                // monitoring many validators, but since the normal use case of this tool is to
                // monitor a single validator, it is acceptable for now. In the future, we should
                // share the work of reconnecting by having a dedicated task manage the connections.
                *self.inner.write() = Node::Connected {
                    client: client.clone(),
                    uri: uri.clone(),
                };
                Ok(client)
            }
        }
    }

    pub async fn disconnect(&self) {
        let mut node = self.inner.write();
        *node = Node::Disconnected {
            uri: node.uri().clone(),
        };
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    Connected {
        client: StakeQueryServiceClient<Channel>,
        uri: Uri,
    },
    Disconnected {
        uri: Uri,
    },
}

impl Node {
    pub fn uri(&self) -> &Uri {
        match self {
            Self::Connected { uri, .. } | Self::Disconnected { uri } => uri,
        }
    }
}
