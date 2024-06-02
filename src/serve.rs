use std::{convert::Infallible, net::SocketAddr, time::Duration};
use tokio::time::sleep;

pub async fn serve(bind: SocketAddr) -> eyre::Result<Infallible> {
    info!(%bind, "serving metrics");
    // TODO: Actually serve the metrics
    loop {
        sleep(Duration::MAX).await;
    }
}
