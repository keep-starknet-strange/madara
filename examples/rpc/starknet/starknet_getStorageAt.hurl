POST http://0.0.0.0:9933
Content-Type: application/json
{
    "jsonrpc": "2.0",
    "method": "starknet_getStorageAt",
    "params": [
        "0x0000000000000000000000000000000000000000000000000000000000001111",
        "0x391a88f3badec8650b4d8356e18655269ee975e58e0060aa076396ce0b1dccb",
        "latest"
    ],
    "id":1
}

HTTP 200
[Asserts]
jsonpath "$.error" not exists
jsonpath "$.result" exists
