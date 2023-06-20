const { ApiPromise, WsProvider } = require("@polkadot/api");
const fs = require("fs");
const os = require("os");
const { parseArgs } = require("util");

const BLOCK_TIME = 6; // in seconds

function hostSpec() {
  // Retrieve the CPU information
  const cpuCount = os.cpus().length;
  const cpu = os.cpus()[0];
  const cpuModel = cpu.model;
  const cpuSpeed = cpu.speed;

  // Retrieve the total memory in bytes
  const totalMemory = os.totalmem();

  // Retrieve the operating system platform
  const platform = os.platform();

  // Retrieve the operating system release
  const release = os.release();

  // Retrieve the architecture of the machine
  const architecture = os.arch();

  return `CPU Count: ${cpuCount}\nCPU Model: ${cpuModel}\nCPU Speed (MHz): ${cpuSpeed}\nTotal Memory: ${
    totalMemory / 1e9
  } GB\nPlatform: ${platform}\nRelease: ${release}\nArchitecture: ${architecture}`;
}

async function main() {
  const {
    values: { type },
  } = parseArgs({
    options: {
      type: {
        type: "string",
      },
    },
  });

  const erc20 = "erc20";
  const erc721 = "erc721";

  if (type !== erc20 && type !== erc721) {
    throw new Error(
      "Please provide a type with --type flag, e.g. --type erc20 or --type erc721, current type is: " +
        type
    );
  }

  const fileName = type == erc20 ? "metrics_erc20.json" : "metrics_erc721.json";

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
    `reports/${fileName}`,
    JSON.stringify([
      {
        name:
          type === erc20
            ? "Average Extrinsics per block"
            : "Average Extrinsics per block (ERC721 mints)",
        unit: "extrinsics/block",
        value: avgExtrinsicsPerBlock,
        extra: hostSpec(),
      },
      {
        name: type === erc20 ? "Average TPS" : "Average TPS (ERC721 mints)",
        unit: "tps",
        value: avgTps,
        extra: hostSpec(),
      },
    ])
  );

  console.log(`Benchmark running on:\n${hostSpec()}`);
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
