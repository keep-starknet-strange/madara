import axios from "axios";
import { performance } from "perf_hooks";
import * as dotenv from "dotenv";
dotenv.config();

const REMOTE_RPC_URL = process.env.REMOTE_RPC;
const LOCAL_RPC_URL = process.env.LOCAL_RPC;

const requestData = (method: string) => ({
  id: 1,
  jsonrpc: "2.0",
  method: method,
  params: [
    {
      block_number: 49,
    },
  ],
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

const requestDataForMethod = (method: string, params: any[]) => ({
  id: 1,
  jsonrpc: "2.0",
  method: method,
  params: params,
});

async function benchmarkMethod(method: string, params: any[]) {
  console.log(`\x1b[34mBenchmarking method: ${method}\x1b[0m`);

  console.log(`\x1b[36mMaking request to Alchemy RPC...\x1b[0m`);
  const startAlchemy = performance.now();
  const alchemyResponse = await axios.post(
    REMOTE_RPC_URL,
    requestDataForMethod(method, params),
  );
  const endAlchemy = performance.now();

  console.log(`\x1b[36mMaking request to Local RPC...\x1b[0m`);
  const startLocal = performance.now();
  const localResponse = await axios.post(
    LOCAL_RPC_URL,
    requestDataForMethod(method, params),
  );
  const endLocal = performance.now();

  console.log(`Alchemy RPC time for ${method}: ${endAlchemy - startAlchemy}ms`);
  console.log(`Local RPC time for ${method}: ${endLocal - startLocal}ms`);

  const differences = compareObjects(alchemyResponse.data, localResponse.data);
  console.log(differences);
}

(async () => {
  await benchmarkMethod("starknet_getBlockWithTxHashes", [
    { block_number: 49 },
  ]);
  await benchmarkMethod("starknet_getBlockWithTxs", [{ block_number: 49 }]);
  // await benchmarkMethod('starknet_getStateUpdate', ["latest"]);
  // await benchmarkMethod('starknet_getStorageAt', [
  //   "0x044e5b3f0471a26bc749ffa1d8dd8e43640e05f1b33cf05cef6adee6f5b1b4cf",
  //   "0x0000000000000000000000000000000000000000000000000000000000000001",
  //   "latest"
  // ]);
  await benchmarkMethod("starknet_getTransactionByHash", [
    "0x0070b0a4765370b0b0bf5525ae98c9e89d51525dcf93de914dd0b8e3fdb14d6e",
  ]);
  // await benchmarkMethod('starknet_getTransactionByBlockIdAndIndex', ["0x03b6581f3222ff1f79c0e9959462aef03bd464999e998292772a0c51da53f9b1"]);
  await benchmarkMethod("starknet_getTransactionReceipt", [
    "0x0070b0a4765370b0b0bf5525ae98c9e89d51525dcf93de914dd0b8e3fdb14d6e",
  ]);
})();
