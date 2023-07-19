#!/bin/bash


echo "////////////////////////////////////////GET_TRANSACTION_RECEIPT//////////////////////////////////////////////"


# Faire la première requête curl
response1=$(curl -s -X POST \
     -H "Content-Type: application/json" \
     --data '{"jsonrpc":"2.0","method":"starknet_getTransactionReceipt","params":["0x585a43c4df382ff17bc6079c2747dc017fe71d0aca9c34cac890b835e90786e"],"id":0}' \
     http://127.0.0.1:9956/ | jq '.')

# Faire la deuxième requête curl
response2=$(curl -s -X POST \
     -H "Content-Type: application/json" \
     --data '{"jsonrpc":"2.0","method":"starknet_getTransactionReceipt","params":["0x585a43c4df382ff17bc6079c2747dc017fe71d0aca9c34cac890b835e90786e"],"id":0}' \
     https://starknet-mainnet.g.alchemy.com/v2/hnj_DGevqpyoyeoEs9Vfx-6qSTHOnaIu | jq '.')

# Extraire les champs pour comparaison
fields=$(echo "$response1" | jq -r '.result | keys[]')

# Comparer les champs et afficher les différences
diff_output=""

for field in $fields; do
    value1=$(echo "$response1" | jq -r ".result.$field")
    value2=$(echo "$response2" | jq -r ".result.$field")
    diff_output=$(diff -u <(echo "$value1") <(echo "$value2") | colordiff)
    if [ -z "$diff_output" ]; then
    	echo ""
    else
    	echo "$field"
    	echo "$diff_output"
    fi
done


echo "////////////////////////////////////////////GET_TRANSACTION_BY_HASH//////////////////////////////////////////"

# Faire la première requête curl
response1=$(curl -s -X POST \
     -H "Content-Type: application/json" \
     --data '{"jsonrpc":"2.0","method":"starknet_getTransactionByHash","params":["0x585a43c4df382ff17bc6079c2747dc017fe71d0aca9c34cac890b835e90786e"],"id":0}' \
     http://127.0.0.1:9956/ | jq '.')

# Faire la deuxième requête curl
response2=$(curl -s -X POST \
     -H "Content-Type: application/json" \
     --data '{"jsonrpc":"2.0","method":"starknet_getTransactionByHash","params":["0x585a43c4df382ff17bc6079c2747dc017fe71d0aca9c34cac890b835e90786e"],"id":0}' \
     https://starknet-mainnet.g.alchemy.com/v2/hnj_DGevqpyoyeoEs9Vfx-6qSTHOnaIu | jq '.')

# Extraire les champs pour comparaison
fields=$(echo "$response1" | jq -r '.result | keys[]')

# Comparer les champs et afficher les différences
diff_output=""

for field in $fields; do
    value1=$(echo "$response1" | jq -r ".result.$field")
    value2=$(echo "$response2" | jq -r ".result.$field")
    diff_output=$(diff -u <(echo "$value1") <(echo "$value2") | colordiff)
    if [ -z "$diff_output" ]; then
        echo ""
    else
        echo "$field"
        echo "$diff_output"
    fi
done


echo "////////////////////////////////////////////CHAIN ID//////////////////////////////////////////"

# Faire la première requête curl
response1=$(curl -s -X POST \
     -H "Content-Type: application/json" \
     --data '{"jsonrpc":"2.0","method":"starknet_chainId","id":0}' \
     http://127.0.0.1:9956/ | jq '.')

# Faire la deuxième requête curl
response2=$(curl -s -X POST \
     -H "Content-Type: application/json" \
     --data '{"jsonrpc":"2.0","method":"starknet_chainId","id":0}' \
     https://starknet-mainnet.g.alchemy.com/v2/hnj_DGevqpyoyeoEs9Vfx-6qSTHOnaIu | jq '.')


# Comparer les champs et afficher les différences
diff_output=""

diff_output=$(diff -u <(echo "$response1") <(echo "$response2") | colordiff)
echo "$diff_output"

echo "////////////////////////////////////////////GET_TRANSACTION_BY_BLOCKID_AND_INDEX//////////////////////////////////////////"

# Faire la première requête curl
response1=$(curl -s -X POST \
     -H "Content-Type: application/json" \
     --data '{"jsonrpc":"2.0","method":"starknet_getTransactionByBlockIdAndIndex","params":[{"block_number": 9100}, 4],"id":0}' \
     http://127.0.0.1:9956/ | jq '.')

# Faire la deuxième requête curl
response2=$(curl -s -X POST \
     -H "Content-Type: application/json" \
     --data '{"jsonrpc":"2.0","method":"starknet_getTransactionByBlockIdAndIndex","params":[{"block_number": 9100}, 4],"id":0}' \
     https://starknet-mainnet.g.alchemy.com/v2/hnj_DGevqpyoyeoEs9Vfx-6qSTHOnaIu | jq '.')

# Extraire les champs pour comparaison
fields=$(echo "$response1" | jq -r '.result | keys[]')

# Comparer les champs et afficher les différences
diff_output=""

for field in $fields; do
    value1=$(echo "$response1" | jq -r ".result.$field")
    value2=$(echo "$response2" | jq -r ".result.$field")
    diff_output=$(diff -u <(echo "$value1") <(echo "$value2") | colordiff)
    if [ -z "$diff_output" ]; then
        echo ""
    else
        echo "$field"
        echo "$diff_output"
    fi
done
