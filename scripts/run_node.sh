#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
set -e

log_level=${1:-info}
exec ../target/release/madara --dev --rpc-external --execution native --pool-limit=100000 --pool-kbytes=500000 --rpc-methods=unsafe --rpc-cors=all --in-peers=0 --out-peers=1 --no-telemetry -l $log_level
