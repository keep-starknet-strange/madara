import "@keep-starknet-strange/madara-api-augment";
import chaiAsPromised from "chai-as-promised";
import chai, { expect } from "chai";
import deepEqualInAnyOrder from "deep-equal-in-any-order";
import {
  Account,
  AccountInvocationItem,
  LibraryError,
  RpcProvider,
  constants,
  hash,
  validateAndParseAddress,
  json,
  encode,
  CompressedProgram,
  LegacyContractClass,
  Signer,
} from "starknet";
import { ungzip } from "pako";
import { createAndFinalizeBlock, jumpBlocks } from "../../util/block";
import { describeDevMadara } from "../../util/setup-dev-tests";
import { cleanHex, rpcTransfer, starknetKeccak, toHex } from "../../util/utils";
import {
  ACCOUNT_CONTRACT,
  ACCOUNT_CONTRACT_CLASS_HASH,
  ARGENT_ACCOUNT_CLASS_HASH,
  ARGENT_CONTRACT_ADDRESS,
  ARGENT_PROXY_CLASS_HASH,
  CHAIN_ID_STARKNET_TESTNET,
  ERC721_CONTRACT,
  ERC20_CONTRACT,
  FEE_TOKEN_ADDRESS,
  MINT_AMOUNT,
  SALT,
  SEQUENCER_ADDRESS,
  SIGNER_PRIVATE,
  SIGNER_PUBLIC,
  TEST_CONTRACT,
  TEST_CONTRACT_ADDRESS,
  TEST_CONTRACT_CLASS_HASH,
  TOKEN_CLASS_HASH,
  UDC_CONTRACT_ADDRESS,
  DEPLOY_ACCOUNT_COST,
  TEST_CAIRO_1_SIERRA,
  TEST_CAIRO_1_CASM,
  CAIRO_1_ACCOUNT_CONTRACT,
  ERC20_CAIRO_1_CASM,
  ERC20_CAIRO_1_SIERRA,
  CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH,
} from "../constants";
import { Block, InvokeTransaction } from "./types";
import { assert, numberToHex } from "@polkadot/util";

function atobUniversal(a: string): Uint8Array {
  return encode.IS_BROWSER
    ? stringToArrayBuffer(atob(a))
    : Buffer.from(a, "base64");
}
function stringToArrayBuffer(s: string): Uint8Array {
  return Uint8Array.from(s, (c) => c.charCodeAt(0));
}
function decompressProgram(base64: CompressedProgram) {
  if (Array.isArray(base64)) return base64;
  const decompressed = encode.arrayBufferToString(
    ungzip(atobUniversal(base64))
  );
  return json.parse(decompressed);
}

chai.use(deepEqualInAnyOrder);
chai.use(chaiAsPromised);

// keep "let" over "const" as the nonce is passed by reference
// to abstract the increment
// eslint-disable-next-line prefer-const
let ARGENT_CONTRACT_NONCE = { value: 0 };
const CAIRO_1_NO_VALIDATE_ACCOUNT = { value: 0 };

