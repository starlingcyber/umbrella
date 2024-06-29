use axum::{extract::State, http::StatusCode, routing::get, Router};
use std::{convert::Infallible, net::SocketAddr};
use tokio::net::TcpListener;

use crate::App;

pub async fn serve(bind: SocketAddr, app: App) -> eyre::Result<Infallible> {
    info!(%bind, "serving metrics");
    // Use axum to serve metrics at the given address:
    let router = Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(app)
        .into_make_service();
    let listener = TcpListener::bind(bind).await?;
    axum::serve(listener, router).await?;
    unreachable!("axum::serve should never return without an error");
}

async fn metrics_handler(State(app): State<App>) -> axum::response::Result<String> {
    app.update().await;
    prometheus::TextEncoder::new()
        .encode_to_string(&prometheus::default_registry().gather())
        .map_err(|e| {
            error!(%e, "failed to encode metrics");
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .map_err(Into::into)
}
