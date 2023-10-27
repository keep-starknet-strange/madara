import axios from "axios";
import * as dotenv from "dotenv";

dotenv.config();

const REMOTE_RPC_URL = process.env.REMOTE_RPC;
const LOCAL_RPC_URL = process.env.LOCAL_RPC;
const START_BLOCK = 0;
const END_BLOCK = 1800;

const requestDataForMethod = (method: string, params: any[] = []) => ({
  id: 1,
  jsonrpc: "2.0",
  method: method,
  params: params,
});

const compareObjects = (
  remoteData: any,
  localData: any,
  path: string = "",
): string => {
  let differences = "";

  for (const key in remoteData) {
    const currentPath = path ? `${path}.${key}` : key;

    if (!localData.hasOwnProperty(key)) {
      differences += `DIFFERENCE in Local at ${currentPath}: ${remoteData[key]}\n`;
      continue;
    }

    if (typeof remoteData[key] === "object" && remoteData[key] !== null) {
      differences += compareObjects(
        remoteData[key],
        localData[key],
        currentPath,
      );
    } else if (remoteData[key] !== localData[key]) {
      differences += `DIFFERENCE at ${currentPath}: ${remoteData[key]} (Remote) vs ${localData[key]} (Local)\n`;
    }
  }

  return differences;
};

async function benchmarkMethod(
  method: string,
  params: any[] = [],
): Promise<string> {
  const remoteResponse = await axios.post(
    REMOTE_RPC_URL,
    requestDataForMethod(method, params),
  );
  const localResponse = await axios.post(
    LOCAL_RPC_URL,
    requestDataForMethod(method, params),
  );

  return compareObjects(remoteResponse.data, localResponse.data);
}

async function checkDifferencesInBlocks() {
  const blocksWithDifferences: number[] = [];

  for (let blockNumber = START_BLOCK; blockNumber < END_BLOCK; blockNumber++) {
    const differences = await benchmarkMethod("starknet_blockNumber");

    if (differences.includes("DIFFERENCE")) {
      blocksWithDifferences.push(blockNumber);
    }
  }

  if (blocksWithDifferences.length === 0) {
    console.log("All blocks match!");
  } else {
    console.log("Differences found in blocks:", blocksWithDifferences);
  }
}

(async () => {
  const singleCallDifferences = await benchmarkMethod("starknet_blockNumber");
  console.log(singleCallDifferences);

  await checkDifferencesInBlocks();
})();
