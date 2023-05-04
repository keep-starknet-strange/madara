POST http://0.0.0.0:9933
Content-Type: application/json
{
    "jsonrpc":"2.0",
    "method":"starknet_call",
    "params": {
        "request": {
            "contract_address": "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
            "entry_point_selector": "0x361458367e696363fbcc70777d07ebbd2394e89fd0adcaf147faccd1d294d60",
            "calldata": []
        },
        "block_number": 33482
    },
    "id":1
}

HTTP 200
[Asserts]
jsonpath "$.error" not exists
jsonpath "$.result" exists
