# Bitcoin Integration Demo

A demo of the `bitcoin_get_balance` and `bitcoin_get_utxos` endpoints.

This demo is already deployed to the IC, so you can already try it yourself. Examples:

```
dfx canister --network=ic call bitcoin_integration_demo get_balance '("tb1qsgx55dp6gn53tsmyjjv4c2ye403hgxynxs0dnm")'

dfx canister --network=ic call bitcoin_integration_demo get_utxos '("tb1qc7psdze9j0r38rv8gj2kl8gysqevtqyqs20upw")'
```

Note that, at the time of this writing, when calling `get_utxos` with thousands of UTXOs, one may
get a "Payload too large" error. This is an error that will be rectified in a future replica
upgrade.

