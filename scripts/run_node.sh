#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
set -e

while [[ $# -gt 0 ]]; do
  case $1 in
    --with-state-root)
      with_state_root=true
      ;;
    *)
      echo "Unknown argument: $1"
      exit 1
      ;;
  esac
  shift
done

# Check if the --with-state-root flag is provided
if [[ $with_state_root ]]; then
    echo "Running with state root..."
    cargo build --release --features madara-state-root
    exec ../target/release/madara --dev --tmp --rpc-external --execution native --pool-limit=100000 --pool-kbytes=500000 --rpc-methods=unsafe --rpc-cors=all --in-peers=0 --out-peers=1 --no-telemetry
else
    echo "Running without state root..."
    cargo build --release
    exec ../target/release/madara --dev --tmp --rpc-external --execution native --pool-limit=100000 --pool-kbytes=500000 --rpc-methods=unsafe --rpc-cors=all --in-peers=0 --out-peers=1 --no-telemetry
fi

