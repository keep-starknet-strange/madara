const starknet = require("starknet");
const {
  SIGNER_PRIVATE,
  ACCOUNT_CONTRACT,
  FEE_TOKEN_ADDRESS,
  SIGNER_PUBLIC,
} = require("../tests/build/tests/constants");
main();
async function main() {
  for (let i = 0; i < 100; i++) {
    console.log(i)
    let res = await fetch("http://127.0.0.1:9933", {
      method: "POST",
      body: `{"method":"starknet_addInvokeTransaction","jsonrpc":"2.0","params":{"invoke_transaction":{"sender_address":"0x0000000000000000000000000000000000000000000000000000000000000001","calldata":["${FEE_TOKEN_ADDRESS}","0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e","0x3","0x01176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8","0x2","0x0"],"type":"INVOKE","max_fee":"0x989680","version":"0x1","signature":["0x01", "0x01"],"nonce":"0x${i.toString(16)}"}},"id":0}`,
      headers: { "Content-Type": "application/json" },
    });
    console.log(await res.json())
  }
}
async function rpcTransfer(account, nonce, recipient, transferAmount, maxFee) {
  return await account.execute(
    {
      contractAddress: FEE_TOKEN_ADDRESS,
      entrypoint: "transfer",
      calldata: [recipient, transferAmount, "0x0"],
    },
    undefined,
    {
      nonce,
      maxFee: maxFee ?? "123456",
    }
  );
}
