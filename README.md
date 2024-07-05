# Umbrella ☂️

Operating a validator can be stressful: when it rains, it pours! ⛈️

Protect your [Penumbra](https://penumbra.zone) validator from the weather by monitoring its uptime with `umbrella`. ☂️

![Screenshot from 2024-07-01 12-44-11](https://github.com/starlingcyber/umbrella/assets/597089/0ee1c16f-eef2-4254-a4e4-57a9d6cb6279)

## What is Umbrella? 

Umbrella is a [Prometheus](https://prometheus.io/) exporter which monitors on-chain uptime for one or several Penumbra validators. It connects on-demand to one or more specified Penumbra nodes, picking data from the node with the highest height, and exports Prometheus metrics via HTTP at `/metrics`, so that you can set alerts for validator signing downtime.

In other words, Umbrella is a caching proxy translating Prometheus scraping requests into RPC requests to Penumbra fullnodes, and translating their responses into Prometheus metrics reporting validator uptime.

## What isn't Umbrella?

Umbrella is a simple tool meant to do one thing well: correctly monitor on-chain uptime for a node according to the semantics of Penumbra.

It is not a full-stack monitoring solution for validators. It does not export metrics about the health of individual nodes. It should be used in conjunction with other monitoring solutions to provide a comprehensive picture of a validator operation's wellbeing.

## Quick start

### Install `umbrella`

#### Build locally

Clone this repository and build `umbrella` with the command:

```shell
cargo build --release
```

This builds the executable `target/release/umbrella`, which you can then run as you desire. Depending on your deployment, you may consider using a systemd service, Docker container, or something else. For Nix configuration, [see here](#nix-configuration).

### Run `umbrella`

Once you've installed `umbrella`, you can start the metrics server like this:

```shell
umbrella --validator $VALIDATOR_IDENTITY_KEY --node $PD_GRPC_BIND_ADDRESS --fallback $FALLBACK_PD_GRPC_BIND_ADDRESS
```

In the above, `$VALIDATOR_IDENTITY_KEY` is the identity key of the validator you wish to monitor, `$PD_GRPC_BIND_ADDRESS` is the URI of the RPC endpoint you want to get the information from, and `$FALLBACK_PD_GRPC_BIND_ADDRESS` is a fallback RPC which will only be used if no `--node` endpoint is reachable. All of these options can be repeated any number of times to specify multiple validators, multiple fullnodes, and multiple fallbacks, respectively.

**Please be nice to public RPC endpoints:** All nodes specified with `--node` are polled concurrently, and the information from the node with the highest block height is returned to Prometheus. Only if none of them respond, each `--fallback` is tried sequentially in the order specified on the command line. If you're connecting to a public RPC, it's courteous to set it as a `--fallback` node so that you only use its resources if your own fullnodes are all unreachable.

### Scrape metrics using Prometheus

To check if `umbrella` is working, visit `localhost:1984/metrics`, and you should see Prometheus metrics for your selected validator's uptime.

At present, the metrics reported are:

- `state{validator=...}`: gauge per validator measuring the validator's state by numeric label, with the meanings: `0=Defined`, `1=Disabled`, `2=Inactive`, `3=Active`, `4=Jailed`, `5=Tombstoned`
- `uptime{validator=...}` gauge per validator measuring the validator's uptime as a percentage in the numeric range [0, 100]
- `consecutive_missed_blocks{validator=...}`: gauge per validator measuring the length in blocks of the most recent string of consecutive downtime (reset to zero every time a block is signed)
- `update_success`: gauge reading `1` if the most recent update was successful, `0` if data could not be refreshed from any source
- `update_staleness`: gauge measuring the number of seconds since `umbrella` refreshed its cache of information (reset on every attempted update, regardless of success)

### Set up monitoring

Once you have `umbrella` running (perhaps as a systemd service or some such), you can configure Prometheus to scrape it, and Grafana to display its metrics and set alerts for when they are problematic. For a quick start Grafana dashboard, see the [example Grafana dashboard](dashboard.example.json).

A possible starting configuration for alerting on an active validator could be something like:

- **P0 critical** alert if `state > 3` (validator has been slashed and is jailed or tombstoned)
- **P1 high** alert if `state < 3` (validator is not active, but not due to downtime or misbehavior)
- **P1 high** alert if `update_success = 0` for longer than 10 minutes (`umbrella` is not managing to update itself, so you are flying blind)
- **P1 high** alert if `consecutive_missed_blocks > 120` (~10 minutes of consecutive downtime means something is wrong and it's not just ephemeral)
- **P1 high** alert if `uptime < 95` (cumulative downtime has exceeded ~40 minutes, something is interfering with availability in a significant way)
- **P2 moderate** alert if `consecutive_missed_blocks > 12` (~1 minute of consecutive downtime would be unusual for a well-configured functioning validator)
- **P2 moderate** alert if `uptime < 99` (normal operating condition should be > 99% uptime, so it might indicate an issue if there's a dip beneath this threshold)

## Nix configuration

Umbrella is tested and used on NixOS internally at Starling Cybernetics. If you use Nix, you can borrow from this Nix quickstart:

First, add this input to your flake:

```nix
inputs.umbrella.url = "github:starlingcyber/umbrella";
```

Then, you can import the Umbrella module into a NixOS configuration to enable and configure its systemd service:

```nix
imports = [ umbrella.outputs.nixosModules.default ];

services.umbrella = {
  enable = true;
  nodes = [ "https://..." ];
  fallbacks = [];
  validators = [ "penumbravalid1..." ];
};
```

On the same host, you can then configure a Prometheus scrape configuration for Umbrella:

```nix
services.prometheus = {
  enable = true;
  scrapeConfigs = [
    {
      job_name = "umbrella";
      scrape_interval = "5s";
      scrape_timeout = "5s";
      metrics_path = "/metrics";
      scheme = "http";
      static_configs = [
        {
          targets = [ "localhost:1984" ];
          labels.instance = "umbrella";
        }
      ];
      relabel_configs = [
        {
          source_labels = [ "__address__" ];
          target_label = "address";
        }
      ];
    }
  ];
};
```

Now Prometheus should be able to see your Umbrella metrics.
