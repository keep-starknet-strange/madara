POST {{RPC_URL}}
Content-Type: application/json
{
    "jsonrpc": "2.0",
    "method": "starknet_estimateFee",
    "params": {
        "request": {
            "type": "INVOKE",
            "max_fee":"0x0",
            "version":"0x100000000000000000000000000000001",
            "nonce":"0x0",
            "signature":[],
            "sender_address":"0x0000000000000000000000000000000000000000000000000000000000000001",
            "calldata": [
                "0x0000000000000000000000000000000000000000000000000000000000001111",
                "0x13769b427d1425aae02dd51cc2487375670e57d638de17ae5197d2ac1092fc0",
                "0x0"
            ]
        },
        "block_id": "latest"
    },
    "id": 1
}

HTTP 200