# Umbrella ☂️

Operating a validator can be stressful: when it rains, it pours! ⛈️

Protect your [Penumbra](https://penumbra.zone) validator from the weather by monitoring its uptime with `umbrella`. ☂️

## What is Umbrella? 

Umbrella is a [Prometheus](https://prometheus.io/) exporter which monitors on-chain uptime for one or several Penumbra validators. It polls one or more specified Penumbra nodes, picking the data from the node with the highest height, and exports metrics which can be scraped by Prometheus, so that you can set alerts for validator signing downtime.

## What isn't Umbrella?

Umbrella is merely meant to fill a missing gap in monitoring infrastructure for Penumbra: correctly monitoring on-chain uptime for a node according to the semantics of Penumbra.

It is not a full-stack monitoring solution for validators. It does not export metrics about the health of individual nodes. It should be used in conjunction with other monitoring solutions to provide a comprehensive picture of your validator operation's wellbeing.
