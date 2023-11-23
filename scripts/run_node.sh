#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
set -e

cargo build --release
exec ../target/release/madara --dev --tmp --rpc-external --pool-limit=100000 --pool-kbytes=500000 --rpc-methods=unsafe --rpc-cors=all --in-peers=0 --out-peers=1 --no-telemetry