describeDevMadara("Starknet RPC", (context) => {
  let providerRPC: RpcProvider;

  before(async function () {
    providerRPC = new RpcProvider({
      nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
      retries: 3,
    }); // substrate node
  });

  describe("getBlockhashAndNumber", () => {
    it("should not be undefined", async function () {
      const block = await providerRPC.getBlockHashAndNumber();

      expect(block).to.not.be.undefined;
      expect(block.block_hash).to.not.be.equal("");
      expect(block.block_number).to.be.equal(0);
    });
  });

  describe("getBlockNumber", async () => {
    it("should return current block number", async function () {
      const blockNumber = await providerRPC.getBlockNumber();

      expect(blockNumber).to.not.be.undefined;

      await jumpBlocks(context, 10);

      const blockNumber2 = await providerRPC.getBlockNumber();

      expect(blockNumber2).to.be.equal(blockNumber + 10);
    });
  });

  describe("getBlockTransactionCount", async () => {
    it("should return 0 for latest block", async function () {
      const transactionCount = await providerRPC.getTransactionCount("latest");

      expect(transactionCount).to.not.be.undefined;
      expect(transactionCount).to.be.equal(0);
    });

    it("should return 1 for 1 transaction", async function () {
      await context.createBlock(
        rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT
        ),
        {
          finalize: true,
        }
      );

      const transactionCount = await providerRPC.getTransactionCount("latest");

      expect(transactionCount).to.not.be.undefined;
      expect(transactionCount).to.be.equal(1);
    });

    it("should raise on invalid block id", async () => {
      const count = providerRPC.getTransactionCount("0x123");
      await expect(count)
        .to.eventually.be.rejectedWith("24: Block not found")
        .and.be.an.instanceOf(LibraryError);
    });
  });

  describe("getNonce", async () => {
    it("should increase after a transaction", async function () {
      let nonce = await providerRPC.getNonceForAddress(
        ARGENT_CONTRACT_ADDRESS,
        "latest"
      );

      await context.createBlock(
        rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT
        )
      );

      nonce = await providerRPC.getNonceForAddress(
        ARGENT_CONTRACT_ADDRESS,
        "latest"
      );

      expect(nonce).to.not.be.undefined;
      expect(nonce).to.be.equal(toHex(ARGENT_CONTRACT_NONCE.value));
    });
  });

  describe("call", async () => {
    it("should return calldata on return_result entrypoint", async function () {
      const call = await providerRPC.callContract(
        {
          contractAddress: TEST_CONTRACT_ADDRESS,
          entrypoint: "return_result",
          calldata: ["0x19"],
        },
        "latest"
      );

      expect(call.result).to.contain("0x19");
    });

    it("should raise with invalid entrypoint", async () => {
      const callResult = providerRPC.callContract(
        {
          contractAddress: TEST_CONTRACT_ADDRESS,
          entrypoint: "return_result_WRONG",
          calldata: ["0x19"],
        },
        "latest"
      );
      await expect(callResult)
        .to.eventually.be.rejectedWith("40: Contract error")
        .and.be.an.instanceOf(LibraryError);
    });
  });

  describe("getClassAt", async () => {
    it("should not be undefined", async function () {
      const contract_class = await providerRPC.getClassAt(
        TEST_CONTRACT_ADDRESS,
        "latest"
      );

      expect(contract_class).to.not.be.undefined;
      expect(contract_class.entry_points_by_type).to.deep.equal(
        TEST_CONTRACT.entry_points_by_type
      );
    });
  });

  describe("getClassHashAt", async () => {
    it("should return correct class hashes for account and test contract", async function () {
      const account_contract_class_hash = await providerRPC.getClassHashAt(
        ACCOUNT_CONTRACT,
        "latest"
      );

      expect(account_contract_class_hash).to.not.be.undefined;
      expect(validateAndParseAddress(account_contract_class_hash)).to.be.equal(
        ACCOUNT_CONTRACT_CLASS_HASH
      );

      const test_contract_class_hash = await providerRPC.getClassHashAt(
        TEST_CONTRACT_ADDRESS,
        "latest"
      );

      expect(test_contract_class_hash).to.not.be.undefined;
      expect(validateAndParseAddress(test_contract_class_hash)).to.be.equal(
        TEST_CONTRACT_CLASS_HASH
      );
    });

    it("should raise with invalid block id", async () => {
      // Invalid block id
      const classHash = providerRPC.getClassHashAt(
        TEST_CONTRACT_ADDRESS,
        "0x123"
      );
      await expect(classHash)
        .to.eventually.be.rejectedWith("24: Block not found")
        .and.be.an.instanceOf(LibraryError);
    });

    it("should raise with invalid contract address", async () => {
      // Invalid/un-deployed contract address
      const classHash = providerRPC.getClassHashAt("0x123", "latest");
      await expect(classHash)
        .to.eventually.be.rejectedWith("20: Contract not found")
        .and.be.an.instanceOf(LibraryError);
    });
  });

  describe("syncing", async () => {
    it("should return starting setup and current_block info", async function () {
      await jumpBlocks(context, 10);

      const status = await providerRPC.getSyncingStats();
      const current_block = await providerRPC.getBlockHashAndNumber();

      // starknet starting block number should be 0 with this test setup
      expect(status["starting_block_num"]).to.be.equal("0x0");
      // starknet current and highest block number should be equal to
      // the current block with this test setup
      expect(parseInt(status["current_block_num"])).to.be.equal(
        current_block["block_number"]
      );
      expect(parseInt(status["highest_block_num"])).to.be.equal(
        current_block["block_number"]
      );

      // the starknet block hash for number 0 starts with "0x31eb" with this test setup
      expect(status["starting_block_hash"]).to.contain("0x31eb");
      // starknet current and highest block number should be equal to
      // the current block with this test setup
      expect(status["current_block_hash"]).to.be.equal(
        current_block["block_hash"]
      );
      expect(status["highest_block_hash"]).to.be.equal(
        current_block["block_hash"]
      );
    });
  });

  describe("getClass", async () => {
    it("should return ERC_20 contract at class 0x10000", async function () {
      const contract_class = (await providerRPC.getClass(
        TOKEN_CLASS_HASH,
        "latest"
      )) as LegacyContractClass;
      // https://github.com/keep-starknet-strange/madara/issues/652
      // TODO: Compare program as well
      expect(contract_class.entry_points_by_type).to.deep.equal(
        ERC20_CONTRACT.entry_points_by_type
      );
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const program = json.parse(
        encode.arrayBufferToString(decompressProgram(contract_class.program))
      );
      // starknet js parses the values in the identifiers as negative numbers (maybe it's in madara).
      // FIXME: https://github.com/keep-starknet-strange/madara/issues/664
      // expect(program).to.deep.equal(ERC20_CONTRACT.program);
    });
  });

  describe("getBlockWithTxHashes", async () => {
    it("should return an empty block", async function () {
      await context.createBlock(undefined, {
        parentHash: undefined,
        finalize: true,
      });
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const latestBlock: Block = await providerRPC.getBlockWithTxHashes(
        "latest"
      );
      expect(latestBlock).to.not.be.undefined;
      expect(latestBlock.status).to.be.equal("ACCEPTED_ON_L2");
      expect(latestBlock.transactions.length).to.be.equal(0);
    });

    it("should returns transactions", async function () {
      await context.createBlock(
        rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT
        )
      );

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const blockWithTxHashes: Block = await providerRPC.getBlockWithTxHashes(
        "latest"
      );
      expect(blockWithTxHashes).to.not.be.undefined;
      expect(blockWithTxHashes.status).to.be.equal("ACCEPTED_ON_L2");
      expect(blockWithTxHashes.transactions.length).to.be.equal(1);
    });

    it("should raise with invalid block id", async function () {
      const block = providerRPC.getBlockWithTxHashes("0x123");
      await expect(block)
        .to.eventually.be.rejectedWith("24: Block not found")
        .and.be.an.instanceOf(LibraryError);
    });
  });

  describe("getBlockWithTxs", async () => {
    it("should returns empty block", async function () {
      await context.createBlock(undefined, {
        parentHash: undefined,
        finalize: true,
      });
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const latestBlock: Block = await providerRPC.getBlockWithTxs("latest");
      expect(latestBlock).to.not.be.undefined;
      expect(latestBlock.status).to.be.equal("ACCEPTED_ON_L2");
      expect(latestBlock.transactions.length).to.be.equal(0);
    });

    it("should returns transactions", async function () {
      await context.createBlock(
        rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT
        )
      );

      const blockHash = await providerRPC.getBlockHashAndNumber();
      await jumpBlocks(context, 10);

      const blockWithTxHashes = await providerRPC.getBlockWithTxs(
        blockHash.block_hash
      );
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const tx: InvokeTransaction = blockWithTxHashes.transactions[0];
      expect(blockWithTxHashes).to.not.be.undefined;
      expect(blockWithTxHashes.transactions.length).to.be.equal(1);
      expect(tx.type).to.be.equal("INVOKE");
      expect(tx.sender_address).to.be.equal(toHex(ARGENT_CONTRACT_ADDRESS));
      expect(tx.calldata).to.deep.equal(
        [
          1,
          FEE_TOKEN_ADDRESS,
          hash.getSelectorFromName("transfer"),
          0,
          3,
          3,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT,
          0,
        ].map(toHex)
      );
    });

    it("should raise with invalid block id", async function () {
      const block = providerRPC.getBlockWithTxs("0x123");
      await expect(block)
        .to.eventually.be.rejectedWith("24: Block not found")
        .and.be.an.instanceOf(LibraryError);
    });
  });

  describe("getStorageAt", async () => {
    it("should return value from the fee contract storage", async function () {
      const value = await providerRPC.getStorageAt(
        FEE_TOKEN_ADDRESS,
        // ERC20_balances(0x02).low
        "0x1d8bbc4f93f5ab9858f6c0c0de2769599fb97511503d5bf2872ef6846f2146f",
        "latest"
      );
      // fees were paid du to the transfer in the previous test so the value should be < u128::MAX
      expect(value).to.be.equal("0xfffffffffffffffffffffffffff97f4f");
    });

    it("should return 0 if the storage slot is not set", async function () {
      const value = await providerRPC.getStorageAt(
        FEE_TOKEN_ADDRESS,
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        "latest"
      );
      expect(value).to.be.equal("0x0");
    });

    it("should raise if the contract does not exist", async function () {
      const storage = providerRPC.getStorageAt(
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        "latest"
      );
      await expect(storage)
        .to.eventually.be.rejectedWith("20: Contract not found")
        .and.be.an.instanceOf(LibraryError);
    });
  });

  describe("getChainId", async () => {
    it("should return the correct value", async function () {
      const chainId = await providerRPC.getChainId();

      expect(chainId).to.not.be.undefined;
      expect(chainId).to.be.equal(CHAIN_ID_STARKNET_TESTNET);
    });
  });

  describe("getTransactionByBlockIdAndIndex", async () => {
    it("should returns 1 transaction", async function () {
      // Send a transaction
      await context.createBlock(
        rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT
        )
      );

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const tx: InvokeTransaction =
        await providerRPC.getTransactionByBlockIdAndIndex("latest", 0);
      expect(tx).to.not.be.undefined;
      expect(tx.type).to.be.equal("INVOKE");
      expect(tx.sender_address).to.be.equal(toHex(ARGENT_CONTRACT_ADDRESS));
      expect(tx.calldata).to.deep.equal(
        [
          1,
          FEE_TOKEN_ADDRESS,
          hash.getSelectorFromName("transfer"),
          0,
          3,
          3,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT,
          0,
        ].map(toHex)
      );
    });

    it("should throws block not found error", async function () {
      const transaction = providerRPC.getTransactionByBlockIdAndIndex(
        "0x123",
        2
      );
      await expect(transaction)
        .to.eventually.be.rejectedWith("24: Block not found")
        .and.be.an.instanceOf(LibraryError);
    });

    it("should throws invalid transaction index error", async function () {
      await context.createBlock(undefined, {
        parentHash: undefined,
        finalize: true,
      });
      const latestBlockCreated = await providerRPC.getBlockHashAndNumber();
      const transaction = providerRPC.getTransactionByBlockIdAndIndex(
        latestBlockCreated.block_hash,
        2
      );
      await expect(transaction)
        .to.eventually.be.rejectedWith(
          "27: Invalid transaction index in a block"
        )
        .and.be.an.instanceOf(LibraryError);
    });
  });

  describe("getStateUpdate", async () => {
    it("should return latest block state update", async function () {
      await context.createBlock(
        rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT
        ),
        {
          finalize: true,
        }
      );
      const stateUpdate = await providerRPC.getStateUpdate("latest");

      const latestBlock = await providerRPC.getBlockHashAndNumber();

      // TODO: Add real values

      expect(stateUpdate).to.not.be.undefined;
      assert(
        "block_hash" in stateUpdate,
        "block_hash is not in stateUpdate which means it's still pending"
      );
      expect(stateUpdate.block_hash).to.be.equal(latestBlock.block_hash);
      expect(stateUpdate.state_diff).to.deep.equal({
        storage_diffs: [],
        deprecated_declared_classes: [],
        declared_classes: [],
        deployed_contracts: [],
        replaced_classes: [],
        nonces: [],
      });
    });

    it("should return anterior block state update", async function () {
      const anteriorBlock = await providerRPC.getBlockHashAndNumber();

      await context.createBlock(
        rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT
        ),
        {
          finalize: true,
        }
      );
      const stateUpdate = await providerRPC.getStateUpdate(
        anteriorBlock.block_hash
      );

      // TODO: Add real values

      expect(stateUpdate).to.not.be.undefined;
      assert(
        "block_hash" in stateUpdate,
        "block_hash is not in stateUpdate which means it's still pending"
      );
      expect(stateUpdate.block_hash).to.be.equal(anteriorBlock.block_hash);
      expect(stateUpdate.state_diff).to.deep.equal({
        storage_diffs: [],
        deprecated_declared_classes: [],
        declared_classes: [],
        deployed_contracts: [],
        replaced_classes: [],
        nonces: [],
      });
    });

    it("should throw block not found error", async function () {
      const transaction = providerRPC.getStateUpdate("0x123");
      await expect(transaction)
        .to.eventually.be.rejectedWith("24: Block not found")
        .and.be.an.instanceOf(LibraryError);
    });
  });

  describe("addInvokeTransaction", async () => {
    it("should invoke successfully", async function () {
      const account = new Account(
        providerRPC,
        ARGENT_CONTRACT_ADDRESS,
        SIGNER_PRIVATE
      );

      await account.execute(
        {
          contractAddress: FEE_TOKEN_ADDRESS,
          entrypoint: "transfer",
          calldata: ["0xdeadbeef", "0x123", "0x0"],
        },
        undefined,
        {
          nonce: ARGENT_CONTRACT_NONCE.value,
          maxFee: "123456",
        }
      );
      ARGENT_CONTRACT_NONCE.value += 1;
      await jumpBlocks(context, 1);

      // ERC20_balances(0xdeadbeef).low = 0x4c761778f11aa10fc40190ff3127637fe00dc59bfa557bd4c8beb30a178f016
      const balance = await providerRPC.getStorageAt(
        FEE_TOKEN_ADDRESS,
        "0x04c761778f11aa10fc40190ff3127637fe00dc59bfa557bd4c8beb30a178f016",
        "latest"
      );
      expect(toHex(balance)).to.be.equal("0x123");
    });

    it("should deploy ERC20 via UDC", async function () {
      const account = new Account(
        providerRPC,
        ARGENT_CONTRACT_ADDRESS,
        SIGNER_PRIVATE
      );

      const calldata = [
        numberToHex(1, 256), // Token Name
        numberToHex(1, 256), // Token Symbol
        numberToHex(18, 256), // Token Decimals
        numberToHex(42, 256), // Initial Supply
        "0x0000000000000000000000000000000000000000000000000000000000000000", // Initial Supply Cont { since u256 }
        "0xdeadbeef", // Recipient
      ];

      const deployedContractAddress = hash.calculateContractAddressFromHash(
        SALT,
        TOKEN_CLASS_HASH,
        calldata,
        0
      );

      await account.execute(
        {
          contractAddress: UDC_CONTRACT_ADDRESS,
          entrypoint: "deployContract",
          calldata: [TOKEN_CLASS_HASH, SALT, "0x0", "0x6", ...calldata],
        },
        undefined,
        {
          nonce: ARGENT_CONTRACT_NONCE.value,
          maxFee: "123456",
        }
      );
      ARGENT_CONTRACT_NONCE.value += 1;
      await jumpBlocks(context, 1);

      // ERC20_balances(0xdeadbeef).low = 0x4c761778f11aa10fc40190ff3127637fe00dc59bfa557bd4c8beb30a178f016
      const balance = await providerRPC.getStorageAt(
        deployedContractAddress,
        "0x04c761778f11aa10fc40190ff3127637fe00dc59bfa557bd4c8beb30a178f016",
        "latest"
      );
      expect(toHex(balance)).to.be.equal("0x2a");
    });
  });

  describe("addDeployAccountTransaction", async () => {
    it("should deploy successfully", async function () {
      // Compute contract address
      const selector = hash.getSelectorFromName("initialize");
      const calldata = [
        ARGENT_ACCOUNT_CLASS_HASH,
        selector,
        2,
        SIGNER_PUBLIC,
        0,
      ];

      const deployedContractAddress = hash.calculateContractAddressFromHash(
        SALT,
        ARGENT_PROXY_CLASS_HASH,
        calldata,
        0
      );
      // fund address
      await rpcTransfer(
        providerRPC,
        ARGENT_CONTRACT_NONCE,
        deployedContractAddress,
        DEPLOY_ACCOUNT_COST
      );
      await jumpBlocks(context, 1);

      const invocationDetails = {
        nonce: "0x0",
        maxFee: "0x1111111111111111111111",
        version: "0x1",
      };

      const signer = new Signer(SIGNER_PRIVATE);
      const signature = await signer.signDeployAccountTransaction({
        classHash: ARGENT_PROXY_CLASS_HASH,
        contractAddress: deployedContractAddress,
        constructorCalldata: calldata,
        addressSalt: SALT,
        maxFee: invocationDetails.maxFee,
        version: invocationDetails.version,
        chainId: constants.StarknetChainId.SN_GOERLI,
        nonce: invocationDetails.nonce,
      });

      // Deploy account contract
      const txDeployAccount = {
        signature: signature, // signature
        contractAddress: deployedContractAddress, // address of the sender contract
        addressSalt: SALT, // contract address salt
        classHash: ARGENT_PROXY_CLASS_HASH, // class hash of the contract
        constructorCalldata: calldata,
      };

      await providerRPC.deployAccountContract(
        txDeployAccount,
        invocationDetails
      );
      await createAndFinalizeBlock(context.polkadotApi);

      const accountContractClassHash = await providerRPC.getClassHashAt(
        deployedContractAddress
      );

      expect(validateAndParseAddress(accountContractClassHash)).to.be.equal(
        ARGENT_PROXY_CLASS_HASH
      );
    });
  });

  // TODO:
  //    - once starknet-rs supports query tx version
  //    - test w/ account.estimateInvokeFee, account.estimateDeclareFee, account.estimateAccountDeployFee
  describe("estimateFee", async () => {
    it("should estimate fee", async function () {
      const tx = {
        contractAddress: ACCOUNT_CONTRACT,
        calldata: [
          TEST_CONTRACT_ADDRESS,
          "0x36fa6de2810d05c3e1a0ebe23f60b9c2f4629bbead09e5a9704e1c5632630d5",
          "0x0",
        ],
        signature: [],
      };

      const nonce = await providerRPC.getNonceForAddress(
        ACCOUNT_CONTRACT,
        "latest"
      );

      const txDetails = {
        nonce: nonce,
        version: "0x1",
      };

      const invocation: AccountInvocationItem = {
        type: "INVOKE_FUNCTION",
        ...tx,
        ...txDetails,
      };

      const fee_estimates = await providerRPC.getEstimateFeeBulk([invocation], {
        blockIdentifier: "latest",
      });

      expect(fee_estimates[0].overall_fee > 0n).to.be.equal(true);
      expect(fee_estimates[0].gas_consumed > 0n).to.be.equal(true);
    });

    it("should raise if contract does not exist", async function () {
      const tx = {
        contractAddress: ACCOUNT_CONTRACT,
        calldata: [
          "0x000000000000000000000000000000000000000000000000000000000000DEAD",
          "0x36fa6de2810d05c3e1a0ebe23f60b9c2f4629bbead09e5a9704e1c5632630d5",
          "0x0",
        ],
        signature: [],
      };

      const nonce = await providerRPC.getNonceForAddress(
        ACCOUNT_CONTRACT,
        "latest"
      );

      const txDetails = {
        nonce: nonce,
        version: "0x1",
      };

      const invocation: AccountInvocationItem = {
        type: "INVOKE_FUNCTION",
        ...tx,
        ...txDetails,
      };

      const fee_estimates = providerRPC.getEstimateFeeBulk([invocation], {
        blockIdentifier: "latest",
      });

      //    TODO: once starknet-js supports estimateFee using array
      //   expect(estimate).to.eventually.be.rejectedWith(
      //     "invalid type: map, expected variant identifier"
      //   );

      expect(fee_estimates)
        .to.eventually.be.rejectedWith("40: Contract error")
        .and.be.an.instanceOf(LibraryError);
    });

    it("should estimate fees for multiple invocations", async function () {
      const tx = {
        contractAddress: ACCOUNT_CONTRACT,
        calldata: [
          TEST_CONTRACT_ADDRESS,
          "0x36fa6de2810d05c3e1a0ebe23f60b9c2f4629bbead09e5a9704e1c5632630d5",
          "0x0",
        ],
        signature: [],
      };

      const nonce = await providerRPC.getNonceForAddress(
        ACCOUNT_CONTRACT,
        "latest"
      );

      const txDetails = {
        nonce: nonce,
        version: "0x1",
      };

      const invocation: AccountInvocationItem = {
        type: "INVOKE_FUNCTION",
        ...tx,
        ...txDetails,
      };

      const fee_estimates = await providerRPC.getEstimateFeeBulk(
        [invocation, invocation],
        {
          blockIdentifier: "latest",
        }
      );

      expect(fee_estimates[0].overall_fee > 0n).to.be.equal(true);
      expect(fee_estimates[0].gas_consumed > 0n).to.be.equal(true);
      expect(fee_estimates[1].overall_fee > 0n).to.be.equal(true);
      expect(fee_estimates[1].gas_consumed > 0n).to.be.equal(true);
    });

    it("should return empty array if no invocations", async function () {
      const fee_estimates = await providerRPC.getEstimateFeeBulk([], {
        blockIdentifier: "latest",
      });

      expect(fee_estimates.length == 0).to.be.equal(true);
    });
  });

  describe("addDeclareTransaction", async () => {
    it("should set class at given class hash (legacy)", async function () {
      const account = new Account(
        providerRPC,
        ARGENT_CONTRACT_ADDRESS,
        SIGNER_PRIVATE
      );
      // computed via: starkli class-hash ./cairo-contracts/build/ERC20.json
      // the above command should be used at project root
      const classHash =
        "0x372ee6669dc86563007245ed7343d5180b96221ce28f44408cff2898038dbd4";
      const res = await account.declare(
        {
          classHash: classHash,
          contract: ERC20_CONTRACT,
        },
        { nonce: ARGENT_CONTRACT_NONCE.value, version: 1, maxFee: "123456" }
      );
      ARGENT_CONTRACT_NONCE.value += 1;
      await jumpBlocks(context, 1);

      const contractClassActual = await providerRPC.getClass(
        classHash,
        "latest"
      );
      expect(contractClassActual.entry_points_by_type).to.deep.equal(
        ERC20_CONTRACT.entry_points_by_type
      );
      // TODO compare the program as well
      // expect(contractClassActual.program).to.be.equal(
      //   stark.compressProgram(ERC20_CONTRACT.program)
      // );
      expect(res.class_hash).to.be.eq(classHash);
    });

    it("should set class at given class hash and deploy a new contract (cairo 1)", async function () {
      const account = new Account(
        providerRPC,
        CAIRO_1_ACCOUNT_CONTRACT,
        "0x123" // it's the no validate account
      );
      // computed via: starknetjs 5.14.1
      const classHash =
        "0x9cf5ef6166edaa87767d05bbfd54ad02fd110028597343a200e82949ce05cf";
      const res = await account.declare(
        {
          casm: TEST_CAIRO_1_CASM,
          contract: TEST_CAIRO_1_SIERRA,
        },
        {
          nonce: CAIRO_1_NO_VALIDATE_ACCOUNT.value,
          version: 1,
          maxFee: "123456",
        }
      );
      CAIRO_1_NO_VALIDATE_ACCOUNT.value += 1;
      await jumpBlocks(context, 1);

      const contractClassActual = await providerRPC.getClass(
        classHash,
        "latest"
      );
      // TODO: (Apoorv) make these checks better once we to_rpc_contract_class is fixed #775 and #790
      expect(contractClassActual).to.have.property("entry_points_by_type");
      expect(contractClassActual).to.have.property("sierra_program");
      expect(contractClassActual).to.have.property("contract_class_version");
      expect(contractClassActual).to.have.property("abi");
      expect(res.class_hash).to.be.eq(classHash);
    });

    it("should fail to declare duplicate class", async function () {
      const account = new Account(
        providerRPC,
        ARGENT_CONTRACT_ADDRESS,
        SIGNER_PRIVATE
      );

      // computed via: starkli class-hash ./cairo-contracts/build/ERC20.json
      // the above command should be used at project root
      const classHash =
        "0x372ee6669dc86563007245ed7343d5180b96221ce28f44408cff2898038dbd4";

      await expect(
        account.declare(
          {
            classHash: classHash,
            contract: ERC20_CONTRACT,
          },
          { nonce: ARGENT_CONTRACT_NONCE.value, version: 1, maxFee: "123456" }
        )
      ).to.be.rejectedWith("51: Class already declared");
    });
  });

  describe("pendingTransactions", async () => {
    it("should return all the starknet invoke transactions", async function () {
      // create a invoke transaction
      await rpcTransfer(
        providerRPC,
        ARGENT_CONTRACT_NONCE,
        ARGENT_CONTRACT_ADDRESS,
        MINT_AMOUNT
      );

      const txs = await providerRPC.getPendingTransactions();

      expect(txs.length).equals(1);

      expect(txs[0]).to.include({ type: "INVOKE" });
      expect(txs[0]).that.includes.all.keys([
        "transaction_hash",
        "max_fee",
        "version",
        "signature",
        "nonce",
        "type",
        "sender_address",
        "calldata",
      ]);

      await jumpBlocks(context, 10);
    });

    it("should return all starknet declare transactions", async function () {
      const account = new Account(
        providerRPC,
        ARGENT_CONTRACT_ADDRESS,
        SIGNER_PRIVATE
      );

      // computed via: starkli class-hash ./cairo-contracts/build/ERC721.json
      // the above command should be used at project root
      const classHash =
        "0x077cc28ed3c661419fda16bf120fb81f1f8f28617f5543b05a86d63b0926bbf4";
      await account.declare(
        {
          classHash: classHash,
          contract: ERC721_CONTRACT,
        },
        { nonce: ARGENT_CONTRACT_NONCE.value, version: 1, maxFee: "123456" }
      );

      const txs = await providerRPC.getPendingTransactions();

      expect(txs.length).equals(1);

      expect(txs[0]).to.include({ type: "DECLARE" });
      expect(txs[0]).that.includes.all.keys([
        "sender_address",
        "class_hash",
        "max_fee",
        "nonce",
        "signature",
        "transaction_hash",
        "type",
        "version",
      ]);

      await jumpBlocks(context, 10);
    });

    it("should return all starknet deploy_account transactions", async function () {
      // create a deploy_contract transaction
      const selector = hash.getSelectorFromName("initialize");
      const calldata = [
        ARGENT_ACCOUNT_CLASS_HASH,
        selector,
        2,
        SIGNER_PUBLIC,
        0,
      ];

      const deployedContractAddress = hash.calculateContractAddressFromHash(
        SALT,
        ARGENT_PROXY_CLASS_HASH,
        calldata,
        0
      );

      const invocationDetails = {
        nonce: "0x0",
        maxFee: "0x1111111111111111111111",
        version: "0x1",
      };

      const signer = new Signer(SIGNER_PRIVATE);
      const signature = await signer.signDeployAccountTransaction({
        classHash: ARGENT_PROXY_CLASS_HASH,
        contractAddress: deployedContractAddress,
        constructorCalldata: calldata,
        addressSalt: SALT,
        maxFee: invocationDetails.maxFee,
        version: invocationDetails.version,
        chainId: constants.StarknetChainId.SN_GOERLI,
        nonce: invocationDetails.nonce,
      });

      // Deploy account contract
      const txDeployAccount = {
        signature: signature, // signature
        contractAddress: deployedContractAddress, // address of the sender contract
        addressSalt: SALT, // contract address salt
        classHash: ARGENT_PROXY_CLASS_HASH, // class hash of the contract
        constructorCalldata: calldata,
      };

      await providerRPC.deployAccountContract(
        txDeployAccount,
        invocationDetails
      );

      const txs = await providerRPC.getPendingTransactions();

      expect(txs.length).equals(1);
      expect(txs[0]).to.include({ type: "DEPLOY_ACCOUNT" });
      expect(txs[0]).that.includes.all.keys([
        "class_hash",
        "constructor_calldata",
        "contract_address_salt",
        "max_fee",
        "nonce",
        "signature",
        "transaction_hash",
        "type",
        "version",
      ]);

      await jumpBlocks(context, 10);
    });

    it("should return transactions from the ready and future queues", async function () {
      const transactionNonceOffset = 1_000;
      // ready transaction
      await rpcTransfer(
        providerRPC,
        ARGENT_CONTRACT_NONCE,
        ARGENT_CONTRACT_ADDRESS,
        MINT_AMOUNT
      );
      // future transaction
      // add a high number to the nonce to make sure the transaction is added to the future queue
      await rpcTransfer(
        providerRPC,
        { value: ARGENT_CONTRACT_NONCE.value + transactionNonceOffset },
        ARGENT_CONTRACT_ADDRESS,
        MINT_AMOUNT
      );

      // the pendingExtrinsics endpoint returns only the ready transactions
      // (https://github.com/paritytech/substrate/blob/master/client/rpc/src/author/mod.rs#L153)
      const readyExtrinsics =
        await context.polkadotApi.rpc.author.pendingExtrinsics();
      const readyTxs = readyExtrinsics.map((pending) => {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const obj: any = pending.toHuman();
        return {
          type: obj.method.method.toUpperCase(),
          nonce: toHex(obj.method.args.transaction.nonce),
        };
      });

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const txs: InvokeTransaction[] =
        await providerRPC.getPendingTransactions();

      expect(readyExtrinsics.length).to.be.equal(1);
      expect(txs.length).to.be.equal(2);

      expect(readyTxs[0]).to.include({
        type: "INVOKE",
        nonce: toHex(ARGENT_CONTRACT_NONCE.value - 1),
      });
      expect(txs[0]).to.include({
        type: "INVOKE",
        nonce: toHex(ARGENT_CONTRACT_NONCE.value - 1),
      });
      expect(txs[1]).to.include({
        type: "INVOKE",
        nonce: toHex(ARGENT_CONTRACT_NONCE.value + transactionNonceOffset),
      });

      await jumpBlocks(context, 10);
    });
  });

  describe("getTransactionByHash", () => {
    it("should return a transaction", async function () {
      await createAndFinalizeBlock(context.polkadotApi);

      // Send a transaction
      const b = await context.createBlock(
        rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT
        ),
        {
          finalize: true,
        }
      );

      const r = await providerRPC.getTransactionByHash(b.result.hash);
      expect(r).to.not.be.undefined;
    });

    it("should return transaction hash not found", async function () {
      // Send a transaction
      await context.createBlock(
        rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT
        )
      );

      const transaction = providerRPC.getTransactionByHash("0x1234");
      await expect(transaction)
        .to.eventually.be.rejectedWith("25: Transaction hash not found")
        .and.be.an.instanceOf(LibraryError);
    });

    it("should return transaction hash not found when a transaction is in the pool", async function () {
      await createAndFinalizeBlock(context.polkadotApi);

      // create a invoke transaction
      const b = await rpcTransfer(
        providerRPC,
        ARGENT_CONTRACT_NONCE,
        ARGENT_CONTRACT_ADDRESS,
        MINT_AMOUNT
      );

      const transaction = providerRPC.getTransactionByHash(b.transaction_hash);
      await expect(transaction)
        .to.eventually.be.rejectedWith("25: Transaction hash not found")
        .and.be.an.instanceOf(LibraryError);
    });
  });

  describe("getTransactionReceipt", () => {
    it("should return a receipt", async function () {
      await createAndFinalizeBlock(context.polkadotApi);

      // Send a transaction
      const b = await context.createBlock(
        rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT
        ),
        {
          finalize: true,
        }
      );

      const block_hash_and_number = await providerRPC.getBlockHashAndNumber();
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const r: TransactionReceipt = await providerRPC.getTransactionReceipt(
        b.result.hash
      );
      expect(r).to.not.be.undefined;
      expect(r.block_hash).to.be.equal(block_hash_and_number.block_hash);
      expect(r.block_number).to.be.equal(block_hash_and_number.block_number);
    });

    it("should return transaction hash not found", async function () {
      // Send a transaction
      await context.createBlock(
        rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT
        )
      );

      const transaction = providerRPC.getTransactionReceipt("0x1234");
      await expect(transaction)
        .to.eventually.be.rejectedWith("25: Transaction hash not found")
        .and.be.an.instanceOf(LibraryError);
    });
  });
  describe("getEvents", function () {
    it("should fail on invalid continuation token", async function () {
      const filter = {
        from_block: { block_number: 0 },
        to_block: { block_number: 1 },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 1,
        continuation_token: "0xabdel",
        keys: [[]],
      };

      let events = providerRPC.getEvents(filter);
      await expect(events)
        .to.eventually.be.rejectedWith(
          "33: The supplied continuation token is invalid or unknown"
        )
        .and.be.an.instanceOf(LibraryError);

      // Send transactions
      const transactions = [];
      for (let i = 0; i < 5; i++) {
        transactions.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT
          )
        );
      }
      await context.createBlock(transactions);
      const block = await providerRPC.getBlockHashAndNumber();
      let filter2 = {
        from_block: { block_number: block.block_number },
        to_block: { block_number: block.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 1,
        continuation_token: "0,100,1",
        keys: [[]],
      };

      events = providerRPC.getEvents(filter2);
      await expect(events)
        .to.eventually.be.rejectedWith(
          "33: The supplied continuation token is invalid or unknown"
        )
        .and.be.an.instanceOf(LibraryError);

      filter2 = {
        from_block: { block_number: block.block_number },
        to_block: { block_number: block.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 1,
        continuation_token: "0,0,100",
        keys: [[]],
      };

      events = providerRPC.getEvents(filter2);
      await expect(events)
        .to.eventually.be.rejectedWith(
          "33: The supplied continuation token is invalid or unknown"
        )
        .and.be.an.instanceOf(LibraryError);
    });

    it("should fail on chunk size too big", async function () {
      const filter = {
        from_block: { block_number: 0 },
        to_block: { block_number: 1 },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 1001,
        keys: [[]],
      };

      const events = providerRPC.getEvents(filter);
      await expect(events)
        .to.eventually.be.rejectedWith("31: Requested page size is too big")
        .and.be.an.instanceOf(LibraryError);
    });

    it("should fail on keys too big", async function () {
      const filter = {
        from_block: { block_number: 0 },
        to_block: { block_number: 1 },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 1,
        keys: Array(101).fill(["0x0"]),
      };

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const events = providerRPC.getEvents(filter);
      await expect(events)
        .to.eventually.be.rejectedWith("34: Too many keys provided in a filter")
        .and.be.an.instanceOf(LibraryError);
    });

    it("returns expected events on correct filter", async function () {
      // Send a transaction
      await context.createBlock(
        rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT
        )
      );

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const tx: InvokeTransaction =
        await providerRPC.getTransactionByBlockIdAndIndex("latest", 0);
      const block_hash_and_number = await providerRPC.getBlockHashAndNumber();
      const filter = {
        from_block: "latest",
        to_block: "latest",
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 10,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const events = await providerRPC.getEvents(filter);

      expect(events.events.length).to.be.equal(2);
      expect(events.continuation_token).to.be.null;
      for (const event of events.events) {
        expect(validateAndParseAddress(event.from_address)).to.be.equal(
          FEE_TOKEN_ADDRESS
        );
        expect(event.transaction_hash).to.be.equal(tx.transaction_hash);
      }
      // check transfer event
      const transfer_event = events.events[0];
      expect(transfer_event).to.deep.equal({
        transaction_hash: tx.transaction_hash,
        block_hash: block_hash_and_number.block_hash,
        block_number: block_hash_and_number.block_number,
        from_address: cleanHex(FEE_TOKEN_ADDRESS),
        keys: [toHex(starknetKeccak("Transfer"))],
        data: [
          ARGENT_CONTRACT_ADDRESS,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT,
          "0x0",
        ].map(cleanHex),
      });
      // check fee transfer event
      const fee_event = events.events[1];
      expect(fee_event).to.deep.equal({
        transaction_hash: tx.transaction_hash,
        block_hash: block_hash_and_number.block_hash,
        block_number: block_hash_and_number.block_number,
        from_address: cleanHex(FEE_TOKEN_ADDRESS),
        keys: [toHex(starknetKeccak("Transfer"))],
        data: [
          ARGENT_CONTRACT_ADDRESS,
          SEQUENCER_ADDRESS,
          "0x1a02c", // current fee perceived for the transfer
          "0x0",
        ].map(cleanHex),
      });
    });

    it("returns expected events on correct filter two blocks", async function () {
      // Send transactions
      const transactions = [];
      for (let i = 0; i < 5; i++) {
        transactions.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT
          )
        );
      }
      await context.createBlock(transactions);
      const firstBlockCreated = await providerRPC.getBlockHashAndNumber();
      // Second block
      const transactions2 = [];
      for (let i = 0; i < 5; i++) {
        transactions2.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT
          )
        );
      }
      await context.createBlock(transactions2);
      const secondBlockCreated = await providerRPC.getBlockHashAndNumber();

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const filter = {
        from_block: { block_number: firstBlockCreated.block_number },
        to_block: { block_number: secondBlockCreated.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 100,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const events = await providerRPC.getEvents(filter);

      expect(events.events.length).to.be.equal(20);
      expect(events.continuation_token).to.be.null;
      for (let i = 0; i < 2; i++) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const tx: InvokeTransaction =
          await providerRPC.getTransactionByBlockIdAndIndex(
            firstBlockCreated.block_hash,
            i
          );
        expect(
          validateAndParseAddress(events.events[2 * i].from_address)
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events.events[2 * i].transaction_hash).to.be.equal(
          tx.transaction_hash
        );
        expect(
          validateAndParseAddress(events.events[2 * i + 1].from_address)
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events.events[2 * i + 1].transaction_hash).to.be.equal(
          tx.transaction_hash
        );
      }
      for (let i = 0; i < 2; i++) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const tx_second_block: InvokeTransaction =
          await providerRPC.getTransactionByBlockIdAndIndex(
            secondBlockCreated.block_hash,
            i
          );
        expect(
          validateAndParseAddress(events.events[10 + 2 * i].from_address)
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events.events[10 + 2 * i].transaction_hash).to.be.equal(
          tx_second_block.transaction_hash
        );
        expect(
          validateAndParseAddress(events.events[10 + 2 * i + 1].from_address)
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events.events[10 + 2 * i + 1].transaction_hash).to.be.equal(
          tx_second_block.transaction_hash
        );
      }
    });

    it("returns expected events on correct filter two blocks pagination", async function () {
      // Send transactions
      const transactions = [];
      for (let i = 0; i < 5; i++) {
        transactions.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT
          )
        );
      }
      await context.createBlock(transactions);
      const firstBlockCreated = await providerRPC.getBlockHashAndNumber();
      // Second block
      const transactions2 = [];
      for (let i = 0; i < 5; i++) {
        transactions2.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT
          )
        );
      }
      await context.createBlock(transactions2);
      const secondBlockCreated = await providerRPC.getBlockHashAndNumber();

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      let filter = {
        from_block: { block_number: firstBlockCreated.block_number },
        to_block: { block_number: secondBlockCreated.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 7,
        continuation_token: null,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      let { events, continuation_token } = await providerRPC.getEvents(filter);

      expect(events.length).to.be.equal(7);
      expect(continuation_token).to.be.equal("0,3,2");

      for (let i = 0; i < 3; i++) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const tx: InvokeTransaction =
          await providerRPC.getTransactionByBlockIdAndIndex(
            firstBlockCreated.block_hash,
            i
          );
        expect(validateAndParseAddress(events[2 * i].from_address)).to.be.equal(
          FEE_TOKEN_ADDRESS
        );
        expect(events[2 * i].transaction_hash).to.be.equal(tx.transaction_hash);
        expect(
          validateAndParseAddress(events[2 * i + 1].from_address)
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events[2 * i + 1].transaction_hash).to.be.equal(
          tx.transaction_hash
        );
      }
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const tx3: InvokeTransaction =
        await providerRPC.getTransactionByBlockIdAndIndex(
          firstBlockCreated.block_hash,
          3
        );
      expect(validateAndParseAddress(events[6].from_address)).to.be.equal(
        FEE_TOKEN_ADDRESS
      );
      expect(events[6].transaction_hash).to.be.equal(tx3.transaction_hash);

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      filter = {
        from_block: { block_number: firstBlockCreated.block_number },
        to_block: { block_number: secondBlockCreated.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 7,
        continuation_token: continuation_token,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      ({ events, continuation_token } = await providerRPC.getEvents(filter));

      expect(events.length).to.be.equal(7);
      expect(continuation_token).to.be.equal("1,1,3");

      expect(validateAndParseAddress(events[0].from_address)).to.be.equal(
        FEE_TOKEN_ADDRESS
      );
      expect(events[0].transaction_hash).to.be.equal(tx3.transaction_hash);

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const tx4: InvokeTransaction =
        await providerRPC.getTransactionByBlockIdAndIndex(
          firstBlockCreated.block_hash,
          4
        );
      expect(validateAndParseAddress(events[1].from_address)).to.be.equal(
        FEE_TOKEN_ADDRESS
      );
      expect(events[1].transaction_hash).to.be.equal(tx4.transaction_hash);
      expect(validateAndParseAddress(events[2].from_address)).to.be.equal(
        FEE_TOKEN_ADDRESS
      );
      expect(events[2].transaction_hash).to.be.equal(tx4.transaction_hash);

      for (let i = 0; i < 2; i++) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const tx: InvokeTransaction =
          await providerRPC.getTransactionByBlockIdAndIndex(
            secondBlockCreated.block_hash,
            i
          );
        expect(
          validateAndParseAddress(events[2 * i + 3].from_address)
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events[2 * i + 3].transaction_hash).to.be.equal(
          tx.transaction_hash
        );
        expect(
          validateAndParseAddress(events[2 * i + 4].from_address)
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events[2 * i + 4].transaction_hash).to.be.equal(
          tx.transaction_hash
        );
      }

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      filter = {
        from_block: { block_number: firstBlockCreated.block_number },
        to_block: { block_number: secondBlockCreated.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 7,
        continuation_token: continuation_token,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      ({ events, continuation_token } = await providerRPC.getEvents(filter));

      expect(events.length).to.be.equal(6);
      expect(continuation_token).to.be.null;

      for (let i = 2; i < 5; i++) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const tx: InvokeTransaction =
          await providerRPC.getTransactionByBlockIdAndIndex(
            secondBlockCreated.block_hash,
            i
          );
        expect(
          validateAndParseAddress(events[2 * i - 4].from_address)
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events[2 * i - 4].transaction_hash).to.be.equal(
          tx.transaction_hash
        );
        expect(
          validateAndParseAddress(events[2 * i - 3].from_address)
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events[2 * i - 3].transaction_hash).to.be.equal(
          tx.transaction_hash
        );
      }
    });

    it("returns expected events on correct filter many blocks pagination", async function () {
      // Send transactions
      const transactions = [];
      for (let i = 0; i < 5; i++) {
        transactions.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT
          )
        );
      }
      await context.createBlock(transactions);
      const firstBlockCreated = await providerRPC.getBlockHashAndNumber();

      // 3 blocks without transactions
      const empty_transactions = [];
      await context.createBlock(empty_transactions);
      await context.createBlock(empty_transactions);
      await context.createBlock(empty_transactions);
      // Second block
      const transactions2 = [];
      for (let i = 0; i < 5; i++) {
        transactions2.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT
          )
        );
      }
      await context.createBlock(transactions2);
      const fifthBlockCreated = await providerRPC.getBlockHashAndNumber();

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      let filter = {
        from_block: { block_number: firstBlockCreated.block_number },
        to_block: { block_number: fifthBlockCreated.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 10,
        continuation_token: null,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      let { events, continuation_token } = await providerRPC.getEvents(filter);

      expect(events.length).to.be.equal(10);
      expect(continuation_token).to.be.equal("0,4,3");

      for (let i = 0; i < 5; i++) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const tx: InvokeTransaction =
          await providerRPC.getTransactionByBlockIdAndIndex(
            firstBlockCreated.block_hash,
            i
          );
        expect(validateAndParseAddress(events[2 * i].from_address)).to.be.equal(
          FEE_TOKEN_ADDRESS
        );
        expect(events[2 * i].transaction_hash).to.be.equal(tx.transaction_hash);
        expect(
          validateAndParseAddress(events[2 * i + 1].from_address)
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events[2 * i + 1].transaction_hash).to.be.equal(
          tx.transaction_hash
        );
      }

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      filter = {
        from_block: { block_number: firstBlockCreated.block_number },
        to_block: { block_number: fifthBlockCreated.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 10,
        continuation_token: continuation_token,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      ({ events, continuation_token } = await providerRPC.getEvents(filter));

      expect(events.length).to.be.equal(10);
      expect(continuation_token).to.be.null;

      for (let i = 0; i < 5; i++) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const tx: InvokeTransaction =
          await providerRPC.getTransactionByBlockIdAndIndex(
            fifthBlockCreated.block_hash,
            i
          );
        expect(validateAndParseAddress(events[2 * i].from_address)).to.be.equal(
          FEE_TOKEN_ADDRESS
        );
        expect(events[2 * i].transaction_hash).to.be.equal(tx.transaction_hash);
        expect(
          validateAndParseAddress(events[2 * i + 1].from_address)
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events[2 * i + 1].transaction_hash).to.be.equal(
          tx.transaction_hash
        );
      }
    });

    it("returns expected events on correct filter many empty blocks pagination", async function () {
      // Send transactions
      const transactions = [];
      for (let i = 0; i < 5; i++) {
        transactions.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT
          )
        );
      }
      await context.createBlock(transactions);
      const firstBlockCreated = await providerRPC.getBlockHashAndNumber();

      // 4 blocks without transactions
      const empty_transactions = [];
      await context.createBlock(empty_transactions);
      await context.createBlock(empty_transactions);
      await context.createBlock(empty_transactions);
      await context.createBlock(empty_transactions);

      const fifthBlockCreated = await providerRPC.getBlockHashAndNumber();

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      let filter = {
        from_block: { block_number: firstBlockCreated.block_number },
        to_block: { block_number: fifthBlockCreated.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 10,
        continuation_token: null,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      let { events, continuation_token } = await providerRPC.getEvents(filter);

      expect(events.length).to.be.equal(10);
      expect(continuation_token).to.be.equal("0,4,3");

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      filter = {
        from_block: { block_number: firstBlockCreated.block_number },
        to_block: { block_number: fifthBlockCreated.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 10,
        continuation_token: continuation_token,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      ({ events, continuation_token } = await providerRPC.getEvents(filter));

      expect(events.length).to.be.equal(0);
      expect(continuation_token).to.be.null;
    });

    it("returns expected events on correct filter with chunk size", async function () {
      // Send transactions
      const transactions = [];
      for (let i = 0; i < 5; i++) {
        transactions.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT
          )
        );
      }
      await context.createBlock(transactions);

      const filter = {
        from_block: "latest",
        to_block: "latest",
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 4,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const events = await providerRPC.getEvents(filter);
      expect(events.events.length).to.be.equal(4);
      expect(events.continuation_token).to.be.equal("0,1,3");
      for (let i = 0; i < 2; i++) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const tx: InvokeTransaction =
          await providerRPC.getTransactionByBlockIdAndIndex("latest", i);
        expect(
          validateAndParseAddress(events.events[2 * i].from_address)
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events.events[2 * i].transaction_hash).to.be.equal(
          tx.transaction_hash
        );
        expect(
          validateAndParseAddress(events.events[2 * i + 1].from_address)
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events.events[2 * i + 1].transaction_hash).to.be.equal(
          tx.transaction_hash
        );
      }
    });

    it("returns expected events on correct filter with continuation token", async function () {
      // Send transactions
      const transactions = [];
      for (let i = 0; i < 5; i++) {
        transactions.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT
          )
        );
      }
      await context.createBlock(transactions);

      const skip = 3;
      const filter = {
        from_block: "latest",
        to_block: "latest",
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 4,
        continuation_token: `0,${skip - 1},${3}`, // 3 events per transaction
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const events = await providerRPC.getEvents(filter);
      expect(events.events.length).to.be.equal(4);
      expect(events.continuation_token).to.be.null;
      for (let i = 0; i < 2; i++) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const tx: InvokeTransaction =
          await providerRPC.getTransactionByBlockIdAndIndex("latest", skip + i);
        expect(
          validateAndParseAddress(events.events[2 * i].from_address)
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events.events[2 * i].transaction_hash).to.be.equal(
          tx.transaction_hash
        );
        expect(
          validateAndParseAddress(events.events[2 * i + 1].from_address)
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events.events[2 * i + 1].transaction_hash).to.be.equal(
          tx.transaction_hash
        );
      }
    });

    it("returns expected events on correct filter with keys", async function () {
      // Send a transaction
      await context.createBlock(
        rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT
        )
      );

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const tx: InvokeTransaction =
        await providerRPC.getTransactionByBlockIdAndIndex("latest", 0);
      const block_hash_and_number = await providerRPC.getBlockHashAndNumber();
      const filter = {
        from_block: "latest",
        to_block: "latest",
        chunk_size: 1,
        keys: [[toHex(starknetKeccak("transaction_executed"))]],
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const events = await providerRPC.getEvents(filter);
      expect(events.events.length).to.be.equal(1);
      expect(events.continuation_token).to.be.equal("0,0,1");
      expect(events.events[0]).to.deep.equal({
        transaction_hash: tx.transaction_hash,
        block_hash: block_hash_and_number.block_hash,
        block_number: block_hash_and_number.block_number,
        from_address: cleanHex(ARGENT_CONTRACT_ADDRESS),
        keys: [toHex(starknetKeccak("transaction_executed"))],
        data: [tx.transaction_hash, "0x2", "0x1", "0x1"].map(cleanHex),
      });
    });
  });

  describe("Fix #551: Madara RPC doesn't handle 'pending' block id", async () => {
    it("should support 'pending' block id", async function () {
      const nonce = await providerRPC.getNonceForAddress(
        ARGENT_CONTRACT_ADDRESS,
        "pending"
      );
      expect(nonce).to.not.be.undefined;
    });

    it("should support 'latest' block id", async function () {
      const nonce = await providerRPC.getNonceForAddress(
        ARGENT_CONTRACT_ADDRESS,
        "latest"
      );
      expect(nonce).to.not.be.undefined;
    });
  });

  describe("state root", async () => {
    it("should return default when disabled", async function () {
      const latestBlock = await providerRPC.getBlock("latest");
      expect(latestBlock.new_root).to.eq("0x0");
    });
  });

  describe("Cairo 1 full flow", async () => {
    it("should deploy a Cairo 1 account", async () => {
      const CONSTRUCTOR_CALLDATA = ["0x123"];
      const accountAddress = hash.calculateContractAddressFromHash(
        SALT,
        CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH,
        CONSTRUCTOR_CALLDATA,
        0
      );

      const account = new Account(
        providerRPC,
        accountAddress,
        SIGNER_PRIVATE,
        "1"
      );

      // transfer native token to allow deployment
      await rpcTransfer(
        providerRPC,
        ARGENT_CONTRACT_NONCE,
        accountAddress,
        "0xfffffffffffffffffffffffff"
      );
      await jumpBlocks(context, 1);

      // deploy the account
      await account.deploySelf(
        {
          classHash: CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH,
          addressSalt: SALT,
          constructorCalldata: CONSTRUCTOR_CALLDATA,
        },
        { maxFee: "123456" }
      );
      await jumpBlocks(context, 1);

      expect(await providerRPC.getClassHashAt(accountAddress)).to.be.equal(
        CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH
      );
    });

    it("should declare and deploy erc20 contract then transfer some tokens", async () => {
      const account = new Account(
        providerRPC,
        CAIRO_1_ACCOUNT_CONTRACT,
        SIGNER_PRIVATE, // it's the no validate account
        "1"
      );
      // computed via: starknetjs 5.14.1
      const classHash =
        "0x4596fa4856bbf13f3448a376d607f8852148b0e6be4b958cde2ca8471a72ede";
      const res = await account.declare(
        {
          casm: ERC20_CAIRO_1_CASM,
          contract: ERC20_CAIRO_1_SIERRA,
        },
        {
          nonce: CAIRO_1_NO_VALIDATE_ACCOUNT.value,
          version: 1,
          maxFee: "123456",
        }
      );
      CAIRO_1_NO_VALIDATE_ACCOUNT.value += 1;
      await jumpBlocks(context, 1);

      const contractClassActual = await providerRPC.getClass(
        classHash,
        "latest"
      );
      // TODO: (Apoorv) make these checks better once we to_rpc_contract_class is fixed #775 and #790
      expect(contractClassActual).to.have.property("entry_points_by_type");
      expect(contractClassActual).to.have.property("sierra_program");
      expect(contractClassActual).to.have.property("contract_class_version");
      expect(contractClassActual).to.have.property("abi");
      expect(res.class_hash).to.be.eq(classHash);

      const deployRes = await account.deploy(
        {
          classHash,
          constructorCalldata: [
            1, // Token Name
            1, // Token Symbol
            1, // Token Decimals
            "0xffffffffffffffffffffffffffffffff", // Initial Supply
            "0xffffffffffffffffffffffffffffffff", // Initial Supply Cont { since u256 }
            CAIRO_1_ACCOUNT_CONTRACT, // Recipient
          ],
        },
        {
          maxFee: "123456",
          nonce: CAIRO_1_NO_VALIDATE_ACCOUNT.value,
          version: 1,
        }
      );
      CAIRO_1_NO_VALIDATE_ACCOUNT.value += 1;
      await jumpBlocks(context, 1);
      //  hex(get_storage_var_address("balances", 0x4))
      const balance = await providerRPC.getStorageAt(
        deployRes.contract_address[0],
        "0x617243ac31335377b9d26d1a6b02f47b419ad593e1ae67660dd27ec77635906",
        "latest"
      );
      expect(toHex(balance)).to.be.equal("0xffffffffffffffffffffffffffffffff");

      await account.execute(
        [
          {
            contractAddress: deployRes.contract_address[0],
            entrypoint: "transfer",
            calldata: [
              1, // recipient
              "0xffffffffffffffffffffffffffffffff", // amount low
              0, // amount high
            ],
          },
        ],
        undefined,
        {
          maxFee: "123456",
          nonce: CAIRO_1_NO_VALIDATE_ACCOUNT.value,
          version: 1,
        }
      );
      await jumpBlocks(context, 1);

      const balanceSender = await providerRPC.getStorageAt(
        deployRes.contract_address[0],
        //  hex(get_storage_var_address("balances", 0x4))
        "0x617243ac31335377b9d26d1a6b02f47b419ad593e1ae67660dd27ec77635906",
        "latest"
      );
      expect(toHex(balanceSender)).to.be.equal("0x0");
      const balanceRecipient = await providerRPC.getStorageAt(
        deployRes.contract_address[0],
        // hex(get_storage_var_address("balances", 0x1))
        "0x753d37842b9cfa00ee311ab2564951681d89ee4d5596e84e74030de35018c8a",
        "latest"
      );
      expect(toHex(balanceRecipient)).to.be.equal(
        "0xffffffffffffffffffffffffffffffff"
      );
    });
  });
});
