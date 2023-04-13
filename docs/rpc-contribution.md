# Madara RPC contribution (for new contributors)

This file is intented for very new contributor to onboard on the [madara](https://github.com/keep-starknet-strange/madara)
project, the sequencer of [Starknet](https://docs.starknet.io/documentation/).

At the [last community call](https://www.youtube.com/watch?v=VyvDAxF46uc), a special accent was put on RPC contributions to
make madara quickly featured to be queried as a fullnode.

Here is a little guide to quickly dive into madara project, focused on RPC.

## How to build madara

First, go ahead and clone madara on the `main` branch from https://github.com/keep-starknet-strange/madara.

There are two ways you can build madara to quickly test it:

1. `cargo build --release`, which will then allow us to start madara running `./target/release/madara`.
    This will start the sequencer WITHOUT peers. That's not a problem if you just want to test that your RPC method is accessible,
    and to test [de]serialization of your RPC parameters.
    
    Some libraries that you may required on linux before running `cargo build` command (not exhaustive):
    `protobuf-compiler build-essential g++ clang`.
    
2. Using docker as exposed in the [README](https://github.com/keep-starknet-strange/madara#run-in-docker). 
   With the command `docker-compose -f infra/docker/docker-compose.yml up`, you start
   madara with a genesis block already configured + some accounts + some peers. This is very useful if you want to test RPC methods
   that are targetting the transactions / blocks / etc...
   
   This is the preferred method, but using the method 1 can be helpful for a quick start of madara and playing with RPC and substrate.
   

## Quick intro on Madara architecture
Madara is being built on [Substrate](https://docs.substrate.io/learn/welcome-to-substrate/), which already proposes
an architecture for a modular blockchain development.

To do it short, madara is considered as a `substrate node`, which means madara is being developped using the
SDK proposed by substrate to build a node of a blockchain.

A node can be split in two big components:
1. A **client**, where all the very common logic of blockchain's node lies. This includes networking, storage, etc...
   This is in the client where RPC is implemented, as an RPC is nothing more than a common server accepting HTTP requests.
   However, substrate base libraries can be extended, and that's the beauty of it. So we can customize our RPC (among others).

2. A **runtime**, where the business logic of the blockchain is implemented (eg: which transaction is valid or not).
   The runtime can be resumed as rust code being compiled to WASM and executed by the node of the blockchain. Runtime is constructed
   on the top of a development library called FRAME, where developers work with PALLETS to customize the runtime behavior.

Therefore, if we go into the source code of madara, there is a folder named `crates/client` which contains the code
related to the client component exposed in the point 1.


## How to expose a RPC endpoint

First, revise the [RPC spec](https://github.com/keep-starknet-strange/madara/blob/b5367e0cead0abfef77c13a628c08c64beb1b3aa/crates/client/rpc-core/starknet_openRPC.json) from madara project
to check what are the parameters and return value that are assigned to the endpoint you will implement.

In the `creates/client` we can find two RPC related packages.
1. `rpc-core`: exposes a trait that defines `StarknetRpcApi`. This is where we must define our endpoint "signature".
   We need a structs to be [de]serialized if some parameters must be passed / returned. 

```rust
// crates/client/rpc-core/src/lib.rs

// Note here the macro to ensure correct serialization.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct MyEndpointParams {
    pub some_str: String,
    pub some_u64: u64,
}

// If needed, define MyEndpointResult for instance.

...

/// Starknet rpc interface.
#[rpc(server, namespace = "starknet")]
pub trait StarknetRpcApi {
    /// Get the most recent accepted block number
    #[method(name = "blockNumber")]
    fn block_number(&self) -> RpcResult<BlockNumber>;

    ....

    /// My new RPC endpoint.
    #[method(name = "myEndpoint")] // <-- camel case naming.
    fn my_endpoint(&self, my_params: MyEndpointParams) -> RpcResult<String>; // <-- Define strucs as needed for params or result.
}
```


2. `rpc`: implements actual RPC logic to process the parameters (if any) and return a result.

```rust
// crates/client/rpc/src/lib.rs

...
use mc_rpc_core::{BlockHashAndNumber, BlockId as StarknetBlockId, MyEndpointParams};
...

impl<B, BE, C> StarknetRpcApiServer for Starknet<B, BE, C>
where
    B: BlockT,
    BE: Backend<B> + 'static,
    C: HeaderBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
{
    ...
    
    /// New endpoint for an amazing feature.
    fn my_endpoint(&self, my_params: MyEndpointParams) -> RpcResult<String> {
        // Here comes the logic to interact with storage, etc...
        Ok(String::from("Let's build the future!"))
    }
    
}

```

Recompile madara (with method 1 or 2 depending on your needs), and you should be able to target your new endpoint.
The endpoint name must be prefixed by `starknet_` to be routed correctly. The camel case name must be used.

```
curl -X POST \
     -H 'Content-Type: application/json' \
     -d '{"jsonrpc":"2.0","id":1,"method":"starknet_myEndpoint","params":[{"some_str": "Madara", "some_u64": 1234}]}' \
     http://localhost:9944
```

