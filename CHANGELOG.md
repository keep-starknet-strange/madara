# Madara Changelog

## Next release

- fix: updating outdated links to external resources in documentation
- feat(client/data-availability): implement custom error handling
- fix: get_block_by_block_hash then default rather than error
- feat(rpc): added `get_state_update` real values from DA db
- feat: add transparent representation to `Felt252Wrapper`
- feat(rpc/trace_api): add `trace_block_transaction`
- chore(db): changed the way hashes are encoded

## v0.7.0

- chore: release v0.7.0
- refacto: remove abusive `TryInto` impl
- dev: optimize tx trace creation
- dev: make Madara std compatible
- CI: fix taplo version
- chore: add cache usage for `getEvents` and `getTransactionReceipt`
- fix: cairo1 contracts should be identified by their sierra class hash
- fix(cli): repair broken cli for da conf
- feat(client): on `add_declare_transaction` store sierra contract classes in
  the madara backend
- chore: use struct error in client/db
- fix: don't ignore Sierra to CASM mapping in genesis config
- refacto: early exit txs fee estimation when one fails
- dev: fix linter warning in README.md
- fix: remove waiting loop from `getTxReceipt`
- feat: types in `mp-transactions` impl a method to get their version
- feat: make L1 gas price a `const` of the `RuntimeConfig`
- fix: broken class hashes and contracts in genesis
- refactor: rename LAST_SYNCED_L1_BLOCK to be more clear
- chore: add headers to da calldata, fix eth da in sovereign mode
- refacto(simulate_tx): move logic to the client
- chore: added ca-certificate in DockerFile for SSL related issues
- chore(primitives/commitment): remove crate
- chore(primitives/block/header): remove starknet-trie dependent fields
- refacto(primitives/db): add a temporary way to get a fake global state root
- chore: feature flags for avail and celestia DA
- feat(rpc): added support for v0.5.1 JSON-RPC specs
- feat(rpc): added ordered messages/events in trace fields
- feat(rpc): support for starknet.rs v0.5.1 version
- feat(rpc): added execution resources in trace fields
- feat(rpc): added state diff field in trace fields
- refactor: removed benchmarking folder and traces of CI pipeline
- fix: decouple is_query into is_query and offset_version
- feat: add sierra to casm class hash mapping to genesis assets
- chore: remove ArgentMulticall from genesis assets
- feat: remove `seq_addr_updated` from `GenesisData`
- chore: added prometheus metrics for da layer
- chore: bump celestia rpc crate version
- fix(DA): run the proof first then the state update
- fix: `prove_current_block` is called after `update_state`
- ci: add foundry ci task to push workflow
- fix: first tx for non deployed account is valid
- fix: incorrect base url for fetching config
- feat: add predeployed accounts to genesis state
- feat(rpc): Added starknet_simulateTransactions
- fix: Change serialization of bitvec to &[u8] in merkle tree to avoid memory
  uninitialized
- chore: change SCARB config version for foundry CI
- feat(da): update da calldata encoding to v0.11.0 spec, da conf examples, da
  conf flag, da-tests in CI
- refactor: use `map` in `estimate_fee` to stop computation on error
- fix(node/commands): md5 are also checked when running setup --from-local
- feat(data-availability): extend eth config with poll interval
- fix(snos-output): expose snos codec, remove unused `get_starknet_messages`
  runtime method, and unnecessary mp-snos-output dependencies
- feat(program-hash): add new pallet constant for Starknet OS progam hash;
  expose runtime getter method; add dedicated crate to manage versions
- feat(runtime): expose fee token address getter method
- feat(settlement): run client thread responsible for pushing state updates and
  messaging on Ethereum
- feat(settlement): starknet core contract tests with anvil sandbox
- fix(rpc-test): incorrect node url
- feat(settlement): e2e test with Madara node settling on Ethereum contract
- refactor: use `map` in `estimate_fee` to stop computation on error
- fix: `tempdir` crate has been deprecated; use `tempfile` instead
- dev: add avail and celestia crates behind a feature flag
- dev: replace md5 with sha3_256 hash function
- feat: fixing getNonce Rpc Call and adding a new test
- refactor: use Zaun crate for Starknet core contract bindings
- refactor: use Anvil sandbox from Zaun crate
- feat(rpc): estimateMessageFee RPC call implementation

## v0.6.0

- chore: release v0.6.0
- refacto: substrate/starknet names in rpc library
- feat(rpc): Added starknet_getTransactionStatus and removed
  starknet_pendingTransactions
