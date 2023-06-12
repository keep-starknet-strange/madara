POST {{RPC_URL}}
Content-Type: application/json
{
    "jsonrpc": "2.0",
    "method": "starknet_getTransactionByHash",
    "params": [
        "0x3024c929c1bec3a872441d8ad6a8e5b0c5b931d2bf75dc5dbad2b93c1c4dec0"
    ],
    "id":1
}

HTTP 200
