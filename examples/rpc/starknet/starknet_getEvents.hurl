POST {{RPC_URL}}
Content-Type: application/json
{
  "jsonrpc": "2.0",
  "method": "starknet_getEvents",
  "params": {
    "filter": {
      "from_block": "latest",
      "to_block": "latest",
      "keys": [["0x99cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9"]],
      "chunk_size": 10
    }
  },
  "id": 1
}

HTTP 200
[Asserts]
jsonpath "$.error" not exists
jsonpath "$.result" exists
