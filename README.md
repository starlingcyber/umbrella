# Umbrella ☂️

Operating a validator can be stressful: when it rains, it pours! ⛈️

Protect your [Penumbra](https://penumbra.zone) validator from the weather by monitoring its uptime with `umbrella`. ☂️

## What is Umbrella? 

Umbrella is a [Prometheus](https://prometheus.io/) exporter which monitors on-chain uptime for one or several Penumbra validators. It connects on-demand to one or more specified Penumbra nodes, picking data from the node with the highest height, and exports Prometheus metrics via HTTP at `/metrics`, so that you can set alerts for validator signing downtime.

In other words, Umbrella is a caching proxy translating Prometheus scraping requests into RPC requests to Penumbra fullnodes, and translating their responses into Prometheus metrics reporting validator uptime.

## What isn't Umbrella?

Umbrella is a simple tool meant to do one thing well: correctly monitor on-chain uptime for a node according to the semantics of Penumbra.

It is not a full-stack monitoring solution for validators. It does not export metrics about the health of individual nodes. It should be used in conjunction with other monitoring solutions to provide a comprehensive picture of a validator operation's wellbeing.

## Use only the resources you need

Please be nice to public RPC endpoints: if you're connecting to a public RPC, set it as a `--fallback` node so that you only use its resources if your own fullnodes are all unreachable.
