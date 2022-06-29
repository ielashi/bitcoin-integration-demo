# Bitcoin Integration Demo

A demo of the bitcoin endpoints on the Internet Computer.

This demo is already deployed to the IC, so you can already try it yourself. Examples:

## `get_balance`

```
dfx canister --network=ic call kmpi4-4aaaa-aaaal-aaqba-cai get_balance '("tb1qsgx55dp6gn53tsmyjjv4c2ye403hgxynxs0dnm")'
```

## `get_utxos`

```
dfx canister --network=ic call kmpi4-4aaaa-aaaal-aaqba-cai get_utxos '("tb1qc7psdze9j0r38rv8gj2kl8gysqevtqyqs20upw", null)'
```

In cases where an address has a large number of UTXOs, these may require pagination. If a `next_page` is provided in the response, then it can be retrieved as follows:

```
dfx canister --network=ic call kmpi4-4aaaa-aaaal-aaqba-cai get_utxos '("tb1qsgx55dp6gn53tsmyjjv4c2ye403hgxynxs0dnm", opt blob "<response.next_page>"
```

## `get_current_fee_percentiles`

```
dfx canister --network=ic call kmpi4-4aaaa-aaaal-aaqba-cai get_current_fee_percentiles
```

## `send_transaction`

```
dfx canister --network=ic call kmpi4-4aaaa-aaaal-aaqba-cai send_transaction '(blob "<raw transaction>")'
```
