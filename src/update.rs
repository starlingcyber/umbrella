use eyre::Ok;
use penumbra_proto::core::component::stake::v1::{ValidatorStatusRequest, ValidatorUptimeRequest};
use tokio::{task::JoinSet, time::timeout};
use tonic::transport::Uri;

use crate::{Client, Latest, POLL_INTERVAL};

/// Use all the nodes in each set of nodes to update the info for each validator, treating each set
/// concurrently, and stopping early if all the info is updated.
pub async fn update(node_sets: &[Vec<Client>], info: &[Latest]) -> bool {
    // Try to update the info for each validator from all the nodes in each set of nodes, in order:
    let mut stale = info.to_vec();
    let mut failed: Option<Vec<Uri>> = None;

    for nodes in node_sets.iter() {
        // Log a warning if we're trying to connect to the fallback nodes:
        let fallback = nodes.iter().map(Client::uri).cloned().collect::<Vec<_>>();
        if let Some(failed) = failed {
            warn!(
                ?failed,
                ?fallback,
                "error in previous connection attempt, trying fallback node(s)"
            );
        }
        failed = Some(fallback);

        // Try to update the info for each validator from all the nodes in the set:
        stale = update_all_validator_info(&nodes[..], &stale[..]).await;
        if stale.is_empty() {
            // If all the info was updated, break because we don't need any more information:
            break;
        }
    }

    // If after updating from all nodes in all sets, some info is still stale, log an error:
    if !stale.is_empty() {
        let validators = stale
            .iter()
            .map(|latest| latest.identity())
            .collect::<Vec<_>>();
        error!(?validators, "failed to update from any data source");
    }

    // Return true if all the info was updated, false otherwise:
    stale.is_empty()
}

/// Concurrently update the info for each validator from each node, returning the list of all
/// validators which failed to update.
async fn update_all_validator_info(nodes: &[Client], info: &[Latest]) -> Vec<Latest> {
    // Reset the updated flag on each piece of info:
    for latest in info.iter() {
        latest.reset();
    }

    // Reconnect all the clients:
    for node in nodes.iter() {
        node.connect().await.unwrap_or_else(|error| {
            warn!(node = %node.uri(), "{}", error);
        });
    }

    // For each pair in the cartesian product of client and info, spawn a task that updates the info
    // using the client:
    let mut tasks = JoinSet::new();
    for client in nodes.iter() {
        for latest in info.iter() {
            tasks.spawn(update_validator_info(client.clone(), latest.clone()));
        }
    }

    // Wait for all the tasks to finish:
    while tasks.join_next().await.is_some() {}

    // Check to make sure all the info was updated:
    info.iter()
        .filter(|latest| latest.is_stale())
        .cloned()
        .collect()
}

/// Update the info for a single validator from a single node.
async fn update_validator_info(client: Client, latest: Latest) {
    // If the client is not connected, don't try to update the info: it's disconnected due to a
    // previous error in this round of updates, and will be reconnected in the next round.
    let Some(stake_client) = client.get() else {
        return;
    };

    let validator = latest.identity();

    // Async task that updates the info for a single validator from a single node, concurrently
    // asking for its status and uptime:
    let update = async move {
        let mut uptime_client = stake_client.clone();
        let uptime = async {
            Ok(uptime_client
                .validator_uptime(ValidatorUptimeRequest {
                    identity_key: Some(validator.into()),
                })
                .await?
                .into_inner()
                .uptime
                .ok_or_else(|| eyre!("no uptime data"))?
                .try_into()
                .map_err(|_| eyre!("invalid uptime data"))?)
        };
        let mut status_client = stake_client.clone();
        let status = async {
            Ok(status_client
                .validator_status(ValidatorStatusRequest {
                    identity_key: Some(validator.into()),
                })
                .await?
                .into_inner()
                .status
                .ok_or_else(|| eyre!("no status data"))?
                .try_into()
                .map_err(|_| eyre!("invalid status data"))?)
        };
        let (uptime, status) = join!(uptime, status);
        let uptime = uptime?;
        let status = status?;
        latest.update(status, uptime);
        Ok(())
    };

    // Set a timeout of five seconds so we don't hang forever waiting for a response:
    let update = async move { timeout(POLL_INTERVAL, update).await? };

    // If there was an error in the connection, throw it away and make the next update try to form a
    // new connection, rather than reusing the old, potentially broken one:
    if let Err(error) = update.await {
        client.disconnect();
        warn!(node = %client.uri(), %validator, "{}", error);
    }
}
