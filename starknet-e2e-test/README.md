# Integration tests

This crate contains integration tests that are run against a fully-fledged
Madara node.

In order to run all tests:

```shell
cargo test --package starknet-e2e-test -- --nocapture
```

In order to run a specific test case:

```shell
cargo test --package starknet-e2e-test <your-test-case-name> -- --nocapture
```

## Anvil sandbox for Ethereum E2E tests

If `ANVIL_ENDPOINT` environment variable is set, integration tests involving
Ethereum contracts will try to attach to an already running Anvil. Otherwise, a
new instance will be spawn for every test.

The default binary search location is `~/.foundry/bin/anvil`. Alternatively you
can specify the Anvil binary location by setting `ANVIL_PATH` environment
variable.

IMPORTANT: make sure your local Anvil version uses a compatible `ethers-rs`
library version. In case of an issue, try to update both dependencies.
