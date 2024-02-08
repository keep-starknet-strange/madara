# Madara RPC contribution (for new contributors)

This file is intended for very new contributors to onboard on the
[madara](https://github.com/keep-starknet-strange/madara) project, the sequencer
of [Starknet](https://docs.starknet.io/documentation/).

At the [last community call](https://www.youtube.com/watch?v=VyvDAxF46uc), a
special accent was put on RPC contributions to make madara quickly featured to
be queried as a full node.

Here is a little guide to quickly dive into the madara project, focused on RPC.

## How to build madara

First, go ahead and clone madara on the `main` branch from
<https://github.com/keep-starknet-strange/madara>.

There are two ways you can build madara to quickly test it:

1. `cargo build --release`, which will then allow us to setup madara with
   `./target/release/madara setup`, and then run it with
   `./target/release/madara`. This will start the sequencer WITHOUT peers.
   That's not a problem if you just want to test that your RPC method is
   accessible, and to test (de)serialization of your RPC parameters.

   Some libraries that you may require on linux before running `cargo build`
   command (not exhaustive): `protobuf-compiler build-essential g++ clang`.

2. Using docker, with the command `docker run [TAG] --dev`, you start madara.
   The TAGS are available
   [here](https://github.com/keep-starknet-strange/madara/pkgs/container/madara).
   This is very useful if you want to test RPC methods that are targeting the
   transactions / blocks / etc...

   This is the preferred method, but using the method 1 can be helpful for a
   quick start of madara and playing with RPC and substrate.

## Quick intro on Madara architecture

Madara is being built on
[Substrate](https://docs.substrate.io/learn/welcome-to-substrate/), which
already proposes an architecture for modular blockchain development.

To do it short, madara is considered as a `substrate node`, which means madara
is being developed using the SDK proposed by substrate to build a node of a
blockchain.

A node can be split in two big components:

1. A **client**, where all the very common logic of blockchain's node lies. This
   includes networking, storage, etc... This is in the client where RPC is
   implemented, as an RPC is nothing more than a common server accepting HTTP
   requests. However, substrate base libraries can be extended, and that's the
   beauty of it. So we can customize our RPC (among others).

2. A **runtime**, where the business logic of the blockchain is implemented (eg:
   which transaction is valid or not). The runtime can be resumed as rust code
   being compiled to WASM and executed by the node of the blockchain. Runtime is
   constructed on the top of a development library called
   [FRAME](https://docs.substrate.io/reference/frame-pallets/), where developers
   work with PALLETS to customize the runtime behavior.

Therefore, if we go into the source code of madara, there is a folder named
`crates/client` which contains the code related to the client component exposed
in the point 1.

## How to expose a RPC endpoint

First, revise the
[RPC spec](https://github.com/keep-starknet-strange/madara/blob/main/crates/client/rpc-core/starknet_openRPC.json)
from madara project to check what are the parameters and return value that are
assigned to the endpoint you will implement.

In the `crates/client` we can find two RPC related packages.

1. `rpc-core`: exposes a trait that defines `StarknetRpcApi`. This is where we
   must define our endpoint "signature". We need a struct to be (de)serialized
   if some parameters must be passed / returned.

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
       fn my_endpoint(&self, my_params: MyEndpointParams) -> RpcResult<String>; // <-- Define struct as needed for params or result.
   }
   ```

2. `rpc`: implements actual RPC logic to process the parameters (if any) and
   return a result.

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

           // If you need to access the runtime, you can use the following code:
           let runtime_api = self.client.runtime_api();
       }

   }
   ```

Quite often you will need to interact with the runtime, in order to access
storage or call internal functions. To do so, follow these steps:

1. Add your function signature to the runtime api

   ```rust
   // crates/pallets/starknet/src/runtime_api.rs

   use mp_starknet::execution::ContractAddressWrapper;
   use sp_core::{H256, U256};
   pub extern crate alloc;
   use alloc::vec::Vec;

   use sp_runtime::DispatchError;

   // /!\ You should be using runtime types here.

   sp_api::decl_runtime_apis! {
       pub trait StarknetRuntimeApi {
           /// Returns a `Call` response.
           fn call(address: ContractAddressWrapper, function_selector: Felt252Wrapper, calldata: Vec<Felt252Wrapper>) -> Result<Vec<Felt252Wrapper>, DispatchError>;
           /// Your new function.
           fn my_function() -> H256;
       }
   }
   ```

2. Implement your function in the runtime

   ```rust
   // crates/runtime/src/lib.rs

   impl pallet_starknet::runtime_api::StarknetRuntimeApi<Block> for Runtime {

         fn call(address: ContractAddressWrapper, function_selector: Felt252Wrapper, calldata: Vec<Felt252Wrapper>) -> Result<Vec<Felt252Wrapper>, DispatchError> {
             Starknet::call_contract(address, function_selector, calldata)
         }

         fn my_function() -> H256 {
             // Here comes the logic to interact with storage, pallets...
             H256::from_low_u64_be(1234)
         }
   }
   ```

Great, now it's finally time to write some integration tests to ensure
everything is working as expected.

## Integration tests

Integration tests are located in the `starknet-rpc-test` folder, and are written
in rust using `rstest`. We use `starknet-rs` to interact with the blockchain and
test compatibility with Starknet's tooling.

You can find the documentation on this
[link](https://github.com/xJonathanLEI/starknet-rs).

```rust
#[rstest]
#[tokio::test]
async fn fail_non_existing_block(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    // We retrieve the madara client
    let madara = madara.await;

    // We get the RPC Provider to interact with the madara node
    let rpc = madara.get_starknet_client();

    // Expected values
    let test_contract_class_hash =
        FieldElement::from_hex_be(TEST_CONTRACT_CLASS_HASH).expect("Invalid Contract Address");

    // Assertions
    assert_matches!(
        rpc
        .get_class(
            BlockId::Number(100),
            test_contract_class_hash,
        )
        .await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::BlockNotFound
    );

    Ok(())
}
```

Recompile madara (with method 1 or 2 depending on your needs), and you should be
able to target your new endpoint.

### Run your integration tests

Prior to running the test, you will need to run the
generate_declare_contracts.sh script
`cd starknet-rpc-test/contracts && ./generate_declare_contracts.sh 8`

To run the tests, simply run
`cargo test -p starknet-rpc-test -- test <test_file> -- <test_name> --exact --nocapture --test-threads=1`.

For easier debugging make sure to enable the background node's logs with
`MADARA_LOG=true`.

e.g

```bash
MADARA_LOG=true cargo test --package starknet-rpc-test -- --exact --nocapture --test-threads=1
```

### Test locally

The endpoint name must be prefixed by `starknet_` to be routed correctly. The
camel case name must be used.

```sh
curl -X POST \
     -H 'Content-Type: application/json' \
     -d '{"jsonrpc":"2.0","id":1,"method":"starknet_myEndpoint","params":[{"some_str": "Madara", "some_u64": 1234}]}' \
     http://localhost:9933
```

### Testing Madara RPC Endpoints automatically

To test the Madara RPC endpoints, follow the steps below:

Run Madara locally (by default, it runs on port 9933):

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
