import axios from "axios";
import { performance } from "perf_hooks";
import * as dotenv from "dotenv";
dotenv.config();

const REMOTE_RPC_URL = process.env.REMOTE_RPC!;
const LOCAL_RPC_URL = process.env.DEOXYS_RPC!;
const BLOCK_NUMBER = 1058;
const START_BLOCK = 0;
const END_BLOCK = 1700;

const requestDataForMethod = (method: string, params: any[]) => ({
  id: 1,
  jsonrpc: "2.0",
  method: method,
  params: params,
});

const compareTransactionCount = (remoteCount: number, localCount: number, blockNumber: number): string => {
  if (remoteCount !== localCount) {
    return `\x1b[31mDIFFERENCE at block ${blockNumber}: ${remoteCount} (Remote) vs ${localCount} (Local)\x1b[0m\n`;
  } else {
    return `\x1b[32mMATCH at block ${blockNumber}: ${remoteCount}\x1b[0m\n`;
  }
};

async function benchmarkTransactionCount(blockNumber: number): Promise<string> {
  console.log(`\x1b[34mBenchmarking transaction count for block: ${blockNumber}\x1b[0m`);

  const alchemyResponse = await axios.post(REMOTE_RPC_URL, requestDataForMethod("starknet_getBlockTransactionCount", [{ block_number: blockNumber }]));
  const localResponse = await axios.post(LOCAL_RPC_URL, requestDataForMethod("starknet_getBlockTransactionCount", [{ block_number: blockNumber }]));

  const differences = compareTransactionCount(alchemyResponse.data.result, localResponse.data.result, blockNumber);

  if (differences.includes("\x1b[31mDIFFERENCE")) {
    console.log(`\x1b[31mBlock ${blockNumber} has differences.\x1b[0m`);
  } else {
    console.log(`\x1b[32mBlock ${blockNumber} matches.\x1b[0m`);
  }

  return differences;
}

async function checkDifferencesInBlocks() {
  const blocksWithDifferences: number[] = [];

  for (let blockNumber = START_BLOCK; blockNumber < END_BLOCK; blockNumber++) {
    const differences = await benchmarkTransactionCount(blockNumber);

    if (differences.includes("\x1b[31mDIFFERENCE")) {
      blocksWithDifferences.push(blockNumber);
    }
  }

  if (blocksWithDifferences.length === 0) {
    console.log("\x1b[32mAll blocks match!\x1b[0m");
  } else {
    console.log("\x1b[31mDifferences found in blocks:\x1b[0m", JSON.stringify(blocksWithDifferences));
  }
}

(async () => {
  // Single block test
  const singleBlockDifferences = await benchmarkTransactionCount(BLOCK_NUMBER);
  console.log(singleBlockDifferences);

  // Loop through blocks
  await checkDifferencesInBlocks();
})();
