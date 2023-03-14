# Run validator node A
echo "Starting validator node A"
cargo run --release -- --chain=local --force-authoring --rpc-cors=all --alice --port 30334 --ws-port 9944 --rpc-port 9934 --base-path /tmp/kaioshin/validator-node-a --ws-external &

# Run validator node B
echo "Starting validator node B"
cargo run --release -- --chain=local --force-authoring --rpc-cors=all --bob --port 31344 --ws-port 10944 --rpc-port 10934 --base-path /tmp/kaioshin/validator-node-b --ws-external &