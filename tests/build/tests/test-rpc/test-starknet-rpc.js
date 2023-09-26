"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const tslib_1 = require("tslib");
require("@keep-starknet-strange/madara-api-augment");
const chai_1 = tslib_1.__importStar(require("chai"));
const deep_equal_in_any_order_1 = tslib_1.__importDefault(
  require("deep-equal-in-any-order"),
);
const fs_1 = tslib_1.__importDefault(require("fs"));
const starknet_1 = require("starknet");
const block_1 = require("../../util/block");
const setup_dev_tests_1 = require("../../util/setup-dev-tests");
const constants_1 = require("../constants");
const utils_1 = require("../../util/utils");
chai_1.default.use(deep_equal_in_any_order_1.default);
let ARGENT_CONTRACT_NONCE = { value: 0 };
(0, setup_dev_tests_1.describeDevMadara)("Starknet RPC", (context) => {
  let providerRPC;
  before(async function () {
    providerRPC = new starknet_1.RpcProvider({
      nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
      retries: 3,
    });
  });
  describe("getBlockhashAndNumber", () => {
    it("should not be undefined", async function () {
      const block = await providerRPC.getBlockHashAndNumber();
      (0, chai_1.expect)(block).to.not.be.undefined;
    });
  });
  describe("getBlockNumber", async () => {
    it("should return current block number", async function () {
      const blockNumber = await providerRPC.getBlockNumber();
      (0, chai_1.expect)(blockNumber).to.not.be.undefined;
      await (0, block_1.jumpBlocks)(context, 10);
      const blockNumber2 = await providerRPC.getBlockNumber();
      (0, chai_1.expect)(blockNumber2).to.be.equal(blockNumber + 10);
    });
  });
  describe("getBlockTransactionCount", async () => {
    it("should return 0 for latest block", async function () {
      const transactionCount = await providerRPC.getTransactionCount("latest");
      (0, chai_1.expect)(transactionCount).to.not.be.undefined;
      (0, chai_1.expect)(transactionCount).to.be.equal(0);
    });
  });
  describe("getNonce", async () => {
    it("should increase after a transaction", async function () {
      let nonce = await providerRPC.getNonceForAddress(
        constants_1.ARGENT_CONTRACT_ADDRESS,
        "latest",
      );
      await context.createBlock(
        (0, utils_1.rpcTransfer)(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          constants_1.ARGENT_CONTRACT_ADDRESS,
          constants_1.MINT_AMOUNT,
        ),
      );
      nonce = await providerRPC.getNonceForAddress(
        constants_1.ARGENT_CONTRACT_ADDRESS,
        "latest",
      );
      (0, chai_1.expect)(nonce).to.not.be.undefined;
      (0, chai_1.expect)((0, utils_1.toHex)(nonce)).to.be.equal(
        (0, utils_1.toHex)(ARGENT_CONTRACT_NONCE.value),
      );
    });
  });
  describe("call", async () => {
    it("should return calldata on return_result entrypoint", async function () {
      const call = await providerRPC.callContract(
        {
          contractAddress: constants_1.TEST_CONTRACT,
          entrypoint: "return_result",
          calldata: ["0x19"],
        },
        "latest",
      );
      (0, chai_1.expect)(call.result).to.contain("0x19");
    });
  });
  describe("getClassAt", async () => {
    it("should not be undefined", async function () {
      const contract_class = await providerRPC.getClassAt(
        constants_1.TEST_CONTRACT,
        "latest",
      );
      (0, chai_1.expect)(contract_class).to.not.be.undefined;
    });
  });
  describe("getClassHashAt", async () => {
    it("should return correct class hashes for account and test contract", async function () {
      const account_contract_class_hash = await providerRPC.getClassHashAt(
        constants_1.ACCOUNT_CONTRACT,
        "latest",
      );
      (0, chai_1.expect)(account_contract_class_hash).to.not.be.undefined;
      (0, chai_1.expect)(
        (0, starknet_1.validateAndParseAddress)(account_contract_class_hash),
      ).to.be.equal(constants_1.ACCOUNT_CONTRACT_CLASS_HASH);
      const test_contract_class_hash = await providerRPC.getClassHashAt(
        constants_1.TEST_CONTRACT,
        "latest",
      );
      (0, chai_1.expect)(test_contract_class_hash).to.not.be.undefined;
      (0, chai_1.expect)(
        (0, starknet_1.validateAndParseAddress)(test_contract_class_hash),
      ).to.be.equal(constants_1.TEST_CONTRACT_CLASS_HASH);
    });
    it("should raise with invalid block id", async () => {
      try {
        await providerRPC.getClassHashAt(constants_1.TEST_CONTRACT, "0x123");
      } catch (error) {
        (0, chai_1.expect)(error).to.be.instanceOf(starknet_1.LibraryError);
        (0, chai_1.expect)(error.message).to.equal("24: Block not found");
      }
    });
    it("should raise with invalid contract address", async () => {
      try {
        await providerRPC.getClassHashAt("0x123", "latest");
      } catch (error) {
        (0, chai_1.expect)(error).to.be.instanceOf(starknet_1.LibraryError);
        (0, chai_1.expect)(error.message).to.equal("20: Contract not found");
      }
    });
  });
  describe("syncing", async () => {
    it("should return starting setup and current_block info", async function () {
      await (0, block_1.jumpBlocks)(context, 10);
      const status = await providerRPC.getSyncingStats();
      const current_block = await providerRPC.getBlockHashAndNumber();
      (0, chai_1.expect)(status["starting_block_num"]).to.be.equal("0x0");
      (0, chai_1.expect)(parseInt(status["current_block_num"])).to.be.equal(
        current_block["block_number"],
      );
      (0, chai_1.expect)(parseInt(status["highest_block_num"])).to.be.equal(
        current_block["block_number"],
      );
      (0, chai_1.expect)(status["starting_block_hash"]).to.contain("0x49ee");
      (0, chai_1.expect)(status["current_block_hash"]).to.be.equal(
        current_block["block_hash"],
      );
      (0, chai_1.expect)(status["highest_block_hash"]).to.be.equal(
        current_block["block_hash"],
      );
    });
  });
  describe("getClass", async () => {
    it("should not be undefined", async function () {
      const contract_class = await providerRPC.getClass(
        constants_1.TOKEN_CLASS_HASH,
        "latest",
      );
      (0, chai_1.expect)(contract_class).to.not.be.undefined;
    });
  });
  describe("getBlockWithTxHashes", async () => {
    it("should returns transactions", async function () {
      await context.createBlock(
        (0, utils_1.rpcTransfer)(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          constants_1.ARGENT_CONTRACT_ADDRESS,
          constants_1.MINT_AMOUNT,
        ),
      );
      const blockWithTxHashes =
        await providerRPC.getBlockWithTxHashes("latest");
      (0, chai_1.expect)(blockWithTxHashes).to.not.be.undefined;
      (0, chai_1.expect)(blockWithTxHashes.status).to.be.equal(
        "ACCEPTED_ON_L2",
      );
      (0, chai_1.expect)(blockWithTxHashes.transactions.length).to.be.equal(1);
    });
    it("should throws block not found error", async function () {
      await providerRPC.getBlockWithTxHashes("0x123").catch((error) => {
        (0, chai_1.expect)(error).to.be.instanceOf(starknet_1.LibraryError);
        (0, chai_1.expect)(error.message).to.equal("24: Block not found");
      });
    });
  });
  describe("getBlockWithTxs", async () => {
    it("should returns transactions", async function () {
      await context.createBlock(
        (0, utils_1.rpcTransfer)(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          constants_1.ARGENT_CONTRACT_ADDRESS,
          constants_1.MINT_AMOUNT,
        ),
      );
      const blockHash = await providerRPC.getBlockHashAndNumber();
      await (0, block_1.jumpBlocks)(context, 10);
      const blockWithTxHashes = await providerRPC.getBlockWithTxs(
        blockHash.block_hash,
      );
      const tx = blockWithTxHashes.transactions[0];
      (0, chai_1.expect)(blockWithTxHashes).to.not.be.undefined;
      (0, chai_1.expect)(blockWithTxHashes.transactions.length).to.be.equal(1);
      (0, chai_1.expect)(tx.type).to.be.equal("INVOKE");
      (0, chai_1.expect)(tx.sender_address).to.be.equal(
        (0, utils_1.toHex)(constants_1.ARGENT_CONTRACT_ADDRESS),
      );
      (0, chai_1.expect)(tx.calldata).to.deep.equal(
        [
          1,
          constants_1.FEE_TOKEN_ADDRESS,
          starknet_1.hash.getSelectorFromName("transfer"),
          0,
          3,
          3,
          constants_1.ARGENT_CONTRACT_ADDRESS,
          constants_1.MINT_AMOUNT,
          0,
        ].map(utils_1.toHex),
      );
    });
    it("should throws block not found error", async function () {
      await providerRPC.getBlockWithTxHashes("0x123").catch((error) => {
        (0, chai_1.expect)(error).to.be.instanceOf(starknet_1.LibraryError);
        (0, chai_1.expect)(error.message).to.equal("24: Block not found");
      });
    });
    it("should returns empty block", async function () {
      await context.createBlock(undefined, {
        parentHash: undefined,
        finalize: true,
      });
      const latestBlock = await providerRPC.getBlockWithTxHashes("latest");
      (0, chai_1.expect)(latestBlock).to.not.be.undefined;
      (0, chai_1.expect)(latestBlock.status).to.be.equal("ACCEPTED_ON_L2");
      (0, chai_1.expect)(latestBlock.transactions.length).to.be.equal(0);
    });
  });
  describe("getBlockWithTxHashes", async () => {
    it("should return an empty block", async function () {
      await context.createBlock(undefined, {
        parentHash: undefined,
        finalize: true,
      });
      const latestBlock = await providerRPC.getBlockWithTxHashes("latest");
      (0, chai_1.expect)(latestBlock).to.not.be.undefined;
      (0, chai_1.expect)(latestBlock.status).to.be.equal("ACCEPTED_ON_L2");
      (0, chai_1.expect)(latestBlock.transactions.length).to.be.equal(0);
    });
  });
  describe("getStorageAt", async () => {
    it("should return value from the fee contract storage", async function () {
      const value = await providerRPC.getStorageAt(
        constants_1.FEE_TOKEN_ADDRESS,
        "0x1d8bbc4f93f5ab9858f6c0c0de2769599fb97511503d5bf2872ef6846f2146f",
        "latest",
      );
      (0, chai_1.expect)((0, utils_1.toHex)(value)).to.be.equal(
        "0xffffffffffffffffffffffffffffffff",
      );
    });
    it("should return 0 if the storage slot is not set", async function () {
      const value = await providerRPC.getStorageAt(
        constants_1.FEE_TOKEN_ADDRESS,
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        "latest",
      );
      (0, chai_1.expect)(value).to.be.equal("0");
    });
    it("should raise if the contract does not exist", async function () {
      try {
        await providerRPC.getStorageAt(
          "0x0000000000000000000000000000000000000000000000000000000000000000",
          "0x0000000000000000000000000000000000000000000000000000000000000000",
          "latest",
        );
      } catch (error) {
        (0, chai_1.expect)(error).to.be.instanceOf(starknet_1.LibraryError);
        (0, chai_1.expect)(error.message).to.equal("20: Contract not found");
      }
    });
  });
  describe("chainId", async () => {
    it("should return the correct value", async function () {
      const chainId = await providerRPC.getChainId();
      (0, chai_1.expect)(chainId).to.not.be.undefined;
      (0, chai_1.expect)(chainId).to.be.equal(
        constants_1.CHAIN_ID_STARKNET_TESTNET,
      );
    });
  });
  describe("getTransactionByBlockIdAndIndex", async () => {
    it("should returns transactions", async function () {
      await context.createBlock(
        (0, utils_1.rpcTransfer)(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          constants_1.ARGENT_CONTRACT_ADDRESS,
          constants_1.MINT_AMOUNT,
        ),
      );
      const getTransactionByBlockIdAndIndexResponse =
        await providerRPC.getTransactionByBlockIdAndIndex("latest", 0);
      (0, chai_1.expect)(getTransactionByBlockIdAndIndexResponse).to.not.be
        .undefined;
    });
    it("should throws block not found error", async function () {
      await providerRPC
        .getTransactionByBlockIdAndIndex("0x123", 2)
        .catch((error) => {
          (0, chai_1.expect)(error).to.be.instanceOf(starknet_1.LibraryError);
          (0, chai_1.expect)(error.message).to.equal("24: Block not found");
        });
    });
    it("should throws invalid transaction index error", async function () {
      await context.createBlock(undefined, {
        parentHash: undefined,
        finalize: true,
      });
      const latestBlockCreated = await providerRPC.getBlockHashAndNumber();
      await providerRPC
        .getTransactionByBlockIdAndIndex(latestBlockCreated.block_hash, 2)
        .catch((error) => {
          (0, chai_1.expect)(error).to.be.instanceOf(starknet_1.LibraryError);
          (0, chai_1.expect)(error.message).to.equal(
            "27: Invalid transaction index in a block",
          );
        });
    });
  });
  describe("addInvokeTransaction", async () => {
    it("should invoke successfully", async function () {
      const keyPair = starknet_1.ec.getKeyPair(constants_1.SIGNER_PRIVATE);
      const account = new starknet_1.Account(
        providerRPC,
        constants_1.ARGENT_CONTRACT_ADDRESS,
        keyPair,
      );
      const resp = await account.execute(
        {
          contractAddress: constants_1.TEST_CONTRACT,
          entrypoint: "test_storage_var",
          calldata: [],
        },
        undefined,
        {
          nonce: "0",
          maxFee: "123456",
        },
      );
      await (0, block_1.jumpBlocks)(context, 1);
      (0, chai_1.expect)(resp).to.not.be.undefined;
      (0, chai_1.expect)(resp.transaction_hash).to.contain("0x");
    });
    it("should raise with unknown entrypoint", async function () {
      const keyPair = starknet_1.ec.getKeyPair(constants_1.SIGNER_PRIVATE);
      const account = new starknet_1.Account(
        providerRPC,
        constants_1.ARGENT_CONTRACT_ADDRESS,
        keyPair,
      );
      try {
        await account.execute(
          {
            contractAddress: constants_1.TEST_CONTRACT,
            entrypoint: "test_storage_var_WRONG",
            calldata: [],
          },
          undefined,
          {
            nonce: "0",
            maxFee: "123456",
          },
        );
      } catch (error) {
        (0, chai_1.expect)(error).to.be.instanceOf(starknet_1.LibraryError);
        (0, chai_1.expect)(error.message).to.equal("40: Contract error");
      }
    });
  });
  describe("addDeployAccountTransaction", async () => {
    it("should deploy successfully", async function () {
      const selector = starknet_1.hash.getSelectorFromName("initialize");
      const calldata = [
        constants_1.ARGENT_ACCOUNT_CLASS_HASH,
        selector,
        2,
        constants_1.SIGNER_PUBLIC,
        0,
      ];
      const deployedContractAddress =
        starknet_1.hash.calculateContractAddressFromHash(
          constants_1.SALT,
          constants_1.ARGENT_PROXY_CLASS_HASH,
          calldata,
          0,
        );
      const invocationDetails = {
        nonce: "0x0",
        maxFee: "0x1111111111111111111111",
        version: "0x1",
      };
      const txHash = starknet_1.hash.calculateDeployAccountTransactionHash(
        deployedContractAddress,
        constants_1.ARGENT_PROXY_CLASS_HASH,
        calldata,
        constants_1.SALT,
        invocationDetails.version,
        invocationDetails.maxFee,
        starknet_1.constants.StarknetChainId.TESTNET,
        invocationDetails.nonce,
      );
      const keyPair = starknet_1.ec.getKeyPair(constants_1.SIGNER_PRIVATE);
      const signature = starknet_1.ec.sign(keyPair, txHash);
      const txDeployAccount = {
        signature: signature,
        contractAddress: deployedContractAddress,
        addressSalt: constants_1.SALT,
        classHash: constants_1.ARGENT_PROXY_CLASS_HASH,
        constructorCalldata: calldata,
      };
      await providerRPC.deployAccountContract(
        txDeployAccount,
        invocationDetails,
      );
      await (0, block_1.createAndFinalizeBlock)(context.polkadotApi);
      const accountContractClass = await providerRPC.getClassHashAt(
        deployedContractAddress,
      );
      (0, chai_1.expect)(
        (0, starknet_1.validateAndParseAddress)(accountContractClass),
      ).to.be.equal(constants_1.ARGENT_PROXY_CLASS_HASH);
    });
  });
  describe("estimateFee", async () => {
    it("should estimate fee to 0", async function () {
      const tx = {
        contractAddress: constants_1.ACCOUNT_CONTRACT,
        calldata: [
          constants_1.TEST_CONTRACT,
          "0x36fa6de2810d05c3e1a0ebe23f60b9c2f4629bbead09e5a9704e1c5632630d5",
          "0x0",
        ],
      };
      const nonce = await providerRPC.getNonceForAddress(
        constants_1.ACCOUNT_CONTRACT,
        "latest",
      );
      const txDetails = {
        nonce: nonce,
        version: "0x1",
      };
      const fee_estimate = await providerRPC.getEstimateFee(
        tx,
        txDetails,
        "latest",
      );
      (0, chai_1.expect)(
        fee_estimate.overall_fee.cmp((0, utils_1.toBN)(0)),
      ).to.be.equal(1);
      (0, chai_1.expect)(
        fee_estimate.gas_consumed.cmp((0, utils_1.toBN)(0)),
      ).to.be.equal(1);
    });
    it("should raise if contract does not exist", async function () {
      const tx = {
        contractAddress: constants_1.ACCOUNT_CONTRACT,
        calldata: [
          "0x000000000000000000000000000000000000000000000000000000000000DEAD",
          "0x36fa6de2810d05c3e1a0ebe23f60b9c2f4629bbead09e5a9704e1c5632630d5",
          "0x0",
        ],
      };
      const nonce = await providerRPC.getNonceForAddress(
        constants_1.ACCOUNT_CONTRACT,
        "latest",
      );
      const txDetails = {
        nonce: nonce,
        version: "0x1",
      };
      try {
        await providerRPC.getEstimateFee(tx, txDetails, "latest");
      } catch (error) {
        (0, chai_1.expect)(error).to.be.instanceOf(starknet_1.LibraryError);
        (0, chai_1.expect)(error.message).to.equal("40: Contract error");
      }
    });
  });
  describe("addDeclareTransaction", async () => {
    it("should return hash starting with 0x", async function () {
      const nonce = await providerRPC.getNonceForAddress(
        constants_1.ARGENT_CONTRACT_ADDRESS,
        "latest",
      );
      const keyPair = starknet_1.ec.getKeyPair(constants_1.SIGNER_PRIVATE);
      const account = new starknet_1.Account(
        providerRPC,
        constants_1.ARGENT_CONTRACT_ADDRESS,
        keyPair,
      );
      const contract = fs_1.default
        .readFileSync("./contracts/compiled/erc20.json")
        .toString();
      const resp = await account.declare(
        {
          classHash:
            "0x057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95",
          contract,
        },
        { nonce, version: 1, maxFee: "123456" },
      );
      await (0, block_1.jumpBlocks)(context, 1);
      (0, chai_1.expect)(resp).to.not.be.undefined;
      (0, chai_1.expect)(resp.transaction_hash).to.contain("0x");
      await (0, block_1.jumpBlocks)(context, 10);
    });
  });
  describe("pendingTransactions", async () => {
    it("should return all the starknet invoke transactions", async function () {
      await (0, utils_1.rpcTransfer)(
        providerRPC,
        ARGENT_CONTRACT_NONCE,
        constants_1.ARGENT_CONTRACT_ADDRESS,
        constants_1.MINT_AMOUNT,
      );
      const txs = await providerRPC.getPendingTransactions();
      (0, chai_1.expect)(txs.length).equals(1);
      (0, chai_1.expect)(txs[0]).to.include({ type: "INVOKE" });
      (0, chai_1.expect)(txs[0]).that.includes.all.keys([
        "transaction_hash",
        "max_fee",
        "version",
        "signature",
        "nonce",
        "type",
        "sender_address",
        "calldata",
      ]);
      await (0, block_1.jumpBlocks)(context, 10);
    });
    it("should return all starknet declare transactions", async function () {
      const keyPair = starknet_1.ec.getKeyPair(constants_1.SIGNER_PRIVATE);
      const nonce = await providerRPC.getNonceForAddress(
        constants_1.ARGENT_CONTRACT_ADDRESS,
        "latest",
      );
      const account = new starknet_1.Account(
        providerRPC,
        constants_1.ARGENT_CONTRACT_ADDRESS,
        keyPair,
      );
      const contract = fs_1.default
        .readFileSync("./contracts/compiled/erc20.json")
        .toString();
      await account.declare(
        {
          classHash:
            "0x057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da96",
          contract,
        },
        { nonce, version: 1, maxFee: "123456" },
      );
      const txs = await providerRPC.getPendingTransactions();
      (0, chai_1.expect)(txs.length).equals(1);
      (0, chai_1.expect)(txs[0]).to.include({ type: "DECLARE" });
      (0, chai_1.expect)(txs[0]).that.includes.all.keys([
        "sender_address",
        "class_hash",
        "max_fee",
        "nonce",
        "signature",
        "transaction_hash",
        "type",
        "version",
      ]);
      await (0, block_1.jumpBlocks)(context, 10);
    });
    it("should return all starknet deploy_account transactions", async function () {
      const selector = starknet_1.hash.getSelectorFromName("initialize");
      const calldata = [
        constants_1.ARGENT_ACCOUNT_CLASS_HASH,
        selector,
        2,
        constants_1.SIGNER_PUBLIC,
        0,
      ];
      const deployedContractAddress =
        starknet_1.hash.calculateContractAddressFromHash(
          constants_1.SALT,
          constants_1.ARGENT_PROXY_CLASS_HASH,
          calldata,
          0,
        );
      const invocationDetails = {
        nonce: "0x0",
        maxFee: "0x1111111111111111111111",
        version: "0x1",
      };
      const txHash = starknet_1.hash.calculateDeployAccountTransactionHash(
        deployedContractAddress,
        constants_1.ARGENT_PROXY_CLASS_HASH,
        calldata,
        constants_1.SALT,
        invocationDetails.version,
        invocationDetails.maxFee,
        starknet_1.constants.StarknetChainId.TESTNET,
        invocationDetails.nonce,
      );
      const keyPair = starknet_1.ec.getKeyPair(constants_1.SIGNER_PRIVATE);
      const signature = starknet_1.ec.sign(keyPair, txHash);
      const txDeployAccount = {
        signature: signature,
        contractAddress: deployedContractAddress,
        addressSalt: constants_1.SALT,
        classHash: constants_1.ARGENT_PROXY_CLASS_HASH,
        constructorCalldata: calldata,
      };
      await providerRPC.deployAccountContract(
        txDeployAccount,
        invocationDetails,
      );
      const txs = await providerRPC.getPendingTransactions();
      (0, chai_1.expect)(txs.length).equals(1);
      (0, chai_1.expect)(txs[0]).to.include({ type: "DEPLOY_ACCOUNT" });
      (0, chai_1.expect)(txs[0]).that.includes.all.keys([
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
      await (0, block_1.jumpBlocks)(context, 10);
    });
  });
  describe("getTransactionByHash", () => {
    it("should return a transaction", async function () {
      await (0, block_1.createAndFinalizeBlock)(context.polkadotApi);
      const b = await context.createBlock(
        (0, utils_1.rpcTransfer)(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          constants_1.ARGENT_CONTRACT_ADDRESS,
          constants_1.MINT_AMOUNT,
        ),
        {
          finalize: true,
        },
      );
      const r = await providerRPC.getTransactionByHash(b.result.hash);
      (0, chai_1.expect)(r).to.not.be.undefined;
    });
    it("should return transaction hash not found", async function () {
      await context.createBlock(
        (0, utils_1.rpcTransfer)(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          constants_1.ARGENT_CONTRACT_ADDRESS,
          constants_1.MINT_AMOUNT,
        ),
      );
      try {
        await providerRPC.getTransactionByHash("0x1234");
      } catch (error) {
        (0, chai_1.expect)(error).to.be.instanceOf(starknet_1.LibraryError);
        (0, chai_1.expect)(error.message).to.equal(
          "25: Transaction hash not found",
        );
      }
    });
    it("should return transaction hash not found when a transaction is in the pool", async function () {
      await (0, block_1.createAndFinalizeBlock)(context.polkadotApi);
      const b = await (0, utils_1.rpcTransfer)(
        providerRPC,
        ARGENT_CONTRACT_NONCE,
        constants_1.ARGENT_CONTRACT_ADDRESS,
        constants_1.MINT_AMOUNT,
      );
      try {
        await providerRPC.getTransactionByHash(b.transaction_hash);
      } catch (error) {
        (0, chai_1.expect)(error).to.be.instanceOf(starknet_1.LibraryError);
        (0, chai_1.expect)(error.message).to.equal(
          "25: Transaction hash not found",
        );
      }
    });
  });
  describe("getTransactionReceipt", () => {
    it("should return a receipt", async function () {
      await (0, block_1.createAndFinalizeBlock)(context.polkadotApi);
      const b = await context.createBlock(
        (0, utils_1.rpcTransfer)(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          constants_1.ARGENT_CONTRACT_ADDRESS,
          constants_1.MINT_AMOUNT,
        ),
        {
          finalize: true,
        },
      );
      const r = await providerRPC.getTransactionReceipt(b.result.hash);
      (0, chai_1.expect)(r).to.not.be.undefined;
    });
    it("should return transaction hash not found", async function () {
      await context.createBlock(
        (0, utils_1.rpcTransfer)(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          constants_1.ARGENT_CONTRACT_ADDRESS,
          constants_1.MINT_AMOUNT,
        ),
      );
      try {
        await providerRPC.getTransactionReceipt("0x1234");
      } catch (error) {
        (0, chai_1.expect)(error).to.be.instanceOf(starknet_1.LibraryError);
        (0, chai_1.expect)(error.message).to.equal(
          "25: Transaction hash not found",
        );
      }
    });
  });
  describe("getEvents", function () {
    it("should fail on invalid continuation token", async function () {
      const filter = {
        from_block: { block_number: 0 },
        to_block: { block_number: 1 },
        address: constants_1.FEE_TOKEN_ADDRESS,
        chunk_size: 1,
        continuation_token: "0xabdel",
      };
      try {
        await providerRPC.getEvents(filter);
      } catch (error) {
        (0, chai_1.expect)(error).to.be.instanceOf(starknet_1.LibraryError);
        (0, chai_1.expect)(error.message).to.equal(
          "33: The supplied continuation token is invalid or unknown",
        );
      }
    });
    it("should fail on chunk size too big", async function () {
      const filter = {
        from_block: { block_number: 0 },
        to_block: { block_number: 1 },
        address: constants_1.FEE_TOKEN_ADDRESS,
        chunk_size: 1001,
      };
      try {
        await providerRPC.getEvents(filter);
      } catch (error) {
        (0, chai_1.expect)(error).to.be.instanceOf(starknet_1.LibraryError);
        (0, chai_1.expect)(error.message).to.equal(
          "31: Requested page size is too big",
        );
      }
    });
    it("should fail on keys too big", async function () {
      const filter = {
        from_block: { block_number: 0 },
        to_block: { block_number: 1 },
        address: constants_1.FEE_TOKEN_ADDRESS,
        chunk_size: 1,
        keys: Array(101).fill(["0x0"]),
      };
      try {
        await providerRPC.getEvents(filter);
      } catch (error) {
        (0, chai_1.expect)(error).to.be.instanceOf(starknet_1.LibraryError);
        (0, chai_1.expect)(error.message).to.equal(
          "34: Too many keys provided in a filter",
        );
      }
    });
    it("returns expected events on correct filter", async function () {
      await context.createBlock(
        (0, utils_1.rpcTransfer)(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          constants_1.ARGENT_CONTRACT_ADDRESS,
          constants_1.MINT_AMOUNT,
        ),
      );
      const tx = await providerRPC.getTransactionByBlockIdAndIndex("latest", 0);
      const block_hash_and_number = await providerRPC.getBlockHashAndNumber();
      const filter = {
        from_block: "latest",
        to_block: "latest",
        address: constants_1.FEE_TOKEN_ADDRESS,
        chunk_size: 10,
      };
      const events = await providerRPC.getEvents(filter);
      (0, chai_1.expect)(events.events.length).to.be.equal(2);
      (0, chai_1.expect)(events.continuation_token).to.be.null;
      for (const event of events.events) {
        (0, chai_1.expect)(
          (0, starknet_1.validateAndParseAddress)(event.from_address),
        ).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
        (0, chai_1.expect)(event.transaction_hash).to.be.equal(
          tx.transaction_hash,
        );
      }
      const transfer_event = events.events[0];
      (0, chai_1.expect)(transfer_event).to.deep.equal({
        transaction_hash: tx.transaction_hash,
        block_hash: block_hash_and_number.block_hash,
        block_number: block_hash_and_number.block_number,
        from_address: (0, utils_1.cleanHex)(constants_1.FEE_TOKEN_ADDRESS),
        keys: [(0, utils_1.toHex)((0, utils_1.starknetKeccak)("Transfer"))],
        data: [
          constants_1.ARGENT_CONTRACT_ADDRESS,
          constants_1.ARGENT_CONTRACT_ADDRESS,
          constants_1.MINT_AMOUNT,
          "0x0",
        ].map(utils_1.cleanHex),
      });
      const fee_event = events.events[1];
      (0, chai_1.expect)(fee_event).to.deep.equal({
        transaction_hash: tx.transaction_hash,
        block_hash: block_hash_and_number.block_hash,
        block_number: block_hash_and_number.block_number,
        from_address: (0, utils_1.cleanHex)(constants_1.FEE_TOKEN_ADDRESS),
        keys: [(0, utils_1.toHex)((0, utils_1.starknetKeccak)("Transfer"))],
        data: [
          constants_1.ARGENT_CONTRACT_ADDRESS,
          constants_1.ARGENT_CONTRACT_ADDRESS,
          "0x19e1a",
          "0x0",
        ].map(utils_1.cleanHex),
      });
    });
    it("returns expected events on correct filter with chunk size", async function () {
      const transactions = [];
      for (let i = 0; i < 5; i++) {
        transactions.push(
          (0, utils_1.rpcTransfer)(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            constants_1.ARGENT_CONTRACT_ADDRESS,
            constants_1.MINT_AMOUNT,
          ),
        );
      }
      await context.createBlock(transactions);
      const filter = {
        from_block: "latest",
        to_block: "latest",
        address: constants_1.FEE_TOKEN_ADDRESS,
        chunk_size: 4,
      };
      const events = await providerRPC.getEvents(filter);
      (0, chai_1.expect)(events.events.length).to.be.equal(4);
      (0, chai_1.expect)(
        (0, utils_1.toHex)(events.continuation_token),
      ).to.be.equal("0x6");
      for (let i = 0; i < 2; i++) {
        const tx = await providerRPC.getTransactionByBlockIdAndIndex(
          "latest",
          i,
        );
        (0, chai_1.expect)(
          (0, starknet_1.validateAndParseAddress)(
            events.events[2 * i].from_address,
          ),
        ).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
        (0, chai_1.expect)(events.events[2 * i].transaction_hash).to.be.equal(
          tx.transaction_hash,
        );
        (0, chai_1.expect)(
          (0, starknet_1.validateAndParseAddress)(
            events.events[2 * i + 1].from_address,
          ),
        ).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
        (0, chai_1.expect)(
          events.events[2 * i + 1].transaction_hash,
        ).to.be.equal(tx.transaction_hash);
      }
    });
    it("returns expected events on correct filter with continuation token", async function () {
      const transactions = [];
      for (let i = 0; i < 5; i++) {
        transactions.push(
          (0, utils_1.rpcTransfer)(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            constants_1.ARGENT_CONTRACT_ADDRESS,
            constants_1.MINT_AMOUNT,
          ),
        );
      }
      await context.createBlock(transactions);
      const skip = 3;
      const filter = {
        from_block: "latest",
        to_block: "latest",
        address: constants_1.FEE_TOKEN_ADDRESS,
        chunk_size: 4,
        continuation_token: (skip * 3).toString(),
      };
      const events = await providerRPC.getEvents(filter);
      (0, chai_1.expect)(events.events.length).to.be.equal(4);
      (0, chai_1.expect)(events.continuation_token).to.be.null;
      for (let i = 0; i < 2; i++) {
        const tx = await providerRPC.getTransactionByBlockIdAndIndex(
          "latest",
          skip + i,
        );
        (0, chai_1.expect)(
          (0, starknet_1.validateAndParseAddress)(
            events.events[2 * i].from_address,
          ),
        ).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
        (0, chai_1.expect)(events.events[2 * i].transaction_hash).to.be.equal(
          tx.transaction_hash,
        );
        (0, chai_1.expect)(
          (0, starknet_1.validateAndParseAddress)(
            events.events[2 * i + 1].from_address,
          ),
        ).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
        (0, chai_1.expect)(
          events.events[2 * i + 1].transaction_hash,
        ).to.be.equal(tx.transaction_hash);
      }
    });
    it("returns expected events on correct filter with keys", async function () {
      await context.createBlock(
        (0, utils_1.rpcTransfer)(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          constants_1.ARGENT_CONTRACT_ADDRESS,
          constants_1.MINT_AMOUNT,
        ),
      );
      const tx = await providerRPC.getTransactionByBlockIdAndIndex("latest", 0);
      const block_hash_and_number = await providerRPC.getBlockHashAndNumber();
      const filter = {
        from_block: "latest",
        to_block: "latest",
        chunk_size: 1,
        keys: [
          [
            (0, utils_1.toHex)(
              (0, utils_1.starknetKeccak)("transaction_executed"),
            ),
          ],
        ],
      };
      const events = await providerRPC.getEvents(filter);
      (0, chai_1.expect)(events.events.length).to.be.equal(1);
      (0, chai_1.expect)(
        (0, utils_1.toHex)(events.continuation_token),
      ).to.be.equal("0x1");
      (0, chai_1.expect)(events.events[0]).to.deep.equal({
        transaction_hash: tx.transaction_hash,
        block_hash: block_hash_and_number.block_hash,
        block_number: block_hash_and_number.block_number,
        from_address: (0, utils_1.cleanHex)(
          constants_1.ARGENT_CONTRACT_ADDRESS,
        ),
        keys: [
          (0, utils_1.toHex)(
            (0, utils_1.starknetKeccak)("transaction_executed"),
          ),
        ],
        data: [tx.transaction_hash, "0x2", "0x1", "0x1"].map(utils_1.cleanHex),
      });
    });
  });
});
//# sourceMappingURL=test-starknet-rpc.js.map
