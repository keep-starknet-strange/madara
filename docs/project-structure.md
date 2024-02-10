## Project Structure

The Madara project consists of the following directories:

- `benchmarking`: Contains the code for benchmarking the custom FRAME pallets.
- `crates`: Holds all the crates used by the project, organized into the
  following subdirectories:
  - `node`: Implements services for the blockchain node (e.g., chain
    specification, RPC, etc.).
  - `pallets`: Contains custom FRAME pallets, including:
    - `pallet-starknet`: The Starknet pallet.
  - `runtime`: Assembles Madara's custom logic with the configured pallets.
  - `primitives`: Stores primitives used by the pallets.
- `docs`: Contains the project's documentation.
- `examples`: Provides example implementations for the project.

### Node

Madara node exposes a number of capabilities:

- Networking: Substrate nodes use the [`libp2p`](https://libp2p.io/) networking
  stack to allow the nodes in the network to communicate with one another.
- Consensus: Blockchains must have a way to come to
  [consensus](https://docs.substrate.io/main-docs/fundamentals/consensus/) on
  the state of the network. Substrate makes it possible to supply custom
  consensus engines and also ships with several consensus mechanisms that have
  been built on top of
  [Web3 Foundation research](https://research.web3.foundation/Polkadot/protocols/NPoS).
- RPC Server: A remote procedure call (RPC) server is used to interact with
  Substrate nodes.

There are several files in the `node` directory - take special note of the
following:

- [`chain_spec.rs`](../crates/node/src/chain_spec.rs): A
  [chain specification](https://docs.substrate.io/main-docs/build/chain-spec/)
  is a source code file that defines a Substrate chain's initial (genesis)
  state. Chain specifications are useful for development and testing, and
  critical when architecting the launch of a production chain. Take note of the
  `development_config` and `testnet_genesis` functions, which are used to define
  the genesis state for the local development chain configuration. These
  functions identify some
  [well-known accounts](https://docs.substrate.io/reference/command-line-tools/subkey/)
  and use them to configure the blockchain's initial state.
- [`service.rs`](../crates/node/src/service.rs): This file defines the node
  implementation. Take note of the libraries that this file imports and the
  names of the functions it invokes.

After the node has been [built](#build), refer to the embedded documentation to
learn more about the capabilities and configuration parameters that it exposes:

```shell
./target/release/madara --help
```

### Runtime

In Substrate, the terms "runtime" and "state transition function" are
analogous - they refer to the core logic of the blockchain that is responsible
for validating blocks and executing the state changes they define. The Substrate
project in this repository uses
[FRAME](https://docs.substrate.io/reference/glossary/#frame) to construct a
blockchain runtime. FRAME allows runtime developers to declare domain-specific
logic in modules called "pallets". At the heart of FRAME is a helpful
[macro language](https://docs.substrate.io/reference/frame-macros/) that makes
it easy to create pallets and flexibly compose them to create blockchains that
can address [a variety of needs](https://substrate.io/ecosystem/projects/).

Review the [FRAME runtime implementation](../crates/runtime/src/lib.rs) included
in this template and note the following:

- This file configures several pallets to include in the runtime. Each pallet
  configuration is defined by a code block that begins with
  `impl $PALLET_NAME::Config for Runtime`.
- The pallets are composed into a single runtime by way of the
  [`construct_runtime!`](https://crates.parity.io/frame_support/macro.construct_runtime.html)
  macro, which is part of the core FRAME Support
  [system](https://docs.substrate.io/reference/frame-pallets/#system-pallets)
  library.

### Pallets

The runtime in this project is constructed using the pallets required for the
Starknet sequencer implementation.

A FRAME pallet is compromised of a number of blockchain primitives:

- Storage: FRAME defines a rich set of powerful
  [storage abstractions](https://docs.substrate.io/main-docs/build/runtime-storage/)
  that makes it easy to use Substrate's efficient key-value database to manage
  the evolving state of a blockchain.
- Dispatchables: FRAME pallets define special types of functions that can be
  invoked (dispatched) from outside of the runtime in order to update its state.
- Events: Substrate uses
  [events and errors](https://docs.substrate.io/main-docs/build/events-errors/)
  to notify users of important changes in the runtime.
- Errors: When a dispatchable fails, it returns an error.
- Config: The `Config` configuration interface is used to define the types and
  parameters upon which a FRAME pallet depends.
