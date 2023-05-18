POST http://0.0.0.0:9933
Content-Type: application/json
{
    "jsonrpc":"2.0",
    "method":"starknet_call",
    "params": [
        {
            "contract_address": "0x040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d01",
            "entry_point_selector": "0x2e4263afad30923c891518314c3c95dbe830a16874e8abc5777a9a20b54c76e",
            "calldata": ["0x0000000000000000000000000000000000000000000000000000000000000002"]
        },
        "latest"
    ],
    "id":1
}

HTTP 200
[Asserts]
jsonpath "$.error" not exists
jsonpath "$.result" exists
