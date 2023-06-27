POST https://sharingan.madara.wtf
Content-Type: application/json
{
    "jsonrpc": "2.0",
    "method": "starknet_getTransactionReceipt",
    "params": [
        "0x02f87e43a2027080262a6c5495bb989ec595ae67ab3f474caf8aee51a3589018"
    ],
    "id":1
}

HTTP 200
[Asserts]
jsonpath "$.error" not exists
jsonpath "$.result.transaction_hash" matches /^0x[A-Fa-f0-9]+$/