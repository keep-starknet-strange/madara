POST {{RPC_URL}}
Content-Type: application/json
{
    "jsonrpc": "2.0",
    "method": "starknet_pendingTransactions",
    "params": [],
    "id":1
}

HTTP 200
[Asserts]
jsonpath "$.error" not exists
jsonpath "$.result" isCollection
