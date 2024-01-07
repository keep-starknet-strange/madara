#!/bin/bash

# [ethereum, celestia, avail]
DA_LAYER=$1
MADARA_PATH=$2

if [ -z $MADARA_PATH ]; then
    MADARA_PATH="$HOME/.madara"
fi

if [ ! -f "$MADARA_PATH/da-config.json" ]; then
    echo "{}" > $MADARA_PATH/da-config.json
fi

if [ "$DA_LAYER" = "ethereum" ]; then
    echo "Ethereum DA Test:"
    ./zaun/scripts/sn-base-dev.sh target zaun 2> /dev/null

    echo -e "\t anvil logs -> target/anvil.log"
    echo -e "\t to kill anvil -> ./zaun/scripts/sn-base-kill.sh target"
elif [ "$DA_LAYER" = "celestia" ]; then
    if ! command -v celestia > /dev/null
    then
        echo "please install jq"
        exit 1
    fi
    rm target/celestia.log
    echo "Celestia DA Test:"
    
    celestia light start --core.ip consensus-full-arabica-9.celestia-arabica.com --p2p.network arabica 1>target/celestia.log --keyring.accname da-test 2>&1 &

    sleep 3

    CELESTIA_JWT=$(celestia light auth admin --p2p.network arabica-9)
    jq -r '.auth_token = "'$CELESTIA_JWT'"' $MADARA_PATH/da-config.json > $MADARA_PATH/da-config-tmp.json 
    mv $MADARA_PATH/da-config-tmp.json $MADARA_PATH/da-config.json 

    export CELESTIA_NODE_AUTH_TOKEN=$CELESTIA_JWT
    echo "celestia account balance $(celestia rpc state Balance | jq '.result.amount')"
elif [ "$DA_LAYER" = "avail" ]; then
    echo "Avail DA Test:"
    
    if [ ! -d "avail" ]; then
        echo "Cloning Avail repository"
        git clone https://github.com/availproject/avail 2> /dev/null
    fi
    
    # Navigate to cloned directory
    cd avail
    
    # Check if data-avail binary exists
    if [ ! -f "./target/release/data-avail" ]; then
        # Build the project
        echo "Building repository"
        cargo build --release 2> /dev/null
    fi

    # End avail if we exit
    trap 'pkill -f "data-avail"' EXIT
    
    # Run data-avail and redirect logs and errors
    echo "Launching Avail"
    ./target/release/data-avail --dev --tmp --rpc-port 9934 --ws-port 9945 --port 30334 1>../target/avail.log 2> /dev/null &

    # Navigate back to original directory
    cd ..

    sleep 5
fi
