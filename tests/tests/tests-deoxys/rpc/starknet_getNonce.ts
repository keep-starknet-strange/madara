import axios from "axios";
import * as dotenv from "dotenv";
dotenv.config();

const REMOTE_RPC_URL = process.env.REMOTE_RPC!;
const LOCAL_RPC_URL = process.env.DEOXYS_RPC!;
// THIS ONE IS REALLY COST INTENSIVE
const START_BLOCK = 1000;
const END_BLOCK = 1466;

const requestDataForMethod = (method: string, params: any[]) => ({
  id: 1,
  jsonrpc: "2.0",
  method: method,
  params: params,
});

const compareObjects = (obj1: any, obj2: any, path: string = ""): string => {
  let differences = "";
  for (const key in obj1) {
    const currentPath = path ? `${path}.${key}` : key;
    if (typeof obj1[key] === "object" && obj1[key] !== null) {
      differences += compareObjects(obj1[key], obj2[key], currentPath);
    } else if (obj1[key] !== obj2[key]) {
      differences += `\x1b[31mDIFFERENCE at ${currentPath}: ${obj1[key]} (Alchemy) vs ${obj2[key]} (Local)\x1b[0m\n`;
    } else {
      differences += `\x1b[32mMATCH at ${currentPath}: ${obj1[key]}\x1b[0m\n`;
    }
  }
  return differences;
};

async function benchmarkBlockAndNonces(blockNumber: number): Promise<string> {
  console.log(`\x1b[34mBenchmarking block number: ${blockNumber}\x1b[0m`);
  
  const blockResponse = await axios.post(REMOTE_RPC_URL, requestDataForMethod("starknet_getBlockWithTxHashes", [
    { block_number: blockNumber }
  ]));

  let overallDifferences = "";
  const contractAddresses: string[] = blockResponse.data.result.transactions || [];
  
  for (const contractAddress of contractAddresses) {
    console.log(`\x1b[36mTesting nonce for contract: ${contractAddress}\x1b[0m`); // Logging the contract being tested
    const differences = await benchmarkMethod("starknet_getNonce", [{ block_number: blockNumber }, contractAddress]);
    if (differences.includes("\x1b[31mDIFFERENCE")) {
      console.log(`\x1b[31mNonce for contract ${contractAddress} has differences.\x1b[0m`);
    } else {
      console.log(`\x1b[32mNonce for contract ${contractAddress} matches.\x1b[0m`);
    }
    overallDifferences += differences;
  }

  return overallDifferences;
}

async function benchmarkMethod(method: string, params: any[]): Promise<string> {
  const alchemyResponse = await axios.post(REMOTE_RPC_URL, requestDataForMethod(method, params));
  const localResponse = await axios.post(LOCAL_RPC_URL, requestDataForMethod(method, params));
  return compareObjects(alchemyResponse.data, localResponse.data);
}

async function checkDifferencesInBlocks() {
  const blocksWithDifferences: number[] = [];

  for (let blockNumber = START_BLOCK; blockNumber < END_BLOCK; blockNumber++) {
    const differences = await benchmarkBlockAndNonces(blockNumber);
    if (differences.includes("\x1b[31mDIFFERENCE")) {
      blocksWithDifferences.push(blockNumber);
      console.log(`\x1b[31mBlock ${blockNumber} has differences.\x1b[0m`);
    } else {
      console.log(`\x1b[32mBlock ${blockNumber} matches.\x1b[0m`);
    }
  }

  if (blocksWithDifferences.length === 0) {
    console.log("\x1b[32mAll blocks match!\x1b[0m");
  } else {
    console.log("\x1b[31mDifferences found in blocks:\x1b[0m", JSON.stringify(blocksWithDifferences));
  }
}

(async () => {
  await checkDifferencesInBlocks();
})();
