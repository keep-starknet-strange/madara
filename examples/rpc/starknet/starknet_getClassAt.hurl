POST {{RPC_URL}}
Content-Type: application/json
{
    "jsonrpc": "2.0",
    "method": "starknet_getClassAt",
    "params": [
        "latest",
        "0x040688250Ef0074B4c9e1057B19F9b62139ac28179c7d35e2daE5abAD909d558"
    ],
    "id":1
}

HTTP 200
[Asserts]
jsonpath "$.error" not exists
jsonpath "$.result" exists
