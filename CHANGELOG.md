# Madara Changelog

## Next release

- dev(compilation): add incremental compilation

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
- fix: Ensure transaction checks are compatible with starknet-rs

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
