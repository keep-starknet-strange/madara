<div align="center">
  <h1>Madara</h1>
  <img src="docs/images/madara-no-bg.png" height="256">
  <br />
  <a href="https://github.com/keep-starknet-strange/madara/issues/new?assignees=&labels=bug&template=01_BUG_REPORT.md&title=bug%3A+">Report a Bug</a>
  -
  <a href="https://github.com/keep-starknet-strange/madara/issues/new?assignees=&labels=enhancement&template=02_FEATURE_REQUEST.md&title=feat%3A+">Request a Feature</a>
  -
  <a href="https://github.com/keep-starknet-strange/madara/discussions">Ask a Question</a>
</div>

<div align="center">
<br />

[![GitHub Workflow Status](https://github.com/keep-starknet-strange/madara/actions/workflows/test.yml/badge.svg)](https://github.com/keep-starknet-strange/madara/actions/workflows/test.yml)
[![Project license](https://img.shields.io/github/license/keep-starknet-strange/madara.svg?style=flat-square)](LICENSE)
[![Pull Requests welcome](https://img.shields.io/badge/PRs-welcome-ff69b4.svg?style=flat-square)](https://github.com/keep-starknet-strange/madara/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22)

</div>

## About

Madara is a ⚡ blazing fast ⚡ Starknet sequencer, based on substrate and written in Rust 🦀.

This README file provides an overview of the Madara project, including its structure, components, and instructions for building, testing and running benchmarks.

## Architecture

Here is a high level overview of the current architecture of Starknet sequencer.

![](docs/images/starknet-sequencer-architecture.png)

## Project Structure

The Madara project consists of the following directories:

- `benchmarking`: Contains the code for benchmarking the custom FRAME pallets.
- `crates`: Holds all the crates used by the project, organized into the following subdirectories:
  - `node`: Implements services for the blockchain node (e.g., chain specification, RPC, etc.).
  - `pallets`: Contains custom FRAME pallets, including:
    - `pallet-starknet`: The Starknet pallet.
  - `runtime`: Assembles Madara's custom logic with the configured pallets.
  - `primitives`: Stores primitives used by the pallets.
- `docs`: Contains the project's documentation.
- `examples`: Provides example implementations for the project.
- `infra`: Houses infrastructure-related components, such as deployment scripts and Dockerfiles.

### Node

Madara node expose a number of capabilities:

- Networking: Substrate nodes use the [`libp2p`](https://libp2p.io/) networking stack to allow the
  nodes in the network to communicate with one another.
- Consensus: Blockchains must have a way to come to
  [consensus](https://docs.substrate.io/main-docs/fundamentals/consensus/) on the state of the
  network. Substrate makes it possible to supply custom consensus engines and also ships with
  several consensus mechanisms that have been built on top of
  [Web3 Foundation research](https://research.web3.foundation/en/latest/polkadot/NPoS/index.html).
- RPC Server: A remote procedure call (RPC) server is used to interact with Substrate nodes.

There are several files in the `node` directory - take special note of the following:

- [`chain_spec.rs`](./crates/node/src/chain_spec.rs): A
  [chain specification](https://docs.substrate.io/main-docs/build/chain-spec/) is a
  source code file that defines a Substrate chain's initial (genesis) state. Chain specifications
  are useful for development and testing, and critical when architecting the launch of a
  production chain. Take note of the `development_config` and `testnet_genesis` functions, which
  are used to define the genesis state for the local development chain configuration. These
  functions identify some
  [well-known accounts](https://docs.substrate.io/reference/command-line-tools/subkey/)
  and use them to configure the blockchain's initial state.
- [`service.rs`](./crates/node/src/service.rs): This file defines the node implementation. Take note of
  the libraries that this file imports and the names of the functions it invokes.

After the node has been [built](#build), refer to the embedded documentation to learn more about the
capabilities and configuration parameters that it exposes:

```shell
./target/release/madara --help
```
### Runtime

In Substrate, the terms
"runtime" and "state transition function"
are analogous - they refer to the core logic of the blockchain that is responsible for validating
blocks and executing the state changes they define. The Substrate project in this repository uses
[FRAME](https://docs.substrate.io/main-docs/fundamentals/runtime-intro/#frame) to construct a
blockchain runtime. FRAME allows runtime developers to declare domain-specific logic in modules
called "pallets". At the heart of FRAME is a helpful
[macro language](https://docs.substrate.io/reference/frame-macros/) that makes it easy to
create pallets and flexibly compose them to create blockchains that can address
[a variety of needs](https://substrate.io/ecosystem/projects/).

Review the [FRAME runtime implementation](./crates/runtime/src/lib.rs) included in this template and note
the following:

- This file configures several pallets to include in the runtime. Each pallet configuration is
  defined by a code block that begins with `impl $PALLET_NAME::Config for Runtime`.
- The pallets are composed into a single runtime by way of the
  [`construct_runtime!`](https://crates.parity.io/frame_support/macro.construct_runtime.html)
  macro, which is part of the core
  FRAME Support [system](https://docs.substrate.io/reference/frame-pallets/#system-pallets) library.

### Pallets

The runtime in this project is constructed using the pallets required for the Starknet sequencer implementation.

A FRAME pallet is compromised of a number of blockchain primitives:

- Storage: FRAME defines a rich set of powerful
  [storage abstractions](https://docs.substrate.io/main-docs/build/runtime-storage/) that makes
  it easy to use Substrate's efficient key-value database to manage the evolving state of a
  blockchain.
- Dispatchables: FRAME pallets define special types of functions that can be invoked (dispatched)
  from outside of the runtime in order to update its state.
- Events: Substrate uses [events and errors](https://docs.substrate.io/main-docs/build/events-errors/)
  to notify users of important changes in the runtime.
- Errors: When a dispatchable fails, it returns an error.
- Config: The `Config` configuration interface is used to define the types and parameters upon
  which a FRAME pallet depends.


### Benchmarking

Benchmarking allows you to assess the performance of the project's pallets. To run the benchmarks, follow the instructions in the [benchmarking](./benchmarking/README.md) document.

## Getting Started

Follow the steps below to get started with Madara :hammer_and_wrench:

### Using Nix

Install [nix](https://nixos.org/) and optionally [direnv](https://github.com/direnv/direnv) and
[lorri](https://github.com/nix-community/lorri) for a fully plug and play experience for setting up
the development environment. To get all the correct dependencies activate direnv `direnv allow` and
lorri `lorri shell`.

### Rust Setup

First, complete the [basic Rust setup instructions](./docs/rust-setup.md).

### Run

Use Rust's native `cargo` command to build and launch the template node:

```sh
cargo run --release -- --dev
```

Log level can be specified with `-l` flag. For example, `-ldebug` will show debug logs.
It can also be specified via the `RUST_LOG` environment variable. For example:

```sh
RUSTLOG=runtime=info cargo run --release -- --dev
```

### Build

The `cargo run` command will perform an initial build. Use the following command to build the node
without launching it:

```sh
cargo build --release
```

### Embedded Docs

Once the project has been built, the following command can be used to explore all parameters and
subcommands:

```sh
./target/release/madara -h
```

## Run

The provided `cargo run` command will launch a temporary node and its state will be discarded after
you terminate the process. After the project has been built, there are other ways to launch the
node.

### Single-Node Development Chain

This command will start the single-node development chain with non-persistent state:

```bash
./target/release/madara --dev
```

Purge the development chain's state:

```bash
./target/release/madara purge-chain --dev
```

Start the development chain with detailed logging:

```bash
RUST_BACKTRACE=1 ./target/release/madara -ldebug --dev
```

> Development chain means that the state of our chain will be in a tmp folder while the nodes are
> running. Also, **alice** account will be authority and sudo account as declared in the
> [genesis state](https://github.com/substrate-developer-hub/substrate-madara/blob/main/node/src/chain_spec.rs#L49).
> At the same time the following accounts will be pre-funded:
>
> - Alice
> - Bob
> - Alice//stash
> - Bob//stash

In case of being interested in maintaining the chain' state between runs a base path must be added
so the db can be stored in the provided folder instead of a temporal one. We could use this folder
to store different chain databases, as a different folder will be created per different chain that
is ran. The following commands shows how to use a newly created folder as our db base path.

```bash
// Create a folder to use as the db base path
$ mkdir my-chain-state

// Use of that folder to store the chain state
$ ./target/release/madara --dev --base-path ./my-chain-state/

// Check the folder structure created inside the base path after running the chain
$ ls ./my-chain-state
chains
$ ls ./my-chain-state/chains/
dev
$ ls ./my-chain-state/chains/dev
db keystore network
```

### Connect with Polkadot-JS Apps Front-end

Once the node template is running locally, you can connect it with **Polkadot-JS Apps** front-end
to interact with your chain. [Click
here](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944) connecting the Apps to your
local node template.

### Multi-Node Local Testnet

Build custom chain spec:

```bash
# Build plain chain spec
cargo run --release -- build-spec --chain local > infra/chain-specs/madara-local-testnet-plain.json
# Build final raw chain spec
cargo run --release -- build-spec --chain infra/chain-specs/madara-local-testnet-plain.json --raw > infra/chain-specs/madara-local-testnet.json
```

See more details about [custom chain specs](https://docs.substrate.io/reference/how-to-guides/basics/customize-a-chain-specification/).

Run the local testnet:

```bash
./infra/local-testnet/run.sh
```

### Set Ethereum Node URL for offchain worker

In order for the offchain worker to access an Ethereum RPC node, we need to set the URL for that in offchain local storage.
We can do that by making use of the default [`offchain` rpc calls](https://polkadot.js.org/docs/substrate/rpc/#offchain) provided by Substrate.

In the polkadot explorer, navigate to Developer > RPC calls and choose the `offchain` endpoint.
In there, you can set the value for `ETHEREUM_EXECUTION_RPC` by using the `localStorageSet` function.
You need to select the type of storage, in this case `PERSISTENT`, and use the `starknet::ETHEREUM_EXECUTION_RPC` as the `key`.
The value is the RPC URL you intend to use.

![](docs/images/madara-set-rpc-url-in-local-storage.png)


You can check that the value was properly set by using the `localStorageGet` function

![](docs/images/madara-get-rpc-url-from-local-storage.png)

### Run in Docker

First, install [Docker](https://docs.docker.com/get-docker/) and
[Docker Compose](https://docs.docker.com/compose/install/).

Then run the following command to start a single node development chain.

```bash
docker-compose -f infra/docker/docker-compose.yml up -d
```

This command will firstly compile your code, and then start a local development network. You can
also replace the default command
(`cargo build --release && ./target/release/madara --dev --ws-external`)
by appending your own. A few useful ones are as follow.

```bash
# Run Substrate node without re-compiling
./infra/docker_run.sh ./target/release/madara --dev --ws-external

# Purge the local dev chain
./infra/docker_run.sh ./target/release/madara purge-chain --dev

# Check whether the code is compilable
./infra/docker_run.sh cargo check
```
## Starknet features compatibility

See [Starknet features compatibility](docs/starknet_features_compatibility.md) for details.
## Roadmap

See the [open issues](https://github.com/keep-starknet-strange/madara/issues) for
a list of proposed features (and known issues).

- [Top Feature Requests](https://github.com/keep-starknet-strange/madara/issues?q=label%3Aenhancement+is%3Aopen+sort%3Areactions-%2B1-desc)
  (Add your votes using the 👍 reaction)
- [Top Bugs](https://github.com/keep-starknet-strange/madara/issues?q=is%3Aissue+is%3Aopen+label%3Abug+sort%3Areactions-%2B1-desc)
  (Add your votes using the 👍 reaction)
- [Newest Bugs](https://github.com/keep-starknet-strange/madara/issues?q=is%3Aopen+is%3Aissue+label%3Abug)

## Support

Reach out to the maintainer at one of the following places:

- [GitHub Discussions](https://github.com/keep-starknet-strange/madara/discussions)
- Contact options listed on
  [this GitHub profile](https://github.com/keep-starknet-strange)

## Project assistance

If you want to say **thank you** or/and support active development of Madara:

- Add a [GitHub Star](https://github.com/keep-starknet-strange/madara) to the
  project.
- Tweet about the Madara.
- Write interesting articles about the project on [Dev.to](https://dev.to/),
  [Medium](https://medium.com/) or your personal blog.

Together, we can make Madara **better**!

## Contributing

First off, thanks for taking the time to contribute! Contributions are what make
the open-source community such an amazing place to learn, inspire, and create.
Any contributions you make will benefit everybody else and are **greatly
appreciated**.

Please read [our contribution guidelines](docs/CONTRIBUTING.md), and thank you
for being involved!

## Authors & contributors

For a full list of all authors and contributors, see
[the contributors page](https://github.com/keep-starknet-strange/madara/contributors).

## Security

Madara follows good practices of security, but 100% security cannot be assured.
Madara is provided **"as is"** without any **warranty**. Use at your own risk.

_For more information and to report security issues, please refer to our
[security documentation](docs/SECURITY.md)._

## License

This project is licensed under the **MIT license**.

See [LICENSE](LICENSE) for more information.

## Troubleshooting

### error: failed to run custom build command for `libp2p-core v0.37.0` / Could not find `protoc` installation

<details>
<summary>Click to expand</summary>

#### Error

```text
error: failed to run custom build command for `libp2p-core v0.37.0`

Caused by:
  process didn't exit successfully: `...` (exit status: 101)
  --- stderr
  thread 'main' panicked at 'Could not find `protoc` installation and this build crate cannot proceed without
      this knowledge. If `protoc` is installed and this crate had trouble finding
      it, you can set the `PROTOC` environment variable with the specific path to your
      installed `protoc` binary.You could try running `brew install protobuf` or downloading it from https://github.com/protocolbuffers/protobuf/releases

  For more information: https://docs.rs/prost-build/#sourcing-protoc
  ', /Users/abdel/.cargo/registry/src/github.com-1ecc6299db9ec823/prost-build-0.11.4/src/lib.rs:1296:10
  note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

#### Solution

It means that you don't have `protoc` installed on your machine. You can install it using `brew install protobuf` on MacOs or downloading it from <https://github.com/protocolbuffers/protobuf/releases>.

</details>

### Rust WASM toolchain not installed, please install it

<details>
<summary>Click to expand</summary>

#### Error

```text
error: failed to run custom build command for `madara-runtime v4.0.0-dev (/Users/abdel/dev/me/madara/runtime)`

Caused by:
  process didn't exit successfully: `.../madara/target/release/build/madara-runtime-9df5c70f9d1b72f5/build-script-build` (exit status: 1)
  --- stderr
  Rust WASM toolchain not installed, please install it!

  Further error information:
  ------------------------------------------------------------
     Compiling wasm-test v1.0.0 (/var/folders/...)
  error[E0463]: can't find crate for `std`
    |
    = note: the `wasm32-unknown-unknown` target may not be installed
    = help: consider downloading the target with `rustup target add wasm32-unknown-unknown`
    = help: consider building the standard library from source with `cargo build -Zbuild-std`
```

#### Solution

It means that you don't have `wasm32-unknown-unknown` target installed on your machine. You can install it using `rustup target add wasm32-unknown-unknown`.

</details>

## Acknowledgements

## Contributors ✨

Thanks goes to these wonderful people ([emoji key](https://allcontributors.org/docs/en/emoji-key)):

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/abdelhamidbakhta"><img src="https://avatars.githubusercontent.com/u/45264458?v=4?s=100" width="100px;" alt="Abdel @ StarkWare "/><br /><sub><b>Abdel @ StarkWare </b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=abdelhamidbakhta" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/tdelabro"><img src="https://avatars.githubusercontent.com/u/34384633?v=4?s=100" width="100px;" alt="Timothée Delabrouille"/><br /><sub><b>Timothée Delabrouille</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=tdelabro" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/EvolveArt"><img src="https://avatars.githubusercontent.com/u/12902455?v=4?s=100" width="100px;" alt="0xevolve"/><br /><sub><b>0xevolve</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=EvolveArt" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/LucasLvy"><img src="https://avatars.githubusercontent.com/u/70894690?v=4?s=100" width="100px;" alt="Lucas @ StarkWare"/><br /><sub><b>Lucas @ StarkWare</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=LucasLvy" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/DavideSilva"><img src="https://avatars.githubusercontent.com/u/2940022?v=4?s=100" width="100px;" alt="Davide Silva"/><br /><sub><b>Davide Silva</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=DavideSilva" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://www.finiam.com/"><img src="https://avatars.githubusercontent.com/u/58513848?v=4?s=100" width="100px;" alt="Finiam"/><br /><sub><b>Finiam</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=finiam" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/ZePedroResende"><img src="https://avatars.githubusercontent.com/u/17102689?v=4?s=100" width="100px;" alt="Resende"/><br /><sub><b>Resende</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=ZePedroResende" title="Code">💻</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/drspacemn"><img src="https://avatars.githubusercontent.com/u/16685321?v=4?s=100" width="100px;" alt="drspacemn"/><br /><sub><b>drspacemn</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=drspacemn" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/tarrencev"><img src="https://avatars.githubusercontent.com/u/4740651?v=4?s=100" width="100px;" alt="Tarrence van As"/><br /><sub><b>Tarrence van As</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=tarrencev" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://home.cse.ust.hk/~shanaj/"><img src="https://avatars.githubusercontent.com/u/47173566?v=4?s=100" width="100px;" alt="Siyuan Han"/><br /><sub><b>Siyuan Han</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=hsyodyssey" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://zediogoviana.github.io/"><img src="https://avatars.githubusercontent.com/u/25623039?v=4?s=100" width="100px;" alt="Zé Diogo"/><br /><sub><b>Zé Diogo</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=zediogoviana" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Matth26"><img src="https://avatars.githubusercontent.com/u/9798638?v=4?s=100" width="100px;" alt="Matthias Monnier"/><br /><sub><b>Matthias Monnier</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/madara/commits?author=Matth26" title="Code">💻</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind welcome!
