#[macro_use]
extern crate tracing;

#[macro_use]
extern crate eyre;

use clap::Parser;
use eyre::Ok;
use parking_lot::RwLock;
use penumbra_proto::core::component::stake::v1::{ValidatorStatusRequest, ValidatorUptimeRequest};
use penumbra_stake::validator;
use std::{sync::Arc, time::Duration};
use tokio::{
    task::JoinSet,
    time::{interval, timeout},
};

mod options;
use options::Options;
mod client;
use client::Client;
mod latest;
use latest::Latest;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let options = Options::parse();

    // Make a client for each node:
    let clients = options
        .node
        .into_iter()
        .map(Client::new)
        .collect::<Vec<_>>();

    // Make an info for each validator:
    let info = options
        .validator
        .into_iter()
        .map(Latest::new)
        .collect::<Vec<_>>();

    // Every five seconds, update the info for each validator using the clients:
    let mut interval = interval(Duration::from_secs(5));
    loop {
        interval.tick().await;

        // Reset the updated flag on each piece of info:
        for latest in info.iter() {
            latest.reset();
        }

        // For each pair in the cartesian product of client and info, spawn a task that updates the info
        // using the client:
        let mut join_set = JoinSet::new();
        for client in clients.iter() {
            for latest in info.iter() {
                join_set.spawn(update_validator_info(client.clone(), latest.clone()));
            }
        }

        // Wait for all the tasks to finish:
        while join_set.join_next().await.is_some() {}

        // Check to make sure all the info was updated:
        if info.iter().any(|latest| !latest.is_updated()) {
            error!("connection errors or timeouts prevented some updates");
        }

        // Emit the Prometheus metrics for each piece of info:
        for latest in info.iter() {
            todo!("emit Prometheus metrics for each piece of info");
        }
    }
}

async fn update_validator_info(client: Client, latest: Latest) {
    let validator = latest.identity();
    let client_copy = client.clone();

    let update = async move {
        let uptime = client
            .get()
            .await?
            .validator_uptime(ValidatorUptimeRequest {
                identity_key: Some(validator.into()),
            })
            .await?
            .into_inner()
            .uptime
            .ok_or_else(|| eyre!("no uptime data"))?
            .try_into()
            .map_err(|_| eyre!("invalid uptime data"))?;
        let status: validator::Status = client
            .get()
            .await?
            .validator_status(ValidatorStatusRequest {
                identity_key: Some(validator.into()),
            })
            .await?
            .into_inner()
            .status
            .ok_or_else(|| eyre!("no status data"))?
            .try_into()
            .map_err(|_| eyre!("invalid status data"))?;
        latest.update(status, uptime);
        Ok(())
    };

    // Set a timeout of five seconds so we don't hang forever waiting for a response:
    let update = timeout(Duration::from_secs(5), update);

    // If there was an error in the connection, throw it away and make the next update try to form a
    // new connection, rather than reusing the old, potentially broken one:
    if let Err(error) = async move { Ok(update.await??) }.await {
        client_copy.disconnect().await;
        warn!(%validator, node = %client_copy.uri(), "{}", error);
    }
}
