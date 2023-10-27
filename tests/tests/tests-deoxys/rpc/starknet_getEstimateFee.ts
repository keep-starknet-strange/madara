import axios from "axios";
import { performance } from "perf_hooks";
import * as dotenv from "dotenv";
dotenv.config();

const REMOTE_RPC_URL = process.env.REMOTE_RPC;
const LOCAL_RPC_URL = process.env.LOCAL_RPC;
const LATEST = "latest";
const START_BLOCK = 0;
const END_BLOCK = 100;

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
    // Handle cases where a key is not present in one of the objects or is undefined
    if (obj1[key] === undefined) {
      differences += `\x1b[31mDIFFERENCE in Alchemy at ${currentPath}: ${obj2[key]}\x1b[0m\n`;
      continue;
    }

    if (obj2[key] === undefined) {
      differences += `\x1b[31mDIFFERENCE in Local at ${currentPath}: ${obj1[key]}\x1b[0m\n`;
      continue;
    }
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

async function benchmarkMethod(method: string, params: any[]): Promise<string> {
  console.log(
    `\x1b[34mBenchmarking method: ${method}\x1b[0m for params: ${JSON.stringify(
      params,
    )}`,
  );

  const alchemyResponse = await axios.post(
    REMOTE_RPC_URL,
    requestDataForMethod(method, params),
  );
  const localResponse = await axios.post(
    LOCAL_RPC_URL,
    requestDataForMethod(method, params),
  );

  return compareObjects(alchemyResponse.data, localResponse.data);
}

async function checkDifferencesInBlocks() {
  const blocksWithDifferences: number[] = [];

  for (let blockNumber = START_BLOCK; blockNumber < END_BLOCK; blockNumber++) {
    const differences = await benchmarkMethod("starknet_getEstimateFee", [
      {
        block_id: {
          block_number: 2803,
        },
        broadcasted_transaction: {
          type: "INVOKE",
          nonce: "0x3",
          max_fee: "0x12C72866EFA9B",
          version: "0x0",
          signature: [
            "0x10E400D046147777C2AC5645024E1EE81C86D90B52D76AB8A8125E5F49612F9",
            "0x0ADB92739205B4626FEFB533B38D0071EB018E6FF096C98C17A6826B536817B",
          ],
          contract_address:
            "0x0019fcae2482de8fb3afaf8d4b219449bec93a5928f02f58eef645cc071767f4",
          calldata: [
            "0x0000000000000000000000000000000000000000000000000000000000000001",
            "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
            "0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000003",
            "0x0000000000000000000000000000000000000000000000000000000000000003",
            "0x04681402a7ab16c41f7e5d091f32fe9b78de096e0bd5962ce5bd7aaa4a441f64",
            "0x000000000000000000000000000000000000000000000000001d41f6331e6800",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
          ],
          entry_point_selector:
            "0x015d40a3d6ca2ac30f4031e42be28da9b056fef9bb7357ac5e85627ee876e5ad",
        },
      },
    ]);

    if (differences.includes("\x1b[31mDIFFERENCE")) {
      blocksWithDifferences.push(blockNumber);
    }
  }

  if (blocksWithDifferences.length === 0) {
    console.log("\x1b[32mAll blocks match!\x1b[0m");
  } else {
    console.log(
      "\x1b[31mDifferences found in blocks:\x1b[0m",
      blocksWithDifferences,
    );
  }
}

(async () => {
  const singleCallDifferences = await benchmarkMethod(
    "starknet_getEstimateFee",
    [
      {
        block_id: {
          block_number: 2803,
        },
        broadcasted_transaction: {
          type: "INVOKE",
          nonce: "0x3",
          max_fee: "0x12C72866EFA9B",
          version: "0x0",
          signature: [
            "0x10E400D046147777C2AC5645024E1EE81C86D90B52D76AB8A8125E5F49612F9",
            "0x0ADB92739205B4626FEFB533B38D0071EB018E6FF096C98C17A6826B536817B",
          ],
          contract_address:
            "0x0019fcae2482de8fb3afaf8d4b219449bec93a5928f02f58eef645cc071767f4",
          calldata: [
            "0x0000000000000000000000000000000000000000000000000000000000000001",
            "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
            "0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000003",
            "0x0000000000000000000000000000000000000000000000000000000000000003",
            "0x04681402a7ab16c41f7e5d091f32fe9b78de096e0bd5962ce5bd7aaa4a441f64",
            "0x000000000000000000000000000000000000000000000000001d41f6331e6800",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
          ],
          entry_point_selector:
            "0x015d40a3d6ca2ac30f4031e42be28da9b056fef9bb7357ac5e85627ee876e5ad",
        },
      },
    ],
  );
  console.log(singleCallDifferences);

  //await checkDifferencesInBlocks();
})();
