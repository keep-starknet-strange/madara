import axios from 'axios';
import { performance } from 'perf_hooks';

const ALCHEMY_RPC_URL = 'https://starknet-mainnet.g.alchemy.com/v2/hnj_DGevqpyoyeoEs9Vfx-6qSTHOnaIu';
const LOCAL_RPC_URL = 'http://localhost:9944';

const requestData = (method: string) => ({
  id: 1,
  jsonrpc: "2.0",
  method: method,
  params: [
    {
      "block_number": 49
    }
  ]
});

const compareObjects = (obj1: any, obj2: any, path: string = ''): string => {
  let differences = "";

  for (const key in obj1) {
    const currentPath = path ? `${path}.${key}` : key;
    if (typeof obj1[key] === 'object' && obj1[key] !== null) {
      differences += compareObjects(obj1[key], obj2[key], currentPath);
    } else if (obj1[key] !== obj2[key]) {
      differences += `\x1b[31mDIFFERENCE at ${currentPath}: ${obj1[key]} (Alchemy) vs ${obj2[key]} (Local)\x1b[0m\n`;
    } else {
      differences += `\x1b[32mMATCH at ${currentPath}: ${obj1[key]}\x1b[0m\n`;
    }
  }

  return differences;
}

async function benchmarkMethod(method: string) {
  console.log(`\x1b[34mBenchmarking method: ${method}\x1b[0m`);

  console.log(`\x1b[36mMaking request to Alchemy RPC...\x1b[0m`);
  const startAlchemy = performance.now();
  const alchemyResponse = await axios.post(ALCHEMY_RPC_URL, requestData(method));
  const endAlchemy = performance.now();

  console.log(`\x1b[36mMaking request to Local RPC...\x1b[0m`);
  const startLocal = performance.now();
  const localResponse = await axios.post(LOCAL_RPC_URL, requestData(method));
  const endLocal = performance.now();

  console.log(`Alchemy RPC time for ${method}: ${endAlchemy - startAlchemy}ms`);
  console.log(`Local RPC time for ${method}: ${endLocal - startLocal}ms`);

  const differences = compareObjects(alchemyResponse.data, localResponse.data);
  console.log(differences);
}

(async () => {
  await benchmarkMethod('starknet_getBlockWithTxHashes');
  await benchmarkMethod('starknet_getBlockWithTxs');
})();
