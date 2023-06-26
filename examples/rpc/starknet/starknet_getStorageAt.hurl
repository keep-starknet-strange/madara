POST {{RPC_URL}}
Content-Type: application/json
{
    "jsonrpc": "2.0",
    "method": "starknet_getStorageAt",
    "params": [
        "0x0000000000000000000000000000000000000000000000000000000000001111",
        "0x02900ac0f31b4cd8101abb46a91021989eb7c9f6e2a5417186e476f08429efce",
        "latest"
    ],
    "id":1
}

HTTP 200
[Asserts]
jsonpath "$.error" not exists
jsonpath "$.result" exists
