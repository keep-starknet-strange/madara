#!/bin/bash

# [ethereum, celestia, avail]
DA_LAYER=$1

if [ "$DA_LAYER" = "ethereum" ]; then
    echo "Killing Anvil:"
    ./zaun/scripts/sn-base-kill.sh target
elif [ "$DA_LAYER" = "celestia" ]; then
    # TODO: Kill Celestia
    echo "Killing Celestia:"
elif [ "$DA_LAYER" = "avail" ]; then
    # TODO: Kill Avail
    echo "Killing Avail:"
fi
