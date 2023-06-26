POST {{RPC_URL}}
Content-Type: application/json
{
    "jsonrpc": "2.0",
    "method": "starknet_getTransactionByBlockIdAndIndex",
    "params": [
        "latest",
        0
    ],
    "id": 1
}

HTTP 200
[Asserts]
jsonpath "$.error" not exists
jsonpath "$.result.transaction_hash" matches /^0x[A-Fa-f0-9]+$/
