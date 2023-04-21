const { ApiPromise, WsProvider } = require("@polkadot/api");
const fs = require("fs");

const BLOCK_TIME = 6; // in seconds

async function main() {
  const wsProvider = new WsProvider("ws://localhost:9944");
  const api = await ApiPromise.create({ provider: wsProvider });

  const blockHash = await api.rpc.chain.getBlock();
  const blockNumber = blockHash.block.header.number;

  let totalExtrinsics = 0;

  for (let i = blockNumber.toNumber() - 9; i <= blockNumber.toNumber(); i++) {
    const hash = await api.rpc.chain.getBlockHash(i);
    const block = await api.rpc.chain.getBlock(hash);
    totalExtrinsics += block.block.extrinsics.length;
  }

  const avgExtrinsicsPerBlock = totalExtrinsics / 10;
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

main().catch((err) => {
  console.error(err);
  process.exit(-1);
}).then(() => process.exit(0));
