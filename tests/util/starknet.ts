import "@keep-starknet-strange/madara-api-augment";
import { type ApiPromise } from "@polkadot/api";
import { type ApiTypes, type SubmittableExtrinsic } from "@polkadot/api/types";
import { type ISubmittableResult } from "@polkadot/types/types";
import { stringify, u8aWrapBytes } from "@polkadot/util";
import erc20Json from "../contracts/compiled/erc20.json";
import { numberToU832Bytes } from "./utils";
export async function sendTransactionNoValidation(
  transaction: SubmittableExtrinsic<"promise", ISubmittableResult>
): Promise<void> {
  await transaction.send();
}

export async function sendTransactionBatchNoValidation(
  api: ApiPromise,
  transactions: Array<SubmittableExtrinsic<"promise", ISubmittableResult>>
): Promise<void> {
  await api.tx.utility.batch(transactions).send();
}

export async function sendTransaction(
  api: ApiPromise,
  transaction: SubmittableExtrinsic<"promise", ISubmittableResult>
): Promise<string> {
  return await new Promise((resolve, reject) => {
    let unsubscribe;
    const SPAWNING_TIME = 500000;
    const timeout = setTimeout(() => {
      reject(new Error("Transaction timeout"));
    }, SPAWNING_TIME);
    let transaction_success_event = false;
    let block_hash;

    transaction
      .send(async ({ events = [], status, dispatchError }) => {
        console.log(`Current status is ${status.type}`);

        // status would still be set, but in the case of error we can shortcut
        // to just check it (so an error would indicate InBlock or Finalized)
        if (dispatchError) {
          if (dispatchError.isModule) {
            // for module errors, we have the section indexed, lookup
            const decoded = api.registry.findMetaError(dispatchError.asModule);
            const { docs, name, section } = decoded;

            reject(Error(`${section}.${name}: ${docs.join(" ")}`));
          } else {
            // Other, CannotLookup, BadOrigin, no extra info
            reject(Error(dispatchError.toString()));
          }
        }

        if (status.isInBlock) {
          block_hash = status.asInBlock.toHex();
          console.log("Included at block hash", block_hash);
          console.log("Events:");

          events.forEach(({ event: { data, method, section }, phase }) => {
            console.log(
              "\t",
              phase.toString(),
              `: ${section}.${method}`,
              data.toString()
            );

            if (section == "system" && method == "ExtrinsicSuccess") {
              transaction_success_event = true;
            }
          });
        }

        if (transaction_success_event) {
          if (unsubscribe) {
            unsubscribe();
          }

          clearTimeout(timeout);
          resolve(block_hash);
        }
      })
      .then((unsub) => {
        unsubscribe = unsub;
      })
      .catch((error) => {
        console.error(error);
        reject(error);
      });
  });
}

export function declare(
  api: ApiPromise,
  contractAddress: string,
  tokenClassHash: string
): SubmittableExtrinsic<ApiTypes, ISubmittableResult> {
  const tx_declare = {
    version: 1, // version of the transaction
    signature: [], // leave empty for now, will be filled in when signing the transaction
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
      program: u8aWrapBytes(Buffer.from(stringify(erc20Json.program))),
      entryPointsByType: u8aWrapBytes(
        Buffer.from(stringify(erc20Json.entry_points_by_type))
      ),
    },
  };

  const extrisinc_declare = api.tx.starknet.declare(tx_declare);

  return extrisinc_declare;
}

export function deploy(
  api: ApiPromise,
  contractAddress: string,
  tokenClassHash: string
): SubmittableExtrinsic<ApiTypes, ISubmittableResult> {
  // Compute contract address
  // const deployedContractAddress = hash.calculateContractAddressFromHash(
  //   2,
  //   tokenClassHash,
  //   [],
  //   0
  // );

  // Deploy contract
  const tx_deploy = {
    version: 1, // version of the transaction
    signature: [], // leave empty for now, will be filled in when signing the transaction
    sender_address: contractAddress, // address of the sender contract
    nonce: 0, // nonce of the transaction
    account_class_hash: tokenClassHash, // class hash of the contract
    calldata: [
      "0x0000000000000000000000000000000000000000000000000000000000001111",
      "0x0169f135eddda5ab51886052d777a57f2ea9c162d713691b5e04a6d4ed71d47f",
      "0x000000000000000000000000000000000000000000000000000000000000000A", // Calldata len
      "0x0000000000000000000000000000000000000000000000000000000000010000", // Class hash
      "0x0000000000000000000000000000000000000000000000000000000000000001", // Contract address salt
      "0x0000000000000000000000000000000000000000000000000000000000000006", // Constructor_calldata_len
      "0x000000000000000000000000000000000000000000000000000000000000000A", // Name
      "0x0000000000000000000000000000000000000000000000000000000000000001", // Symbol
      "0x0000000000000000000000000000000000000000000000000000000000000002", // Decimals
      "0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", // Initial supply low
      "0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", // Initial supply high
      "0x0000000000000000000000000000000000000000000000000000000000001111", // recipient
      "0x0000000000000000000000000000000000000000000000000000000000000001", // deploy from zero
    ],
    max_fee:
      "0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
  };

  const extrisinc_deploy = api.tx.starknet.invoke(tx_deploy);

  return extrisinc_deploy;
}

