POST http://0.0.0.0:9933
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
jsonpath "$.result" == "23448594291968334"
