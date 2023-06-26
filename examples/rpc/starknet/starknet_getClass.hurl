POST {{RPC_URL}}
Content-Type: application/json
{
    "jsonrpc": "2.0",
    "method": "starknet_getClass",
    "params": [
        "latest",
        "0x00d0e183745e9dae3e4e78a8ffedcce0903fc4900beace4e0abf192d4c202da3"
    ],
    "id":1
}

HTTP 200
[Asserts]
jsonpath "$.error" not exists
jsonpath "$.result" exists
