import axios from "axios";
import { performance } from "perf_hooks";
import * as dotenv from "dotenv";
dotenv.config();

const REMOTE_RPC_URL = process.env.REMOTE_RPC;
const LOCAL_RPC_URL = process.env.LOCAL_RPC;
const START_BLOCK = 0;
const END_BLOCK = 100;
const TRANSACTIONS = [
  "0x160d07b065887fec1f898405d12874b742c553d8dfc52e6dc5a8667b4d05e63",
  "0x79c6fe04996648a8d0620094d35d8929b0d8f8ac1007b4ee65bdb0fd778f530",
  "0x64a7595144011b6a19822b47b0df6c5b5dad6ab57c73b2d6b173552649994e1",
  "0x217f0f39e756dc73dea242ec3df697a95298b8fe68946787cc26da22a057014",
  "0xc5e13997c6b48b482b684b25085f3a5affa6720a93b071af83241295ab4a29",
  "0x14ad4cbb90b23dee77b70a8a358e77df7426e2b3b8ef80dbefe1a13b21784cd",
  "0x3c66742bb3318090e2783b783e1003d364e3defa0746fb959dda61bac58aea9",
];

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
  const blocksWithDifferences: string[] = [];

  for (const tx of TRANSACTIONS) {
    const differences = await benchmarkMethod(
      "starknet_getTransactionReceipt",
      [tx],
    );

    if (differences.includes("\x1b[31mDIFFERENCE")) {
      blocksWithDifferences.push(tx);
    }
  }

  if (blocksWithDifferences.length === 0) {
    console.log("\x1b[32mAll blocks match!\x1b[0m");
  } else {
    console.log(
      "\x1b[31mDifferences found in transactions:\x1b[0m",
      blocksWithDifferences,
    );
  }
}

(async () => {
  const singleCallDifferences = await benchmarkMethod(
    "starknet_getTransactionReceipt",
    ["0x3c28efbc632f0ece25dc30aa2c5035f9aa907e43a42103609b9aded29fde516"],
  );
  console.log(singleCallDifferences);

  await checkDifferencesInBlocks();
})();
