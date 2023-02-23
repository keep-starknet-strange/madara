#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
set -e

exec ./target/release/kaioshin --dev --ws-external
