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

cargo build --release

if [ "$DA_LAYER" = "ethereum" ]; then
    echo "Ethereum DA Test:"
    # TODO: do we want to add zaun as submodule
    git clone --recurse-submodules https://github.com/keep-starknet-strange/zaun.git target/zaun 2> /dev/null
    ./target/zaun/scripts/sn-base-dev.sh target target/zaun 2> /dev/null

    echo -e "\t anvil logs -> target/anvil.log"
    echo -e "\t to kill anvil -> ./target/zaun/scripts/sn-base-kill.sh target"
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
elif [ "$DA_LAYER" = "avail" ]; then
    echo "init avail stuff"
fi

./target/release/madara --dev --da-layer=$DA_LAYER
