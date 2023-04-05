#!/bin/bash

trap "trap - TERM && kill -- -$$" INT TERM EXIT

if [[ ! -f "../target/release/madara" ]];
then
  echo 'Missing madara binary. Please run cargo build --release'
  exit 1;
fi

# Fail if any command fails

echo "Installing Packages"
npm ci

echo "Starting madara node"
../target/release/madara --tmp --chain=local --rpc-port=9933 &> /tmp/node-start.log &
PID=$!

echo "Waiting node...(5s)"
sleep 1
( tail -f -n0 /tmp/node-start.log & ) | grep -q 'new connection'

echo "Generating types...(10s)"
sleep 1
npm run load:meta
npm run load:meta:local
npm run generate:defs
npm run generate:meta
npm run postgenerate

kill $PID
echo "Done :)"
