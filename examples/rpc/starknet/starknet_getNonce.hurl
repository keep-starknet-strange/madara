POST http://0.0.0.0:9933
Content-Type: application/json
{
    "jsonrpc": "2.0",
    "method": "starknet_getNonce",
    "params": [
        "0x010b6c96d364cf182964fbd4a3438a5ae84cab990770c07994f9cb99fd26f6dc"
    ],
    "id":1
}

HTTP 200
[Asserts]
jsonpath "$.error" not exists
jsonpath "$.result" matches /^[0-9]+$/
