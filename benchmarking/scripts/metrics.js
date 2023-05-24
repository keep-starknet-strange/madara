const { ApiPromise, WsProvider } = require("@polkadot/api");
const fs = require("fs");

const BLOCK_TIME = 6; // in seconds

async function main() {
  const wsProvider = new WsProvider("ws://localhost:9944");
  const api = await ApiPromise.create({ provider: wsProvider });

  const blockHash = await api.rpc.chain.getBlock();
  const blockNumber = blockHash.block.header.number;
  // We spam madara with ERC20 transactions
  let totalExtrinsics = 0;
  // Wait for some more transactions to be processed.
  setTimeout(() => {}, 10000);
  // Count the processed transactions of the last 4 blocks.
  for (let i = blockNumber.toNumber() - 3; i <= blockNumber.toNumber(); i++) {
    const hash = await api.rpc.chain.getBlockHash(i);
    const block = await api.rpc.chain.getBlock(hash);
    totalExtrinsics += block.block.extrinsics.length;
  }
  // Compute the average number of tx / block
  const avgExtrinsicsPerBlock = totalExtrinsics / 4;
  // Compute the average TPS.
  const avgTps = avgExtrinsicsPerBlock / BLOCK_TIME;

  // Save avgExtrinsicsPerBlock to file reports/metrics.json
  fs.writeFileSync(
    "reports/metrics.json",
    JSON.stringify({ avgExtrinsicsPerBlock, avgTps })
  );

  console.log(
    `Average TPS : ${avgTps} (avgExtrinsicsPerBlock: ${avgExtrinsicsPerBlock})`
  );
}

main()
  .catch((err) => {
    console.error(err);
    process.exit(-1);
  })
  .then(() => process.exit(0));
