POST {{RPC_URL}}
Content-Type: application/json
{
    "jsonrpc":"2.0",
    "method":"madara_predeployedAccounts",
    "params": [],
    "id":1
}

HTTP 200
[Asserts]
jsonpath "$.error" not exists
jsonpath "$.result" exists
jsonpath "$.result[0].contract_address" == "1"
jsonpath "$.result[1].contract_address" == "2"
jsonpath "$.result[2].contract_address" == "3"
jsonpath "$.result[3].contract_address" == "4"
