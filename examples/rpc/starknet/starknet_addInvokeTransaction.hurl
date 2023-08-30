POST {{RPC_URL}}
Content-Type: application/json
{
    "jsonrpc": "2.0",
    "method": "starknet_addInvokeTransaction",
    "params": {
        "invoke_transaction": {
            "type": "INVOKE",
            "max_fee":"0xDEADB",
            "version":"0x1",
            "nonce":"0x1",
            "signature":[
                "0x0",
                "0x0"
            ],
            "sender_address":"0x0000000000000000000000000000000000000000000000000000000000000001",
            "calldata": [
                "0x0000000000000000000000000000000000000000000000000000000000001111",
                "0x36fa6de2810d05c3e1a0ebe23f60b9c2f4629bbead09e5a9704e1c5632630d5",
                "0x0"
            ]
        }
    },
    "id": 1
}

HTTP 200
