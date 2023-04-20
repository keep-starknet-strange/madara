#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
set -e

exec ./target/release/madara --dev --ws-external --execution native --pool-limit=100000 --pool-kbytes=500000
