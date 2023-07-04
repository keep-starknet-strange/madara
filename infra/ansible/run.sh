#!/bin/sh

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
if [ ! -f "$SCRIPT_DIR/nodes.ips" ]; then
    echo "nodes ips not found, try to run terraform first"
    exit 1
fi
if [ ! -f "$SCRIPT_DIR/madara.pem" ]; then
    echo "madara.pem not found, try to run terraform first"
    exit 1
fi

chmod 600 $SCRIPT_DIR/madara.pem
# export string '["1.1.1.1", "2.2.2.2", ...]' to NODE0, NODE1...
ARR=$(cat $SCRIPT_DIR/nodes.ips | sed -E 's/\"|\[|\]//g' | sed 's/,/ /g')
i=0
for ip in $ARR; do
    export NODE$i=$ip
    i=$((i+1))
done

ansible-playbook -i $SCRIPT_DIR/inventories/hosts.ini $SCRIPT_DIR/playbooks/run-node-dev.yml
