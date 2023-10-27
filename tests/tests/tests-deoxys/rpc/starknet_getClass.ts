import axios from "axios";
import { performance } from "perf_hooks";
import * as dotenv from "dotenv";
dotenv.config();

const REMOTE_RPC_URL = process.env.REMOTE_RPC;
const LOCAL_RPC_URL = process.env.DEOXYS_RPC;
const BLOCK_NUMBER = 4000;
const START_BLOCK = 500;
const END_BLOCK = 1000;

const requestDataForMethod = (method: string, params: any[]) => ({
  id: 1,
  jsonrpc: "2.0",
  method: method,
  params: params,
});

const compareObjects = (obj1: any, obj2: any, alchemyData: any, localData: any, path: string = ""): string => {
  let differences = "";

  for (const key in obj1) {
    const currentPath = path ? `${path}.${key}` : key;

    if (obj1[key] === undefined) {
      differences += `\x1b[31mDIFFERENCE in Alchemy at ${currentPath}: ${JSON.stringify(obj2[key])} | Full Alchemy Data: ${JSON.stringify(alchemyData)}\x1b[0m\n`;
      continue;
    }

    if (obj2[key] === undefined) {
      differences += `\x1b[31mDIFFERENCE in Local at ${currentPath}: ${JSON.stringify(obj1[key])} | Full Local Data: ${JSON.stringify(localData)}\x1b[0m\n`;
      continue;
    }

    if (typeof obj1[key] === "object" && obj1[key] !== null) {
      differences += compareObjects(obj1[key], obj2[key], alchemyData, localData, currentPath);
    } else if (obj1[key] !== obj2[key]) {
      differences += `\x1b[31mDIFFERENCE at ${currentPath}: ${JSON.stringify(obj1[key])} (Alchemy) vs ${JSON.stringify(obj2[key])} (Local)\x1b[0m\n`;
    } else {
      differences += `\x1b[32mMATCH at ${currentPath}: ${JSON.stringify(obj1[key])}\x1b[0m\n`;
    }
  }

  return differences;
};

async function benchmarkMethod(method: string, params: any[]): Promise<string> {
  console.log(`\x1b[34m[Benchmark] Starting ${method} for params: ${JSON.stringify(params)}\x1b[0m`);
  
  const startTime = performance.now();
  
  let alchemyResponse;
  let localResponse;

  try {
    alchemyResponse = await axios.post(REMOTE_RPC_URL, requestDataForMethod(method, params));
  } catch (error) {
    console.log(`\x1b[31mError fetching data from Alchemy: ${error.message}\x1b[0m`);
  }

  try {
    localResponse = await axios.post(LOCAL_RPC_URL, requestDataForMethod(method, params));
  } catch (error) {
    console.log(`\x1b[31mError fetching data from Local: ${error.message}\x1b[0m`);
  }

  const endTime = performance.now();
  console.log(`\x1b[34m[Benchmark] ${method} completed in ${(endTime - startTime).toFixed(2)}ms\x1b[0m`);

  return compareObjects(alchemyResponse?.data, localResponse?.data, alchemyResponse?.data, localResponse?.data);
}

async function checkDifferencesInBlocks() {
  const blocksWithDifferences: number[] = [];

  for (let blockNumber = START_BLOCK; blockNumber < END_BLOCK; blockNumber++) {
    console.log(`\x1b[34mChecking block number: ${blockNumber}\x1b[0m`);
    const differences = await benchmarkMethod("starknet_getClass", [
      { block_number: blockNumber },
      "0x01cb96b938da26c060d5fd807eef8b580c49490926393a5eeb408a89f84b9b46",
    ]);

    if (differences.includes("\x1b[31mDIFFERENCE")) {
      blocksWithDifferences.push(blockNumber);
    } else (differences.includes("\x1b[31MATCH"))
      console.log("✅");
  }

  if (blocksWithDifferences.length === 0) {
    console.log("\x1b[32mAll blocks match!\x1b[0m");
  } else {
    console.log("\x1b[31mDifferences found in blocks:\x1b[0m", blocksWithDifferences);
  }
}

(async () => {
  console.log("\x1b[34mStarting script...\x1b[0m");
  
  const singleCallDifferences = await benchmarkMethod("starknet_getClass", [
    { block_number: BLOCK_NUMBER },
    "0x03131fa018d520a037686ce3efddeab8f28895662f019ca3ca18a626650f7d1e",
  ]);
  
  console.log(singleCallDifferences);
  await checkDifferencesInBlocks();
  
  console.log("\x1b[34mScript completed!\x1b[0m");
})();
