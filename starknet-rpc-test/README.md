# Starknet RPC tests

Starknet RPC rests are run against a single Madara instance (in order to reduce
the CI pipeline time).  
In order to run tests locally you will need to: 0. Make sure you have all test
dependencies compiled (see next section)

1. Build Madara binary: `cargo build --profile release`
2. Setup Madara instance (once):
   `./target/release/madara setup --dev --from-local=./configs`
3. Run Madara node: `./target/release/madara --dev --sealing=manual`
4. Run tests
   `cargo test --package starknet-rpc-test works_with_storage_change -- --nocapture`

If you need to reset the state, run `./target/release/madara purge-chain --dev`

## Generate Cairo contract artifacts

Make sure you have the exact scarb version installed as in
`contracts/.tool-versions`. Follow the instructions:
<https://docs.swmansion.com/scarb/download.html#install-via-installation-script>

It's necessary to compile the same contract several times with small
modifications so that it has different class hash.  
The artifacts are used for testing the declare class operation.

```sh
cd starknet-rpc-test/contracts
./generate_declare_contracts.sh 10
```