- feat(rpc): add starknet_specVersion rpc + added test for future support
- docs: Added v0.6.0-rc5 documentation above the rpc method functions
- dev(deps): bump starknet rs, use Eq for EmmitedEvents comparaison
- test(rust-rpc-test): use undeclared contracts for declare transactions testing
- build: update blockifier, fix divergent substrat block hash
- chore: remove tests that run in wasm and native, only wasm from now
- chore: split StarknetRpcApi trait in two, like in openRPC specs
- refacto: move starknet runtime api in it's own crate
- chore: update README.md and getting-started.md
- chore: remove crates that have been copy-pasted from plkdtSDK
- feat(rpc): return deployed contract address and actual fee in transaction
  receipt
- fix: Wait for 1 minute for transaction to be processed in
  get_transaction_receipt rpc
- ci: Fix starknet foundry sncast not found
- fix: Ensure transaction checks are compatible with starknet-rs
- ci: Run Starknet Foundry tests against Madara RPC
- fix: add name, symbol and decimals to fee token storage
- fix: dependencies for dockerfile and binaries
- docs: add translation of madara beast article to spanish
- chore: update starknet-js version in faucet-setup docs
- dev(compilation): add incremental compilation
- feat(rpc): add support for bulk estimate fee
- feat: add argent multicall contract to genesis
- chore(data-availability): update avail-subxt to version 0.4.0
- fix(ci): setup should fetch files from local config
- chore: deprecate `madara-app` and `madara-dev-explorer` modules
- chore(data-availability-avail): implement fire and forget, and add ws
  reconnection logic
- chore: update `polkadot-sdk` to `release-polkadot-v1.3.0`
- feat: fallback default file for DA and Settlement configuration files

## v0.5.0

- chore: release v0.5.0
- test: add transaction pool logic unit tests
- feat(client): spawn a task that listen to storage changes and build the
  resulting commiment state diff for each block
- dev(StarknetRPC): log error received from node before mapping to
  InternalServerError
- fix: change 'nonce too high' to log in debug instead of info
- chore: update deps, vm ressource fee cost are now FixedU128, and stored in an
  hashmap
- ci: change jobs order in the workflow
- ci: run integrations tests in the same runner as build
- ci: replace ci cache with rust-cache
- fix(transactions): remove `nonce` field from InvokeV0 tx
- feat(transactions): don't enforce ordering in validate_unsigned for invokeV0
- test(pallet): add function to get braavos hash
- fix: event commitment documentation typo
- ci: added testing key generation in the ci
- fix(starknet-rpc-test): init one request client per runtime
- test: validate Nonce for unsigned user txs
- fix: fixed declare V0 placeholder with the hash of an empty list of felts
- feat(cli): `run` is the by default command when running the `madara` bin
- refacto(cli): `run` and `setup` commands are defined in their own files
- refacto(cli): `run.testnet` argument removed in favor of the substrate native
  `chain` arg
- feat(cli): `run.fetch_chain_spec` argument removed in favor of the substrate
  native `chain` arg
- feat(cli): `setup` require a source file, either from an url or a path on the
  local filesystem
- chore(cli): use `Url`, `Path` and `PathBuf` types rather than `String`
- refacto(cli): moved the pallet/chain_spec/utils methods to the node crate
- feat(cli): `madara_path` arg has been remove, we use the substrate native
  `base_path` arg instead
- feat(cli): sharingan chain specs are loaded during the compilation, not
  downloaded from github
- refacto(pallet/starknet): `GenesisLoader` refactored as `GenesisData` + a
  `base_path` field
- feat(cli): for `run` param `--dev` now imply `--tmp`, as it is in substrate
- test(starknet-rpc-test): run all tests against a single madara node
- fix(service): confusing message when node starts (output the actual sealing
  method being used)
- refactor(sealing): how the sealing mode is passed into runtime
- feat(sealing): finalization for instant sealing
- test(starknet-js-test): run basic starknetjs compatibility tests again the
  madara node
- feat(cache-option): add an option to enable aggressive caching in command-line
  parameters

## v0.4.0

- chore: release v0.4.0
- feat: better management of custom configurations for genesis assets
- feat: use actual vm resource costs
- fix: add setup and run for rpc tests
- fix: fix clap for run command
- fix: add `madara_path` flag for setup command
- fix: add official references to configs files
- fix: cargo update and `main` branch prettier fix
- fix: fix sharingan chain spec
- fix: update madara infra to main branch
- fix: update `Cargo.lock`
- fix: rpc test failing
- refactor: exported chain id constant in mp-chain-id crate and added one for
  SN_MAIN
