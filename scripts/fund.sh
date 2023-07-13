#! /usr/bin/env bash
set -e;
set -o pipefail;

destination_flag='0x1' # send to self by default
amount_flag='0x1' # send 1 wei by default

print_usage() {
  printf "Use -d to set the destination and -a to set the amount\n"
}

while getopts 'd:a:' flag; do
echo $flag, ${OPTARG}
  case "${flag}" in
    d) destination_flag="${OPTARG}" ;;
    a) amount_flag="${OPTARG}" ;;
    *) print_usage
       exit 1 ;;
  esac
done

function rpc_call() {
    printf "${1}"
     curl --request POST \
          --header 'Content-Type: application/json' \
          --data "${1}" \
          https://sharingan.madara.wtf
}

nonce=$(rpc_call '{
    "id": 1,
    "jsonrpc": "2.0",
    "method": "starknet_getNonce",
    "params": ["latest", "0x1"]
}' | jq -r '.result' | tr -d 'null\n')

rpc_call "{
    \"id\": 0,
    \"jsonrpc\": \"2.0\",
    \"method\": \"starknet_addInvokeTransaction\",
    \"params\": {
        \"invoke_transaction\": {
            \"version\": \"0x1\",
            \"max_fee\": \"0x12345\",
            \"signature\": [\"0x0\", \"0x0\"],
            \"nonce\": \"${nonce}\",
            \"sender_address\": \"0x1\",
            \"calldata\": [\"0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7\", 
                        \"0x83afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e\", 
                        \"${destination_flag}\", \"${amount_flag}\", \"0x0\"]
        }
    }
}"
