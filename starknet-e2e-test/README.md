# Integration tests

This crate contains integration tests run against a fully-fledged Madara node.

In order to run all tests:

```shell
cargo test --package starknet-e2e-test -- --nocapture
```

In order to run a specific test case:

```shell
cargo test --package starknet-e2e-test <your-test-case-name> -- --nocapture
```

## Madara runner

Sometimes you might want more control over the launched node. Here is how you
can instantiate Madara runner which will run a node and provide you with a
client:

```rust
use madara_test_runner::{MadaraRunner, Settlement};

let madara_client = MadaraRunner::new(
    Some(Settlement::Ethereum),
    Some(format!("/tmp/madara").into()),
)
.await;
```

Available arguments:

- `settlement` [Optional] - which settlement layer to use (can be
  `Settlement::Ethereum` for now)
- `base_path` [Optional] - override Madara base path which is used for storing
  configs and other assets

Note that DA & settlement configs are expected to be stored in "data path" which
is `base_path/chains/<substrate_chain_id>` (for tests it's `dev`).

## Logging

When you run integration tests for the first time, cargo needs to build the
Madara binary first - it takes some time (depending on your h/w) and you might
see "<your-test-case-name> has been running for over 60 seconds" in your console
which is fine and it will eventually terminate.

However, if there are some building or launch errors, the process will stuck
(under the hood it will try to reconnect to the node, but the node fails to
start). In order to troubleshoot such issues you can enable Madara logs. Simply
run tests with `MADARA_LOG` environment variable set:

```shell
MADARA_LOG=1 cargo test --package starknet-rpc-test <your-test-case-name> -- --nocapture
```

The logs will be available at:

- `<project-dir>/target/madara-log/madara-stderr-log.txt`
- `<project-dir>/target/madara-log/madara-stdout-log.txt`

It can also be helpful if you want to troubleshoot some issue with debug/trace
logs.

## Parallel instances

Note that cargo might run tests in parallel meaning that there can be multiple
running Madara instance at a single point of time. In order to avoid concurrent
access to e.g. config files you can override Madara base path and use unique
temporary directories for each instance.

Here is how you can do that using `test_context` and `tempdir` crate:

```rust
use tempdir::TempDir;
use test_context::{test_context, AsyncTestContext};
use async_trait::async_trait;
use madara_node_runner::MadaraRunner;

struct Context {
    pub madara_path: TempDir,
}

#[async_trait]
impl AsyncTestContext for Context {
    async fn setup() -> Self {
        let madara_path = TempDir::new("madara").expect("Failed to create Madara path");
        Self { madara_path }
    }

    async fn teardown(self) {
        self.madara_path.close().expect("Failed to clean up");
    }
}

#[test_context(Context)]
#[rstest]
#[tokio::test]
async fn my_test_case(ctx: &mut Context) -> Result<(), anyhow::Error> {
    let madara = MadaraRunner::new(
        None,
        Some(ctx.madara_path.path().into()),
    )
    .await;

    todo!()
}
```

## Anvil

By default, integration tests involving Ethereum contracts will try to find
Anvil at `~/.foundry/bin/anvil`.  
Alternatively you can specify the Anvil binary location by setting `ANVIL_PATH`
environment variable.

IMPORTANT: make sure your Anvil version uses a compatiuble `ethers-rs` library version.  
In case of an issue, try to update both dependencies first.