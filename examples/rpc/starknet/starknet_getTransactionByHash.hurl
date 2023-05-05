POST http://0.0.0.0:9933
Content-Type: application/json
{
    "jsonrpc": "2.0",
    "method": "starknet_getTransactionByHash",
    "params": [
        "0x02e2a98c1731ece2691edfbb4ed9b057182cec569735bd89825f17e3b342583a"
    ],
    "id":1
}

HTTP 200
[Asserts]
jsonpath "$.error" not exists
jsonpath "$.result.transaction_hash" matches /^0x[A-Fa-f0-9]+$/
