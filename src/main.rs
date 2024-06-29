#[macro_use]
extern crate tracing;
#[macro_use]
extern crate eyre;
#[macro_use]
extern crate tokio;
#[macro_use]
extern crate metrics;

use clap::Parser;
use std::{convert::Infallible, env};

mod app;
mod client;
mod latest;
mod options;
mod report;
mod serve;
mod update;

use app::App;
pub use {client::Client, latest::Latest, options::Options};
pub use {report::report, serve::serve, update::update};

#[tokio::main]
async fn main() -> eyre::Result<Infallible> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "warn");
    }
    tracing_subscriber::fmt::init();
    metrics_prometheus::install();

    // Read the command line options and convert them into an initial application state:
    let options = Options::parse();
    let bind = options.bind;
    let app = options.into_app();
    serve(bind, app).await
}
