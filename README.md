# Umbrella ☂️

When it rains, it pours! ⛈️ Protect your [Penumbra](https://penumbra.zone) validator from the weather by monitoring its uptime with `umbrella`. ☂️

Umbrella is a lightweight monitoring tool for Penumbra validators, inspired by [Tenderduty](https://github.com/blockpane/tenderduty). If you run a validator on Penumbra, it can monitor your signing uptime and the reachability of your individual sentry nodes, and send you alerts if something begins to go wrong. 

Instead of baking in a specific alerting mechanism, Umbrella instead allows you to plug in your favorite alerting platform by specifying a script to connect to its API to send alerts and optionally heartbeats.
