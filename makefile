.PHONY: check fix prt fmt clip tap test help

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

help:
	@echo "Available targets:"
	@echo "  all    - Run the default target (check)"
	@echo "  check  - Run all checks (prt, fmt, clip, tap)"
	@echo "  fix    - Fix code formatting and linting issues"
	@echo "  prt    - Check code formatting with Prettier"
	@echo "  fmt    - Check code formatting with rustfmt"
	@echo "  clip   - Run Clippy linter"
	@echo "  tap    - Check configuration file formatting with Taplo"
	@echo "  test   - Run tests"
