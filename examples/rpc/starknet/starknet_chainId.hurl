POST http://0.0.0.0:9944
Content-Type: application/json
{
    "jsonrpc": "2.0",
    "method": "starknet_chainId",
    "params": [],
    "id": 1
}

HTTP 200
[Asserts]
jsonpath "$.error" not exists
jsonpath "$.result" == "0x534e5f474f45524c49"
