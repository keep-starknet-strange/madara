#!/usr/bin/env sh

# Function to kill background processes
cleanup() {
    echo "Stopping background processes and removing files..."
    find . -type f -name "validator_output.txt" -delete
    find . -type f -name "node_output.txt" -delete
    kill "$validator_pid" "$node_pid" 2>/dev/null
}

# Trap EXIT signal to ensure cleanup runs on script exit
trap cleanup EXIT

# build release
cargo build --release

# copy configs from local
./target/release/madara setup --from-local ./configs/ --chain=local --base-path /tmp/alice

# copy configs over from local
./target/release/madara setup --from-local ./configs/ --chain=local --base-path /tmp/node1

# purge validator chain
./target/release/madara purge-chain --base-path /tmp/alice --chain local -y

# purge node chain
./target/release/madara purge-chain --base-path /tmp/node1 --chain local -y

# run validator in background instance
./target/release/madara \
--base-path /tmp/alice \
--chain local \
--alice \
--node-key 0000000000000000000000000000000000000000000000000000000000000001 \
--validator  > validator_output.txt 2>&1 &
validator_pid=$!


# run node in another background instance
 ./target/release/madara \
--chain local \
--bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp \
--base-path /tmp/node1 \
--rpc-port 9946  > node_output.txt 2>&1 &
node_pid=$!

# Give some time for the services to start and produce output
sleep 20

# look for the 1 peer message
specific_message="1 peers"

# Check if the message is there in the file and fail if not
if ! grep -q "$specific_message" validator_output.txt; then
    echo "Node failed to connect to validator"
    exit 1
fi

echo "Validator successfully connected."
