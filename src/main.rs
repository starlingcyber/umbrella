#[macro_use]
extern crate tracing;
#[macro_use]
extern crate eyre;
#[macro_use]
extern crate tokio;

use clap::Parser;
use std::{convert::Infallible, time::Duration};

mod app;
mod client;
mod latest;
mod metrics;
mod options;
mod serve;
mod update;

use app::App;
pub use {client::Client, latest::Latest, options::Options};
pub use {metrics::metrics, serve::serve, update::update};

/// Poll nodes at this interval to update the metrics.
pub const POLL_INTERVAL: Duration = Duration::from_secs(5);

#[tokio::main]
async fn main() -> eyre::Result<Infallible> {
    tracing_subscriber::fmt::init();

    // Read the command line options and convert them into an initial application state:
    let options = Options::parse();
    let bind = options.bind;
    let app = options.into_app();

    // Run the application to gather data and the metrics server to serve it:
    select! {
        // The application runs forever (all errors are handled and logged):
        forever = app.run() => match forever {},
        // The metrics server will run forever or return an error:
        result = serve(bind) => result,
    }
}
