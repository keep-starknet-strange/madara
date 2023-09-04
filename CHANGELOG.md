# Madara Changelog

## Next release

- feat: use resolver 2 for cargo in the workspace
- upgrade: restructure code for rust latest version
- upgrade: bump rustc nightly version to 1.74 date
- ci: add verification if build-spec is working
- bug: fix build-spec not working by setting the madara-path always and fetching
  relevant files
- fix: RPC getClassAt cairo legacy program code encoding
- ci: added wasm to test
- docs: added translation of madara beast article.md to portuguese
- ci: disable benchmark for pushes and pr's
- ci: fix docker and binaries build
- ci: don't enforce changelog on PR's with label `dependencies`
- feat: rebase of core deps and 0.12.1

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
- fix: events are emitted in correct sequential order
- fix: expected event idx in cotinuation tokens in test responses
- chore: cairo-contracts compilation scripts & docs are updated, cairo_0
  contracts recompiled
- add-contributors: `0xAsten`, `m-kus`, `joaopereira12`
- fix: update RPC URL to use localhost instead of 0.0.0.0 in hurl.config file
- fix: update the default port for running Madara locally in getting-started.md
  file from 9933 to 9944.
- dev: replace the 0 initial gas value with u128::MAX because view call
  entrypoints were failing
