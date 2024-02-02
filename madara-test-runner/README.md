# Madara test runner

Madara runner allows to launch a node with manual block sealing, and provides a
convenient client for interacting with that node.

## Arguments

Madara node can be parameterized with `MadaraArgs`:

```rust
use madara_test_runner::{MadaraRunner, MadaraArgs, Settlement};

let madara_client = MadaraRunner::new(
    MadaraArgs {
        settlement: Some(Settlement::Ethereum),
        settlement_conf: Some(format!("/tmp/madara/chains/dev/eth-config.json").into())
        base_path: Some(format!("/tmp/madara").into()),
    }
)
.await;
```

Available arguments:

- `settlement` [Optional] - which settlement layer to use (can be
  `Settlement::Ethereum` for now)
- `settlement_conf` [Optional] - path to the Ethereum settlement config
- `base_path` [Optional] - override Madara base path which is used for storing
  configs and other assets

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
MADARA_LOG=1 cargo test --package starknet-e2e-test <your-test-case-name> -- --nocapture
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

Here is how you can do that using `test_context` crate and `MadaraTempDir`
helper:

```rust
use tempfile::TempDir;
use test_context::{test_context, AsyncTestContext};
use async_trait::async_trait;
use madara_test_runner::{MadaraRunner, MadaraTempDir, MadaraArgs};

struct Context {
    pub madara_temp_dir: MadaraTempDir,
}

#[async_trait]
impl AsyncTestContext for Context {
    async fn setup() -> Self {
        let madara_temp_dir = MadaraTempDir::new();
        Self { madara_temp_dir }
    }

    async fn teardown(self) {
        self.madara_temp_dir.clear();
    }
}

#[test_context(Context)]
#[rstest]
#[tokio::test]
async fn my_test_case(ctx: &mut Context) -> Result<(), anyhow::Error> {
    let madara = MadaraRunner::new(
        MadaraArgs {
            base_path: Some(self.madara_temp_dir.base_path()),
            ..Default::default()
        }
    )
    .await;

    todo!()
}
```
