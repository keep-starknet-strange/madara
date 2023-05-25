const { FEE_TOKEN_ADDRESS } = require("../tests/build/tests/constants");
const starknet = require("starknet");
const fs = require("fs");

const BLOCK_TIME = 6; // in seconds

main()
  .catch((err) => {
    console.error(err);
    process.exit(-1);
  })
  .then(() => process.exit(0));
main();
function timeout(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
async function main() {
  for (let i = 0; i < 100000; i++) {
    const res = await fetch("http://127.0.0.1:9933", {
      method: "POST",
      body: `{"method":"starknet_addInvokeTransaction","jsonrpc":"2.0","params":{"invoke_transaction":{"sender_address":"0x0000000000000000000000000000000000000000000000000000000000000001","calldata":["${FEE_TOKEN_ADDRESS}","0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e","0x3","0x01176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8","0x2","0x0"],"type":"INVOKE","max_fee":"0x989680","version":"0x1","signature":["0x011", "0x011"],"nonce":"0x${i.toString(
        16
      )}"}},"id":0}`,
      headers: { "Content-Type": "application/json" },
    });
  }
  const provider = new starknet.RpcProvider({
    nodeUrl: "http://127.0.0.1:9933",
  });

  // We spam madara with ERC20 transactions
  let totalExtrinsics = 0;
  // Wait for some more transactions to be processed.
  await timeout(10000);
  // Count the processed transactions of the last 4 blocks.

  const blockNumber = await provider.getBlockNumber();

  for (let i = blockNumber - 3; i <= blockNumber; i++) {
    totalExtrinsics += await provider.getTransactionCount(blockNumber);
  }
  // Compute the average number of tx / block
  const avgExtrinsicsPerBlock = totalExtrinsics / 4;
  // Compute the average TPS.
  const avgTps = avgExtrinsicsPerBlock / BLOCK_TIME;

  // Save avgExtrinsicsPerBlock to file report.json
  fs.writeFileSync(
    "report.json",
    JSON.stringify({ avgExtrinsicsPerBlock, avgTps })
  );

  console.log(
    `Average TPS : ${avgTps} (avgExtrinsicsPerBlock: ${avgExtrinsicsPerBlock})`
  );
}
