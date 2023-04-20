import "@keep-starknet-strange/madara-api-augment";
import { ApiPromise } from "@polkadot/api";
import { SubmittableExtrinsic } from "@polkadot/api/types";
import { KeyringPair } from "@polkadot/keyring/types";
import { ISubmittableResult } from "@polkadot/types/types";
import { stringify, u8aToHex } from "@polkadot/util";
import erc20Json from "../contracts/compiled/erc20.json";
import { hash } from "starknet";

export async function sendTransactionNoValidation(
  api: ApiPromise,
  transaction: SubmittableExtrinsic<"promise", ISubmittableResult>,
  sender: KeyringPair
): Promise<void> {
  await transaction.signAndSend(sender, { nonce: -1 });
}

export async function sendTransactionBatchNoValidation(
  api: ApiPromise,
  transactions: SubmittableExtrinsic<"promise", ISubmittableResult>[],
  sender: KeyringPair
): Promise<void> {
  await api.tx.utility.batch(transactions).signAndSend(sender, { nonce: -1 });
}

export async function sendTransaction(
  api: ApiPromise,
  transaction: SubmittableExtrinsic<"promise", ISubmittableResult>,
  sender: KeyringPair
): Promise<string> {
  return new Promise((resolve, reject) => {
    let unsubscribe;
    let timeout;
    let transaction_success_event = false;
    let block_hash;
    const SPAWNING_TIME = 500000;

    transaction
      .signAndSend(
        sender,
        { nonce: -1 },
        async ({ events = [], status, dispatchError }) => {
          console.log(`Current status is ${status.type}`);

          // status would still be set, but in the case of error we can shortcut
          // to just check it (so an error would indicate InBlock or Finalized)
          if (dispatchError) {
            if (dispatchError.isModule) {
              // for module errors, we have the section indexed, lookup
              const decoded = api.registry.findMetaError(
                dispatchError.asModule
              );
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
        }
      )
      .then((unsub) => {
        unsubscribe = unsub;
      })
      .catch((error) => {
        console.error(error);
        reject(error);
      });

    timeout = setTimeout(() => {
      reject(new Error("Transaction timeout"));
    }, SPAWNING_TIME);
  });
}

export async function declare(
  api: ApiPromise,
  user: any,
  contractAddress: string,
  tokenClassHash: string
): Promise<string> {
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
      program: u8aToHex(Buffer.from(stringify(erc20Json.program))),
      entryPointsByType: u8aToHex(
        Buffer.from(stringify(erc20Json.entry_points_by_type))
      ),
    },
  };

  const extrisinc_declare = api.tx.starknet.declare(tx_declare);

  await sendTransaction(api, extrisinc_declare, user);
  return "";
}

export async function deploy(
  api: ApiPromise,
  user: any,
  contractAddress: string,
  tokenClassHash: string
): Promise<string> {
  // Compute contract address
  const deployedContractAddress = hash.calculateContractAddressFromHash(
    2,
    tokenClassHash,
    [],
    0
  );

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
      entrypointSelector: null,
      calldata: [
        "0x0000000000000000000000000000000000000000000000000000000000001111",
        "0x0169f135eddda5ab51886052d777a57f2ea9c162d713691b5e04a6d4ed71d47f",
        "0x0000000000000000000000000000000000000000000000000000000000000004",
        tokenClassHash,
        "0x0000000000000000000000000000000000000000000000000000000000000002", // Salt
        "0x0000000000000000000000000000000000000000000000000000000000000000", // Calldata len
        "0x0000000000000000000000000000000000000000000000000000000000000001", // Deploy from zero
      ],
      storageAddress: contractAddress,
      callerAddress: contractAddress,
    },
    contractClass: null,
  };

  const extrisinc_deploy = api.tx.starknet.invoke(tx_deploy);

  await sendTransaction(api, extrisinc_deploy, user);

  return deployedContractAddress;
}

export async function initialize(
  api: ApiPromise,
  user: any,
  contractAddress: string,
  tokenAddress: string
): Promise<string> {
  // Initialize contract
  let tx_initialize = {
    version: 1, // version of the transaction
    hash: "", // leave empty for now, will be filled in by the runtime
    signature: [], // leave empty for now, will be filled in when signing the transaction
    events: [], // empty vector for now, will be filled in by the runtime
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

  return sendTransaction(api, extrisinc_init, user);
}

export async function mint(
  api: ApiPromise,
  user: any,
  contractAddress: string,
  tokenAddress: string,
  mintAmount: string
): Promise<string> {
  // Initialize contract
  let tx_mint = {
    version: 1, // version of the transaction
    hash: "", // leave empty for now, will be filled in by the runtime
    signature: [], // leave empty for now, will be filled in when signing the transaction
    events: [], // empty vector for now, will be filled in by the runtime
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

  return sendTransaction(api, extrisinc_mint, user);
}

export async function transfer(
  api: ApiPromise,
  user: any,
  contractAddress: string,
  tokenAddress: string,
  recipientAddress: string,
  transferAmount: string
): Promise<string> {
  // Initialize contract
  let tx_transfer = {
    version: 1, // version of the transaction
    hash: "", // leave empty for now, will be filled in by the runtime
    signature: [], // leave empty for now, will be filled in when signing the transaction
    events: [], // empty vector for now, will be filled in by the runtime
    sender_address: contractAddress, // address of the sender contract
    nonce: 3, // nonce of the transaction
    callEntrypoint: {
      // call entrypoint
      classHash: null, // class hash of the contract
      entrypointSelector: null, // function selector of the transfer function
      calldata: [
        tokenAddress, // CONTRACT ADDRESS
        "0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e", // SELECTOR (transfer)
        "0x0000000000000000000000000000000000000000000000000000000000000003", // CALLDATA SIZE
        recipientAddress,
        transferAmount,
        "0x0000000000000000000000000000000000000000000000000000000000000000",
      ],
      storageAddress: contractAddress,
      callerAddress: contractAddress,
    },
    contractClass: null,
  };

  const extrisinc_transfer = api.tx.starknet.invoke(tx_transfer);

  await sendTransactionNoValidation(api, extrisinc_transfer, user);

  return "";
}

export async function batchTransfer(
  api: ApiPromise,
  user: any,
  contractAddress: string,
  tokenAddress: string,
  recipientAddress: string,
  transferAmount: string
): Promise<string> {
  // Initialize contract
  let tx_transfer = {
    version: 1, // version of the transaction
    hash: "", // leave empty for now, will be filled in by the runtime
    signature: [], // leave empty for now, will be filled in when signing the transaction
    events: [], // empty vector for now, will be filled in by the runtime
    sender_address: contractAddress, // address of the sender contract
    nonce: 3, // nonce of the transaction
    callEntrypoint: {
      // call entrypoint
      classHash: null, // class hash of the contract
      entrypointSelector: null, // function selector of the transfer function
      calldata: [
        tokenAddress, // CONTRACT ADDRESS
        "0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e", // SELECTOR (transfer)
        "0x0000000000000000000000000000000000000000000000000000000000000003", // CALLDATA SIZE
        recipientAddress,
        transferAmount,
        "0x0000000000000000000000000000000000000000000000000000000000000000",
      ],
      storageAddress: contractAddress,
      callerAddress: contractAddress,
    },
    contractClass: null,
  };

  const extrisinc_transfer = api.tx.starknet.invoke(tx_transfer);

  const extrisinc_transfers = Array(200).fill(extrisinc_transfer);

  await sendTransactionBatchNoValidation(api, extrisinc_transfers, user);

  return "";
}
