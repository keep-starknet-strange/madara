import { WsProvider, ApiPromise } from "@polkadot/api";
import { Keyring } from "@polkadot/keyring";
import { u8aToHex } from "@polkadot/util";
import erc20Json from "../../ressources/erc20.json" assert { type: "json" };

interface EntryPointType {
  [name: string]: number;
}

const ENTRYPOINTS: EntryPointType = {
  CONSTRUCTOR: 0,
  EXTERNAL: 1,
  L1_HANDLER: 2,
};

interface EntryPoint {
  offset: string;
  selector: string;
}

// Convert Uint8Array to hex string
const programBytes = u8aToHex(
  new TextEncoder().encode(JSON.stringify(erc20Json.program))
);

const entryPointsMap = new Map<number, EntryPoint[]>(
  Object.entries(erc20Json.entry_points_by_type).map(([k, v]) => [
    ENTRYPOINTS[k],
    v,
  ])
);
// Convert Map to Uint8Array
const entryPointsBytes = u8aToHex(
  new TextEncoder().encode(JSON.stringify(entryPointsMap))
);

async function init() {
  const wsProvider = new WsProvider();
  const api = await ApiPromise.create({ provider: wsProvider });
  await api.isReady;

  console.log(api.genesisHash.toHex());

  const keyring = new Keyring({ type: "sr25519" });
  const user = keyring.addFromUri(`//Alice`);

  return { api, user };
}

async function declare(
  api: ApiPromise,
  user: any,
  contractAddress: string,
  tokenClassHash: string
) {
  const tx_declare = {
    version: 1, // version of the transaction
    hash: "", // leave empty for now, will be filled in by the runtime
    signature: [], // leave empty for now, will be filled in when signing the transaction
    events: [], // empty vector for now, will be filled in by the runtime
    sender_address: contractAddress, // address of the sender contract
    nonce: 0, // nonce of the transaction
    callEntrypoint: {
      // call entrypoint
      classHash: tokenClassHash, // class hash of the contract
      entrypointSelector: null, // function selector of the transfer function
      calldata: [], // empty vector for now, will be filled in by the runtime
      storageAddress: contractAddress,
      callerAddress: contractAddress,
    },
    contractClass: {
      program: programBytes,
      entryPointsByType: "0x",
    },
  };

  const extrisinc_declare = api.tx.starknet.addDeclareTransaction(tx_declare);

  try {
    const signedTxDeclare = await extrisinc_declare.signAsync(user, {
      nonce: -1,
    });
    const resultDeclare = await signedTxDeclare.send();
    console.log(resultDeclare.toHuman());
  } catch (error) {
    console.error("Eror while declaring : ", error);
  }
}

async function deploy(
  api: ApiPromise,
  user: any,
  contractAddress: string,
  tokenClassHash: string
) {
  // Deploy contract
  let tx_deploy = {
    version: 1, // version of the transaction
    hash: "", // leave empty for now, will be filled in by the runtime
    signature: [], // leave empty for now, will be filled in when signing the transaction
    events: [], // empty vector for now, will be filled in by the runtime
    sender_address: contractAddress, // address of the sender contract
    nonce: 1, // nonce of the transaction
    callEntrypoint: {
      // call entrypoint
      classHash: tokenClassHash, // class hash of the contract
      entrypointSelector: null, // function selector of the transfer function
      calldata: [
        "0x0000000000000000000000000000000000000000000000000000000000001111",
        "0x0169f135eddda5ab51886052d777a57f2ea9c162d713691b5e04a6d4ed71d47f",
        "0x0000000000000000000000000000000000000000000000000000000000000005",
        tokenClassHash,
        "0x0000000000000000000000000000000000000000000000000000000000000001",
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
      ],
      storageAddress: contractAddress,
      callerAddress: contractAddress,
    },
    contractClass: null,
  };

  try {
    const extrisinc_deploy = api.tx.starknet.addInvokeTransaction(tx_deploy);
    const signedTxDeploy = await extrisinc_deploy.signAsync(user, {
      nonce: -1,
    });
    const resultDeploy = await signedTxDeploy.send();
    console.log(resultDeploy.toHuman()?.toString());
  } catch (error) {
    console.error("Eror while deploying : ", error);
  }
}

async function e2e_erc20_test() {
  const { api, user } = await init();

  const contractAddress =
    "0x0000000000000000000000000000000000000000000000000000000000000101";
  const tokenClassHash =
    "0x025ec026985a3bf9d0cc1fe17326b245bfdc3ff89b8fde106242a3ea56c5a918";

  await declare(api, user, contractAddress, tokenClassHash);

  await deploy(api, user, contractAddress, tokenClassHash);
}

void e2e_erc20_test();
