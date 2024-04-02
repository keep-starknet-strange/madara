.PHONY: check

fix:
	npx prettier --write .
	cargo fmt
	taplo fmt --config ./taplo/taplo.toml
	cargo clippy --workspace --tests --no-deps --fix -- -D warnings

check: prt fmt clip tap

prt:
	npx prettier --check .

fmt:
	cargo fmt -- --check

clip:
	cargo clippy --workspace --tests --no-deps -- -D warnings

tap:
	taplo fmt --config ./taplo/taplo.toml --check

test:
	cargo test

