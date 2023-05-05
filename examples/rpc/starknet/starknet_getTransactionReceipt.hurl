POST http://0.0.0.0:9933
Content-Type: application/json
{
    "jsonrpc": "2.0",
    "method": "starknet_getTransactionReceipt",
    "params": [
        "0x4c1672e824b5cd7477fca31ee3ab5a1058534ed1820bb27abc976c2e6095151"
    ],
    "id":1
}

HTTP 200
[Asserts]
jsonpath "$.error" not exists
jsonpath "$.result.transaction_hash" matches /^0x[A-Fa-f0-9]+$/