- ci: disable pr close workflow
- ci: add ci verification for detecting genesis changes and config hashes
- test: add e2e test for `estimate_fee`

## v0.3.0

- chore: release v0.3.0
- chore: big transaction type refactoring
- chore: split `primitives` crates into multiple smaller crates
- chore: improve logging about transaction when nonce is too high
- chore: add real class hash values for genesis config
- fix: use specific commit for avail and celestia
- fix: change dep of rustdoc on push
- fix: initial_gas set to max_fee and fixed fee not being charged when max_fee=0
- fix: correct value of compiled_class_hash in RPCTransaction
- fix: std feature import in transactions crate
- fix: replace all calls to `transmute` by calls `from_raw_parts`
- fix: estimate_fee should make sure all transaction have a version being
  2^128 + 1 or 2^128+2 depending on the tx type
- feat: modify the hash_bytes functions in `poseidon` and `pedersen` for dynamic
  data length
- feat: print development accounts at node startup
- feat: unification of the DA interface
- feat: bump starknet-core to 0.6.0 and remove InvokeV0
- feat: use resolver 2 for cargo in the workspace
- feat: impl tx execution and verification as traits
- perf: reduce the amount of data stored in the runtime and use the Substrate
  block to as source of data in the client
- perf: use perfect hash function in calculate_l1_gas_by_vm_usage
- build: restructure code for rust latest version
- build: bump rustc nightly version to 1.74 date
- buid: add rust-analyzer to toolchain components
- ci: scope cache by branch and add cache cleanup
- ci: increase threshold for codecov to 1%
- test: add `starknet-rpc-test` crate to the workspace
- test: add test to check tx signed by OZ account can be signed with Argent pk
- buid: add rust-analyzer to toolchain components
- ci: increase threshold for codecov to 1%
- replace all calls to `transmute` by calls `from_raw_parts`
- big transaction type refactoring
- impl tx execution and verification as traits
- reduce the amount of data stored in the runtime and use the Substrate block to
  as source of data in the client
- perf: use perfect hash function in calculate_l1_gas_by_vm_usage
- chore: add tests for tx hashing
- split `primitives` crates into multiple smaller crates
- fix: std feature import in transactions crate
- chore: improve logging about transaction when nonce is too high
- fix: rpc tests and background node run
- test: add tests for simulate tx offset
- test: add tests for tx hashing
- fix: bring back messages in transaction receipts
- feat: starknet os program output primitive

## v0.2.0

- add-contributors: `0xAsten`, `m-kus`, `joaopereira12`, `kasteph`
- ci: add verification if build-spec is working
- ci: added wasm to test
- ci: disable benchmark for pushes and pr's
- ci: fix docker and binaries build
- ci: don't enforce changelog on PR's with label `dependencies`
- doc: added translation of madara beast article.md to portuguese and russian
- doc: app chain template added in README
- fix: RPC getClassAt cairo legacy program code encoding
- fix: build-spec not working by setting the madara-path always and fetching
  relevant files
- fix: events are emitted in correct sequential order
- fix: expected event idx in continuation tokens in test responses
- fix: update RPC URL to use localhost instead of 0.0.0.0 in hurl.config file
- fix: update the default port for running Madara locally in getting-started.md
  file from 9933 to 9944.
- fix: replace the 0 initial gas value with u128::MAX because view call
  entrypoints were failing
- chore: remove global state root
- chore: cairo-contracts compilation scripts & docs are updated, cairo_0
  contracts recompiled
- chore: rebase of core deps and 0.12.1

## v0.1.0

- ci: rm codespell task and rm .codespellignore
- feat: refactor flags on tests
- feat: fetch config files from gh repo
- refactor: remove config files from the code
- ci: stop closing stale issues
- ci: reactivate changelog enforcement
- cli: change dev flag behaviour and created alias for base and madara path
- configs: fix genesis.json refs to link the config folder
- ci: downgraded windows runner to windows-latest
- ci: added windows binaries build and upload the binaries to the release page
- ci: add `CHANGELOG.md` and enforce it is edited for each PR on `main`
- fix: removed `madara_runtime` as a dependency in the client crates and make
  errors more expressive
- fix: state root bug fix where the tree was stored in runtime _before_ being
  committed
- feat: add a `genesis_loader` for the node and mocking
- feat: add `madara_tsukuyomi` as a submodule
- branding: use new logo in the README
- dev: Get the block status from the actual block in get_block_with_tx_hashes
- fix: l1-l2 messaging
- dev : clean contracts and compiled files