export async function initialize(
  api: ApiPromise,
  contractAddress: string,
  tokenAddress: string
): Promise<string> {
  // Initialize contract
  const tx_initialize = {
    version: 1, // version of the transaction
    hash: "", // leave empty for now, will be filled in by the runtime
    signature: [], // leave empty for now, will be filled in when signing the transaction
    sender_address: contractAddress, // address of the sender contract
    nonce: 1, // nonce of the transaction
    callEntrypoint: {
      // call entrypoint
      classHash: null, // class hash of the contract
      entrypointSelector: null, // function selector of the transfer function
      calldata: [
        tokenAddress, // CONTRACT ADDRESS
        "0x0079dc0da7c54b95f10aa182ad0a46400db63156920adb65eca2654c0945a463", // SELECTOR
        5, // CALLDATA SIZE
        4, // INPUT SIZE
        1413829460, // NAME (TEST)
        1413829460, // SYMBOL (TEST)
        18, // DECIMALS (18)
        contractAddress, // PERMISSIONED MINTER
      ],
      storageAddress: contractAddress,
      callerAddress: contractAddress,
    },
    contractClass: null,
  };

  const extrisinc_init = api.tx.starknet.invoke(tx_initialize);

  return await sendTransaction(api, extrisinc_init);
}

export async function mint(
  api: ApiPromise,
  contractAddress: string,
  tokenAddress: string,
  mintAmount: string
): Promise<string> {
  // Initialize contract
  const tx_mint = {
    version: 1, // version of the transaction
    hash: "", // leave empty for now, will be filled in by the runtime
    signature: [], // leave empty for now, will be filled in when signing the transaction
    sender_address: contractAddress, // address of the sender contract
    nonce: 1, // nonce of the transaction
    callEntrypoint: {
      // call entrypoint
      classHash: null, // class hash of the contract
      entrypointSelector: null, // function selector of the transfer function
      calldata: [
        tokenAddress, // CONTRACT ADDRESS
        "0x00151e58b29179122a728eab07c8847e5baf5802379c5db3a7d57a8263a7bd1d", // SELECTOR (permissionedMint)
        "0x0000000000000000000000000000000000000000000000000000000000000003", // CALLDATA SIZE
        contractAddress, // RECIPIENT ADDRESS
        mintAmount, // AMOUNT
        "0x0000000000000000000000000000000000000000000000000000000000000000",
      ],
      storageAddress: contractAddress,
      callerAddress: contractAddress,
    },
    contractClass: null,
  };

  const extrisinc_mint = api.tx.starknet.invoke(tx_mint);

  return await sendTransaction(api, extrisinc_mint);
}

export function transfer(
  api: ApiPromise,
  contractAddress: string,
  tokenAddress: string,
  recipientAddress: string,
  transferAmount: string,
  nonce?: number
): SubmittableExtrinsic<ApiTypes, ISubmittableResult> {
  // Initialize contract
  const tx_transfer = {
    version: 1, // version of the transaction
    signature: [], // leave empty for now, will be filled in when signing the transaction
    sender_address: contractAddress, // address of the sender contract
    nonce: numberToU832Bytes(nonce ? nonce : 0), // nonce of the transaction
    calldata: [
      tokenAddress, // CONTRACT ADDRESS
      "0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e", // SELECTOR (transfer)
      "0x0000000000000000000000000000000000000000000000000000000000000003", // CALLDATA SIZE
      recipientAddress,
      transferAmount,
      "0x0000000000000000000000000000000000000000000000000000000000000000",
    ],
  };

  const extrisinc_transfer = api.tx.starknet.invoke(tx_transfer);

  return extrisinc_transfer;
}

export function batchTransfer(
  api: ApiPromise,
  contractAddress: string,
  tokenAddress: string,
  recipientAddress: string,
  transferAmount: string
): Array<SubmittableExtrinsic<ApiTypes, ISubmittableResult>> {
  // Initialize contract
  const tx_transfer = {
    version: 1, // version of the transaction
    signature: [], // leave empty for now, will be filled in when signing the transaction
    sender_address: contractAddress, // address of the sender contract
    nonce: 0, // nonce of the transaction
    calldata: [
      tokenAddress, // CONTRACT ADDRESS
      "0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e", // SELECTOR (transfer)
      "0x0000000000000000000000000000000000000000000000000000000000000003", // CALLDATA SIZE
      recipientAddress,
      transferAmount,
      "0x0000000000000000000000000000000000000000000000000000000000000000",
    ],
  };

  const extrisinc_transfer = api.tx.starknet.invoke(tx_transfer);

  const extrisinc_transfers = Array(200).fill(extrisinc_transfer);

  return extrisinc_transfers;
}
