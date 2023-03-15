#!/bin/bash
ROOT_DIR="/tmp/kaioshin"
LOG_DIR="$ROOT_DIR/logs"

# Build the command to run the validator node
CMD_VALIDATOR="cargo run --release -- --chain=local --validator --force-authoring --rpc-cors=all --ws-external --rpc-external --rpc-methods=unsafe"
CMD_FULLNODE="cargo run --release -- --chain=local --rpc-cors=all --ws-external --rpc-external --rpc-methods=unsafe"

function initialize(){
    # Create the root directory
    mkdir -p $ROOT_DIR

    # Create the log directory
    mkdir -p $LOG_DIR
}

# Display a menu to the user
function menu(){
    echo "1. Start nodes"
    echo "2. Stop nodes"
    echo "3. Exit"
    echo "4. Cleanup"
    echo "5. Show logs"
    echo -n "Enter your choice: "
    read choice
    case $choice in
        1) start_nodes ;;
        2) stop_nodes ;;
        3) exit 0 ;;
        4) cleanup ;;
        5) show_logs ;;
        *) echo "Invalid choice" ;;
    esac
}

function start_nodes(){
    # Run validator node A
    start_validator_node "validator-node-a" 30334 9944 9934 9615 "alice"

    # Run validator node B
    start_validator_node "validator-node-b" 30335 9945 9935 9715 "bob"

    # Run light client node C
    start_full_node "full-node-c" 30336 9946 9936 "0000000000000000000000000000000000000000000000000000000000000001"
}

function stop_nodes(){
    echo "Stopping nodes"
    killall kaioshin
}

function start_validator_node(){
    name=$1
    port=$2
    ws_port=$3
    rpc_port=$4
    prometheus_port=$5
    key_alias=$6
    base_path=$ROOT_DIR/$name
    log_file=$LOG_DIR/$name.log

    # Create the validator node base path directory
    mkdir -p $base_path

    echo "Starting $name"
    run_cmd="$CMD_VALIDATOR --$key_alias --port $port --ws-port $ws_port --rpc-port $rpc_port --prometheus-port $prometheus_port --base-path $base_path &> $log_file &"
    echo "Running: $run_cmd"
    eval $run_cmd
}

function start_full_node(){
    name=$1
    port=$2
    ws_port=$3
    rpc_port=$4
    node_key=$5
    base_path=$ROOT_DIR/$name
    log_file=$LOG_DIR/$name.log

    # Create the validator node base path directory
    mkdir -p $base_path

    echo "Starting $name"
    run_cmd="$CMD_FULLNODE --node-key $node_key --port $port --ws-port $ws_port --rpc-port $rpc_port --base-path $base_path &> $log_file &"
    echo "Running: $run_cmd"
    eval $run_cmd
}


function show_logs(){
    #Ask the user to select the node to show logs for
    echo "Select the node to show logs for"
    echo "1. Validator node A"
    echo "2. Validator node B"
    echo "3. Light client node C"
    echo -n "Enter your choice: "
    read choice
    case $choice in
        1) show_node_logs "validator-node-a" ;;
        2) show_node_logs "validator-node-b" ;;
        3) show_node_logs "lightclient-node-c" ;;
        *) echo "Invalid choice" ;;
    esac
}

# Show the logs for a node
function show_node_logs(){
    name=$1
    log_file=$LOG_DIR/$name.log
    echo "Showing logs for $name"
    tail -f $log_file
}

function cleanup {
    echo "Cleaning up"
    rm -rf $ROOT_DIR
}

# Initialize the script
initialize

# Show the menu
menu
