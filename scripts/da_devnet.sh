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
    echo "Celestia DA Test:"
elif [ "$DA_LAYER" = "avail" ]; then
    echo "init avail stuff"
fi

./target/release/madara --dev --da-layer=$DA_LAYER
