import axios from "axios";
import { performance } from "perf_hooks";
import * as dotenv from "dotenv";
dotenv.config();

const REMOTE_RPC_URL = process.env.REMOTE_RPC;
const LOCAL_RPC_URL = process.env.LOCAL_RPC;
const BLOCK_NUMBER = 51;
const START_BLOCK = 0;
const END_BLOCK = 151;

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

async function benchmarkMethod(method: string, params: any): Promise<string> {
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

  for (let blockNumber = 51; blockNumber < END_BLOCK; blockNumber++) {
    const differences = await benchmarkMethod("starknet_getEvents", {
      filter: {
        from_block: {
          block_number: blockNumber,
        },
        to_block: {
          block_number: blockNumber + 40,
        },
        keys: [
          [
            "0x06321563e7fc5a96d97c45639f30e91a059cef769812766c01f28fd5e15f19bb",
          ],
        ],
        chunk_size: 10,
      },
    });

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
  const singleCallDifferences = await benchmarkMethod("starknet_getEvents", {
    invoke_transaction: {
      type: "INVOKE",
      max_fee: "0xDEADB",
      version: "0x1",
      nonce: "0x0",
      signature: ["0x0", "0x0"],
      sender_address:
        "0x0000000000000000000000000000000000000000000000000000000000000001",
      calldata: [
        "0x0000000000000000000000000000000000000000000000000000000000001111",
        "0x36fa6de2810d05c3e1a0ebe23f60b9c2f4629bbead09e5a9704e1c5632630d5",
        "0x0",
      ],
    },
  });
  console.log(singleCallDifferences);

  //await checkDifferencesInBlocks();
})();
