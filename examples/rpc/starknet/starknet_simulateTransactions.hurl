POST {{RPC_URL}}
Content-Type: application/json
{
    "jsonrpc": "2.0",
    "method": "starknet_simulateTransactions",
    "params": {
        "block_id": "latest",
        "transactions": [{
            "type": "DEPLOY_ACCOUNT",
            "max_fee": "0xDEADB",
            "signature": [
                "0x0",
                "0x0"
            ],
            "version": "0x1",
            "nonce": "0x0",
            "class_hash": "0x06f0d6f6ae72e1a507ff4b65181291642889742dbf8f1a53e9ec1c595d01ba7d",
            "contract_address_salt": "0x0000000000000000000000000000000000000000000000000000000000001111",
            "constructor_calldata": [
            ]
        }, {
            "type": "INVOKE",
            "max_fee": "0xDEADB",
            "version": "0x1",
            "nonce": "0x0",
            "signature": [
                "0x0",
                "0x0"
            ],
            "sender_address": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "calldata": [
                '0x0000000000000000000000000000000000000000000000000000000000001111',
                '0x36fa6de2810d05c3e1a0ebe23f60b9c2f4629bbead09e5a9704e1c5632630d5',
                '0x0'
            ],
        }],
        "simulation_flags": [
        ]
    },
    "id": 1
}

HTTP 200
