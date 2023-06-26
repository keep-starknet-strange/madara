## Sub-Command Execution Status

| Test ID | Sub-Command               | Status |
| ------- | ------------------------- | ------ |
| 1       | create-snapshot           | OK     |
| 2       | on-runtime-upgrade (live) | OK     |
| 3       | on-runtime-upgrade (snap) | OK     |
| 4       | execute-block (live)      | OK     |
| 5       | execute-block (snap)      | OK     |
| 6       | offchain-worker (live)    | OK     |
| 7       | offchain-worker (snap)    | OK     |
| 8       | follow-chain              | OK     |
| 9       | fast-forward (live)       | KO     |
| 10      | fast-forward (snap)       | KO     |

Tests with ID 9 and 10 have encountered failures, with the error message "ERROR
main runtime: panicked at 'Timestamp slot must match `CurrentSlot`'". The
underlying cause of this issue is currently unknown. To facilitate the
resolution and ensure progress, a support ticket has been raised and submitted
to the substrate. For more information, please visit
[here](https://substrate.stackexchange.com/questions/9024/substrate-try-runtime-sub-command-fast-forward-error-main-runtime-pa).

## How to Build and Run under Local Testnet

To build and run under the local testnet, follow the steps below:

- Create the release binary and launch the local testnet using the script
  `infra/local-testnet/run-cfg2.sh`:

```bash
cargo build --release --bin madara
```

- Execute the `try-runtime` subcommands against the local testnet:

```bash
cargo run --release --features=try-runtime --bin madara -- try-runtime --help
```

This will provide you with a list of available `try-runtime` subcommands:

- `on-runtime-upgrade`: Execute the migrations of the given runtime.
- `execute-block`: Executes the given block against some state.
- `offchain-worker`: Executes the offchain worker hooks of a given block against
  some state.
- `follow-chain`: Follow the given chain's finalized blocks and apply all of its
  extrinsics.
- `fast-forward`: Produce a series of empty, consecutive blocks and execute them
  one-by-one.
- `create-snapshot`: Create a new snapshot file.
- `help`: Print this message or the help of the given subcommand(s).

Feel free to refer to these instructions when building and running under the
local testnet.

================================================================

## create-snapshot

> Step-01 | create snapshot from sharingan testnet

```bash
export RPC_WS_URL="ws://127.0.0.1:9931" && \
export CHAIN_SPEC="./crates/node/chain-specs/madara-local-cfg2-raw.json" && \
export SNAPSHOT_PATH="../000-01-install/madara-localnet.snap" && \
RUST_LOG=runtime=trace,try-runtime::cli=trace,executor=trace \
cargo \
    run \
    --release \
    --features=try-runtime \
    --bin madara \
    -- \
    try-runtime \
    --runtime existing \
    --chain  "$CHAIN_SPEC" \
    create-snapshot \
    --uri  "$RPC_WS_URL" \
    "$SNAPSHOT_PATH"
```

![](https://i.imgur.com/6uXiAkB.png)

================================================================

## on-runtime-upgrade

> **live**

```bash
export RPC_WS_URL="ws://127.0.0.1:9931" && \
export CHAIN_SPEC="./crates/node/chain-specs/madara-local-cfg2-raw.json" && \
RUST_LOG=runtime=trace,try-runtime::cli=trace,executor=trace \
cargo \
    run \
    --release \
    --features=try-runtime \
    --bin madara \
    -- \
    try-runtime \
    --runtime existing \
    --chain  "$CHAIN_SPEC" \
    on-runtime-upgrade \
    --checks=pre-and-post \
    live \
    --uri  "$RPC_WS_URL"
```

![](https://i.imgur.com/2ExO6Df.png)

> **snap**

```bash
export SNAPSHOT_PATH="../000-01-install/madara-localnet.snap" && \
export CHAIN_SPEC="./crates/node/chain-specs/madara-local-cfg2-raw.json" && \
RUST_LOG=runtime=trace,try-runtime::cli=trace,executor=trace \
cargo \
    run \
    --release \
    --features=try-runtime \
    --bin madara \
    -- \
    try-runtime \
    --runtime existing \
    --chain  "$CHAIN_SPEC" \
    on-runtime-upgrade \
    --checks=pre-and-post \
    snap \
    --snapshot-path "$SNAPSHOT_PATH"
```

![](https://i.imgur.com/xJtloN7.png)

================================================================

## execute-block

> **live**

```bash
export RPC_WS_URL="ws://127.0.0.1:9931" && \
export CHAIN_SPEC="./crates/node/chain-specs/madara-local-cfg2-raw.json" && \
RUST_LOG=runtime=trace,try-runtime::cli=trace,executor=trace \
cargo \
    run \
    --release \
    --features=try-runtime \
    --bin madara \
    -- \
    try-runtime \
    --runtime existing \
    --chain  "$CHAIN_SPEC" \
    execute-block \
    --try-state=all \
    live \
    --uri  "$RPC_WS_URL"
```

![](https://i.imgur.com/g69ntk8.png)

> **snap**

```bash
export RPC_WS_URL="ws://127.0.0.1:9931" && \
export SNAPSHOT_PATH="../000-01-install/madara-localnet.snap" && \
export CHAIN_SPEC="./crates/node/chain-specs/madara-local-cfg2-raw.json" && \
RUST_LOG=runtime=trace,try-runtime::cli=trace,executor=trace \
cargo \
    run \
    --release \
    --features=try-runtime \
    --bin madara \
    -- \
    try-runtime \
    --runtime existing \
    --chain  "$CHAIN_SPEC" \
    execute-block \
    --try-state=all \
 --block-ws-uri="$RPC_WS_URL" \
    snap \
    --snapshot-path "$SNAPSHOT_PATH"
```

![](https://i.imgur.com/fkWNWqW.png)

================================================================

## offchain-worker

> **live**

```bash
export RPC_WS_URL="ws://127.0.0.1:9931" && \
export CHAIN_SPEC="./crates/node/chain-specs/madara-local-cfg2-raw.json" && \
RUST_LOG=runtime=trace,try-runtime::cli=trace,executor=trace \
cargo \
    run \
    --release \
    --features=try-runtime \
    --bin madara \
    -- \
    try-runtime \
    --runtime existing \
    --chain  "$CHAIN_SPEC" \
    offchain-worker \
    live \
    --uri  "$RPC_WS_URL"
```

![](https://i.imgur.com/9ln94nu.png)

> **snap**

```bash
export RPC_WS_URL="ws://127.0.0.1:9931" && \
export SNAPSHOT_PATH="../000-01-install/madara-localnet.snap" && \
export CHAIN_SPEC="./crates/node/chain-specs/madara-local-cfg2-raw.json" && \
RUST_LOG=runtime=trace,try-runtime::cli=trace,executor=trace \
cargo \
    run \
    --release \
    --features=try-runtime \
    --bin madara \
    -- \
    try-runtime \
    --runtime existing \
    --chain  "$CHAIN_SPEC" \
    offchain-worker \
 --header-ws-uri="$RPC_WS_URL" \
    snap \
    --snapshot-path "$SNAPSHOT_PATH"
```

![](https://i.imgur.com/tetO5lQ.png)

================================================================

## follow-chain

```bash
export RPC_WS_URL="ws://127.0.0.1:9931" && \
export CHAIN_SPEC="./crates/node/chain-specs/madara-local-cfg2-raw.json" && \
RUST_LOG=runtime=trace,try-runtime::cli=trace,executor=trace \
cargo \
    run \
    --release \
    --features=try-runtime \
    --bin madara \
    -- \
    try-runtime \
    --runtime existing \
    --chain  "$CHAIN_SPEC" \
    follow-chain \
    --uri  "$RPC_WS_URL" \
 --state-root-check \
 --try-state all \
 --keep-connection
```

![](https://i.imgur.com/mfKgfM0.png)

================================================================

## fast-forward

> **live**

```bash
export RPC_WS_URL="ws://127.0.0.1:9931" && \
export CHAIN_SPEC="./crates/node/chain-specs/madara-local-cfg2-raw.json" && \
RUST_LOG=runtime=trace,try-runtime::cli=trace,executor=trace \
cargo \
    run \
    --release \
    --features=try-runtime \
    --bin madara \
    -- \
    try-runtime \
    --runtime existing \
    --chain  "$CHAIN_SPEC" \
    fast-forward \
 --n-blocks=2 \
 --try-state=all \
    live \
    --uri  "$RPC_WS_URL"
```

"ERROR main runtime: panicked at 'Timestamp slot must match `CurrentSlot`"

![](https://i.imgur.com/k7GW3Xb.png)

> **snap**

```bash
export RPC_WS_URL="ws://127.0.0.1:9931" && \
export SNAPSHOT_PATH="../000-01-install/madara-localnet.snap" && \
export CHAIN_SPEC="./crates/node/chain-specs/madara-local-cfg2-raw.json" && \
RUST_LOG=runtime=trace,try-runtime::cli=trace,executor=trace \
cargo \
    run \
    --release \
    --features=try-runtime \
    --bin madara \
    -- \
    try-runtime \
    --runtime existing \
    --chain  "$CHAIN_SPEC" \
    fast-forward \
 --block-ws-uri="$RPC_WS_URL" \
 --try-state=all \
 --n-blocks=2 \
    snap \
    --snapshot-path "$SNAPSHOT_PATH"
```

"ERROR main runtime: panicked at 'Timestamp slot must match `CurrentSlot`"

![](https://i.imgur.com/lFMBlPP.png)

================================================================
