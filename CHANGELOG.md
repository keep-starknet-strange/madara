# Madara Changelog

## Next release

- ci: disable pr close workflow
- ci: add ci verification for detecting genesis changes and config hashes
- feat: better management of custom configurations for genesis assets
- fix: fix sharingan chain spec
- fix: update madara infra to main branch
- fix: update `Cargo.lock`

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
