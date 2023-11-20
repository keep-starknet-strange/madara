## Getting Started

Follow the steps below to get started with Madara :hammer_and_wrench:

### Rust Setup

First, Install rust using the [rustup](https://rustup.rs/) toolchain installer,
then run:

```bash
rustup show
```

### Single-Node Development Chain

Use Rust's native `cargo` command to build and launch the template node:

You first need to setup up the node, which means you need to load the genesis
state into your file system.

```sh
cargo run --release -- setup --chain=dev --from-remote
```

Now, you can start the node in development mode

```sh
cargo run --release -- --dev
```

### Interacting with the node

Madara is compatible with the Starknet
[spec](https://github.com/starkware-libs/starknet-specs) which means all tooling
around Starknet (starknet-js, starknet-rs, wallets, etc.) can be used out of the
box by just changing the RPC url to point to your node. By default, this would
be `http://localhost:9944`.

### Common chain flags

You can check all the available using the `--help` flag. Some common points to
know about have been mentioned below.

Madara overrides the default `dev` flag in substrate to meet its requirements.
The following flags are automatically enabled with the `--dev` argument:

`--chain=dev`, `--force-authoring`, `--alice`, `--tmp`, `--rpc-external`,
`--rpc-methods=unsafe`

The `--tmp` flag stores the chain database in a temporary folder. You can
specify a custom folder to store the chain state by using the `--base-path`
flag. You cannot combine the `base-path` command with `--dev` as `--dev`
enforces `--tmp` which will store the db at a temporary folder. You can,
however, manually specify all flags that the dev flag adds automatically. Keep
in mind, the path must be the same as the one you used in the setup command.

The node also supports to use manual seal (to produce block manually through
RPC).

```sh
cargo run --release -- --dev --sealing=manual
# Or
cargo run --release -- --dev --sealing=instant
```

Log level can be specified with `-l` flag. For example, `-ldebug` will show
debug logs. It can also be specified via the `RUST_LOG` environment variable.
For example:

```sh
RUSTLOG=runtime=info cargo run --release -- --dev
```

### Using Nix (optional, only for degens)

Install [nix](https://nixos.org/) and optionally
[direnv](https://github.com/direnv/direnv) and
[lorri](https://github.com/nix-community/lorri) for a fully plug and play
experience for setting up the development environment. To get all the correct
dependencies activate direnv `direnv allow` and lorri `lorri shell`.

### Embedded Docs

Once the project has been built, the following command can be used to explore
all parameters and subcommands:

```sh
./target/release/madara -h
```

### Connect with Polkadot-JS Apps Front-end

Once the node template is running locally, you can connect it with **Polkadot-JS
Apps** front-end to interact with your chain.
[Click here](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944)
connecting the Apps to your local node template.

### Multi-Node Local Testnet

Build custom chain spec:

```bash
# Build plain chain spec
cargo run --release -- build-spec --chain local > chain-specs/madara-local-testnet-plain.json
# Build final raw chain spec
cargo run --release -- build-spec --chain chain-specs/madara-local-testnet-plain.json --raw > chain-specs/madara-local-testnet.json
```

See more details about
[custom chain specs](https://docs.substrate.io/reference/how-to-guides/basics/customize-a-chain-specification/).

### Testing Madara RPC Endpoints

To test the Madara RPC endpoints, follow the steps below:

Run Madara locally (by default, it runs on port 9944):

```bash
cargo run --release -- --dev
# Alternatively, use other methods to run Madara
```

Execute hurl tests sequentially:

```bash
hurl --variables-file examples/rpc/hurl.config  --test examples/rpc/**/*.hurl
```

The output should be similar to the image provided:

![Hurl Test Output](./images/hurl-test-output.png)

### Set Ethereum Node URL for offchain worker

In order for the offchain worker to access an Ethereum RPC node, we need to set
the URL for that in offchain local storage. We can do that by making use of the
default
[`offchain` rpc calls](https://polkadot.js.org/docs/substrate/rpc/#offchain)
provided by Substrate.

In the polkadot explorer, navigate to Developer > RPC calls and choose the
`offchain` endpoint. In there, you can set the value for
`ETHEREUM_EXECUTION_RPC` by using the `localStorageSet` function. You need to
select the type of storage, in this case `PERSISTENT`, and use the
`starknet::ETHEREUM_EXECUTION_RPC` as the `key`. The value is the RPC URL you
intend to use.

![](./images/madara-set-rpc-url-in-local-storage.png)

You can check that the value was properly set by using the `localStorageGet`
function

![](./images/madara-get-rpc-url-from-local-storage.png)

### Run in Docker

First, install [Docker](https://docs.docker.com/get-docker/) and
[Docker Compose](https://docs.docker.com/compose/install/).

Then run the following command to start a single node development chain.

```bash
docker run --rm [TAG] --dev
```

This command will firstly compile your code, and then start a local development
network. The TAGS are available
[here](https://github.com/keep-starknet-strange/madara/pkgs/container/madara).

You can also use the command appending your own options. A few useful ones are
as follow.

```bash
# Run Substrate node without re-compiling
docker run --rm [TAG] --dev --ws-external

# Purge the local dev chain
docker run --rm [TAG] purge-chain --dev
```
