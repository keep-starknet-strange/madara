POST http://0.0.0.0:9933
Content-Type: application/json
{
    "jsonrpc": "2.0",
    "method": "starknet_addInvokeTransaction",
    "params": {
        "invoke_transaction": {
            "type": "INVOKE",
            "max_fee":"0x4f388496839",
            "version":"0x1",
            "nonce":"0x2",
            "signature":[
                "0x7dd3a55d94a0de6f3d6c104d7e6c88ec719a82f4e2bbc12587c8c187584d3d5",
                "0x71456dded17015d1234779889d78f3e7c763ddcfd2662b19e7843c7542614f8"
            ],
            "sender_address":"0x23371b227eaecd8e8920cd429d2cd0f3fee6abaacca08d3ab82a7cdd",
            "calldata": ["0x1","0x677bb1cdc050e8d63855e8743ab6e09179138def390676cc03c484daf112ba1","0x362398bec32bc0ebb411203221a35a0301193a96f317ebe5e40be9f60d15320","0x0","0x1","0x1","0x2b","0x0"]
        }
    },
    "id": 1
}

HTTP 200
