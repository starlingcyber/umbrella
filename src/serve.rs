use std::{convert::Infallible, net::SocketAddr};

pub async fn serve(bind: SocketAddr) -> eyre::Result<Infallible> {
    todo!("serve the metrics on an HTTP server");
}
