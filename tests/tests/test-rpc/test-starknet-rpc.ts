import "@keep-starknet-strange/madara-api-augment";
import chai, { expect } from "chai";
import deepEqualInAnyOrder from "deep-equal-in-any-order";
import fs from "fs";
import {
  Account,
  LibraryError,
  RpcProvider,
  constants,
  ec,
  hash,
  validateAndParseAddress,
} from "starknet";
import { createAndFinalizeBlock, jumpBlocks } from "../../util/block";
import { describeDevMadara } from "../../util/setup-dev-tests";
import {
  ACCOUNT_CONTRACT,
  ACCOUNT_CONTRACT_CLASS_HASH,
  ARGENT_ACCOUNT_CLASS_HASH,
  ARGENT_CONTRACT_ADDRESS,
  ARGENT_PROXY_CLASS_HASH,
  CHAIN_ID_STARKNET_TESTNET,
  FEE_TOKEN_ADDRESS,
  MINT_AMOUNT,
  SALT,
  SIGNER_PRIVATE,
  SIGNER_PUBLIC,
  TEST_CONTRACT,
  TEST_CONTRACT_CLASS_HASH,
  TOKEN_CLASS_HASH,
} from "../constants";
import {
  toHex,
  toBN,
  rpcTransfer,
  starknetKeccak,
  cleanHex,
} from "../../util/utils";
import { Block, InvokeTransaction } from "./types";

chai.use(deepEqualInAnyOrder);

// keep "let" over "const" as the nonce is passed by reference
// to abstract the increment
// eslint-disable-next-line prefer-const
let ARGENT_CONTRACT_NONCE = { value: 0 };

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
      expect(toHex(nonce)).to.be.equal(toHex(ARGENT_CONTRACT_NONCE.value));
    });
  });

  describe("call", async () => {
    it("should return calldata on return_result entrypoint", async function () {
      const call = await providerRPC.callContract(
        {
          contractAddress: TEST_CONTRACT,
          entrypoint: "return_result",
          calldata: ["0x19"],
        },
        "latest"
      );

      expect(call.result).to.contain("0x19");
    });
  });

  describe("getClassAt", async () => {
    it("should not be undefined", async function () {
      const contract_class = await providerRPC.getClassAt(
        TEST_CONTRACT,
        "latest"
      );

      expect(contract_class).to.not.be.undefined;
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
        TEST_CONTRACT,
        "latest"
      );

      expect(test_contract_class_hash).to.not.be.undefined;
      expect(validateAndParseAddress(test_contract_class_hash)).to.be.equal(
        TEST_CONTRACT_CLASS_HASH
      );
    });

    it("should raise with invalid block id", async () => {
      // Invalid block id
      try {
        await providerRPC.getClassHashAt(TEST_CONTRACT, "0x123");
      } catch (error) {
        expect(error).to.be.instanceOf(LibraryError);
        expect(error.message).to.equal("24: Block not found");
      }
    });

    it("should raise with invalid contract address", async () => {
      // Invalid/un-deployed contract address
      try {
        await providerRPC.getClassHashAt("0x123", "latest");
      } catch (error) {
        expect(error).to.be.instanceOf(LibraryError);
        expect(error.message).to.equal("20: Contract not found");
      }
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

      // the starknet block hash for number 0 starts with "0x49ee" with this test setup
      expect(status["starting_block_hash"]).to.contain("0x49ee");
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
    it("should not be undefined", async function () {
      const contract_class = await providerRPC.getClass(
        TOKEN_CLASS_HASH,
        "latest"
      );
      expect(contract_class).to.not.be.undefined;
    });
  });

  describe("getBlockWithTxHashes", async () => {
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

    it("should throws block not found error", async function () {
      await providerRPC.getBlockWithTxHashes("0x123").catch((error) => {
        expect(error).to.be.instanceOf(LibraryError);
        expect(error.message).to.equal("24: Block not found");
      });
    });
  });

  describe("getBlockWithTxs", async () => {
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

    it("should throws block not found error", async function () {
      await providerRPC.getBlockWithTxHashes("0x123").catch((error) => {
        expect(error).to.be.instanceOf(LibraryError);
        expect(error.message).to.equal("24: Block not found");
      });
    });

    it("should returns empty block", async function () {
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
  });

  describe("getStorageAt", async () => {
    it("should return value from the fee contract storage", async function () {
      const value = await providerRPC.getStorageAt(
        FEE_TOKEN_ADDRESS,
        // ERC20_balances(0x02).low
        "0x1d8bbc4f93f5ab9858f6c0c0de2769599fb97511503d5bf2872ef6846f2146f",
        "latest"
      );
      // fees were paid du to the transfer in the previous test so the value is still u128::MAX
      expect(toHex(value)).to.be.equal("0xffffffffffffffffffffffffffffffff");
    });

    it("should return 0 if the storage slot is not set", async function () {
      const value = await providerRPC.getStorageAt(
        FEE_TOKEN_ADDRESS,
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        "latest"
      );
      expect(value).to.be.equal("0");
    });

    it("should raise if the contract does not exist", async function () {
      try {
        await providerRPC.getStorageAt(
          "0x0000000000000000000000000000000000000000000000000000000000000000",
          "0x0000000000000000000000000000000000000000000000000000000000000000",
          "latest"
        );
      } catch (error) {
        expect(error).to.be.instanceOf(LibraryError);
        expect(error.message).to.equal("20: Contract not found");
      }
    });
  });

  describe("chainId", async () => {
    it("should return the correct value", async function () {
      const chainId = await providerRPC.getChainId();

      expect(chainId).to.not.be.undefined;
      expect(chainId).to.be.equal(CHAIN_ID_STARKNET_TESTNET);
    });
  });

  describe("getTransactionByBlockIdAndIndex", async () => {
    it("should returns transactions", async function () {
      // Send a transaction
      await context.createBlock(
        rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT
        )
      );

      const getTransactionByBlockIdAndIndexResponse =
        await providerRPC.getTransactionByBlockIdAndIndex("latest", 0);

      expect(getTransactionByBlockIdAndIndexResponse).to.not.be.undefined;
    });

    it("should throws block not found error", async function () {
      await providerRPC
        .getTransactionByBlockIdAndIndex("0x123", 2)
        .catch((error) => {
          expect(error).to.be.instanceOf(LibraryError);
          expect(error.message).to.equal("24: Block not found");
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
          expect(error).to.be.instanceOf(LibraryError);
          expect(error.message).to.equal(
            "27: Invalid transaction index in a block"
          );
        });
    });
  });

  describe("addInvokeTransaction", async () => {
    it("should invoke successfully", async function () {
      const keyPair = ec.getKeyPair(SIGNER_PRIVATE);
      const account = new Account(
        providerRPC,
        ARGENT_CONTRACT_ADDRESS,
        keyPair
      );

      const resp = await account.execute(
        {
          contractAddress: TEST_CONTRACT,
          entrypoint: "test_storage_var",
          calldata: [],
        },
        undefined,
        {
          nonce: "0",
          maxFee: "123456",
        }
      );
      await jumpBlocks(context, 1);

      expect(resp).to.not.be.undefined;
      expect(resp.transaction_hash).to.contain("0x");
    });

    it("should raise with unknown entrypoint", async function () {
      const keyPair = ec.getKeyPair(SIGNER_PRIVATE);
      const account = new Account(
        providerRPC,
        ARGENT_CONTRACT_ADDRESS,
        keyPair
      );

      try {
        await account.execute(
          {
            contractAddress: TEST_CONTRACT,
            entrypoint: "test_storage_var_WRONG",
            calldata: [],
          },
          undefined,
          {
            nonce: "0",
            maxFee: "123456",
          }
        );
      } catch (error) {
        expect(error).to.be.instanceOf(LibraryError);
        expect(error.message).to.equal("40: Contract error");
      }
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

      const invocationDetails = {
        nonce: "0x0",
        maxFee: "0x1111111111111111111111",
        version: "0x1",
      };

      const txHash = hash.calculateDeployAccountTransactionHash(
        deployedContractAddress,
        ARGENT_PROXY_CLASS_HASH,
        calldata,
        SALT,
        invocationDetails.version,
        invocationDetails.maxFee,
        constants.StarknetChainId.TESTNET,
        invocationDetails.nonce
      );

      const keyPair = ec.getKeyPair(SIGNER_PRIVATE);
      const signature = ec.sign(keyPair, txHash);

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

      const accountContractClass = await providerRPC.getClassHashAt(
        deployedContractAddress
      );

      expect(validateAndParseAddress(accountContractClass)).to.be.equal(
        ARGENT_PROXY_CLASS_HASH
      );
    });
  });

  // TODO:
  //    - once starknet-rs supports query tx version
  //    - test w/ account.estimateInvokeFee, account.estimateDeclareFee, account.estimateAccountDeployFee
  describe("estimateFee", async () => {
    it("should estimate fee to 0", async function () {
      const tx = {
        contractAddress: ACCOUNT_CONTRACT,
        calldata: [
          TEST_CONTRACT,
          "0x36fa6de2810d05c3e1a0ebe23f60b9c2f4629bbead09e5a9704e1c5632630d5",
          "0x0",
        ],
      };

      const nonce = await providerRPC.getNonceForAddress(
        ACCOUNT_CONTRACT,
        "latest"
      );

      const txDetails = {
        nonce: nonce,
        version: "0x1",
      };

      const fee_estimate = await providerRPC.getEstimateFee(
        tx,
        txDetails,
        "latest"
      );

      expect(fee_estimate.overall_fee.cmp(toBN(0))).to.be.equal(1);
      expect(fee_estimate.gas_consumed.cmp(toBN(0))).to.be.equal(1);
    });

    it("should raise if contract does not exist", async function () {
      const tx = {
        contractAddress: ACCOUNT_CONTRACT,
        calldata: [
          "0x000000000000000000000000000000000000000000000000000000000000DEAD",
          "0x36fa6de2810d05c3e1a0ebe23f60b9c2f4629bbead09e5a9704e1c5632630d5",
          "0x0",
        ],
      };

      const nonce = await providerRPC.getNonceForAddress(
        ACCOUNT_CONTRACT,
        "latest"
      );

      const txDetails = {
        nonce: nonce,
        version: "0x1",
      };
      try {
        await providerRPC.getEstimateFee(tx, txDetails, "latest");
      } catch (error) {
        expect(error).to.be.instanceOf(LibraryError);
        expect(error.message).to.equal("40: Contract error");
      }
    });
  });

  describe("addDeclareTransaction", async () => {
    it("should return hash starting with 0x", async function () {
      const nonce = await providerRPC.getNonceForAddress(
        ARGENT_CONTRACT_ADDRESS,
        "latest"
      );

      const keyPair = ec.getKeyPair(SIGNER_PRIVATE);
      const account = new Account(
        providerRPC,
        ARGENT_CONTRACT_ADDRESS,
        keyPair
      );

      const contract = fs
        .readFileSync("./contracts/compiled/erc20.json")
        .toString();

      const resp = await account.declare(
        {
          classHash: "0",
          contract,
        },
        { nonce, version: 1, maxFee: "123456" }
      );
      await jumpBlocks(context, 1);

      expect(resp).to.not.be.undefined;
      expect(resp.transaction_hash).to.contain("0x");

      await jumpBlocks(context, 10);
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
      const keyPair = ec.getKeyPair(SIGNER_PRIVATE);

      const nonce = await providerRPC.getNonceForAddress(
        ARGENT_CONTRACT_ADDRESS,
        "latest"
      );

      const account = new Account(
        providerRPC,
        ARGENT_CONTRACT_ADDRESS,
        keyPair
      );

      const contract = fs
        .readFileSync("./contracts/compiled/erc20.json")
        .toString();

      await account.declare(
        {
          classHash: "0",
          contract,
        },
        { nonce, version: 1, maxFee: "123456" }
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

      const txHash = hash.calculateDeployAccountTransactionHash(
        deployedContractAddress,
        ARGENT_PROXY_CLASS_HASH,
        calldata,
        SALT,
        invocationDetails.version,
        invocationDetails.maxFee,
        constants.StarknetChainId.TESTNET,
        invocationDetails.nonce
      );

      const keyPair = ec.getKeyPair(SIGNER_PRIVATE);
      const signature = ec.sign(keyPair, txHash);

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

      try {
        await providerRPC.getTransactionByHash("0x1234");
      } catch (error) {
        expect(error).to.be.instanceOf(LibraryError);
        expect(error.message).to.equal("25: Transaction hash not found");
      }
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

      try {
        await providerRPC.getTransactionByHash(b.transaction_hash);
      } catch (error) {
        expect(error).to.be.instanceOf(LibraryError);
        expect(error.message).to.equal("25: Transaction hash not found");
      }
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

      const r = await providerRPC.getTransactionReceipt(b.result.hash);
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

      try {
        await providerRPC.getTransactionReceipt("0x1234");
      } catch (error) {
        expect(error).to.be.instanceOf(LibraryError);
        expect(error.message).to.equal("25: Transaction hash not found");
      }
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
      };
      try {
        await providerRPC.getEvents(filter);
      } catch (error) {
        expect(error).to.be.instanceOf(LibraryError);
        expect(error.message).to.equal(
          "33: The supplied continuation token is invalid or unknown"
        );
      }
    });

    it("should fail on chunk size too big", async function () {
      const filter = {
        from_block: { block_number: 0 },
        to_block: { block_number: 1 },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 1001,
      };
      try {
        await providerRPC.getEvents(filter);
      } catch (error) {
        expect(error).to.be.instanceOf(LibraryError);
        expect(error.message).to.equal("31: Requested page size is too big");
      }
    });

    it("should fail on keys too big", async function () {
      const filter = {
        from_block: { block_number: 0 },
        to_block: { block_number: 1 },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 1,
        keys: Array(101).fill(["0x0"]),
      };
      try {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        await providerRPC.getEvents(filter);
      } catch (error) {
        expect(error).to.be.instanceOf(LibraryError);
        expect(error.message).to.equal(
          "34: Too many keys provided in a filter"
        );
      }
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
          ARGENT_CONTRACT_ADDRESS,
          "0x19e1a", // current fee perceived for the transfer
          "0x0",
        ].map(cleanHex),
      });
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
      expect(toHex(events.continuation_token)).to.be.equal("0x6");
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
        continuation_token: (skip * 3).toString(), // 3 events per transaction
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
      expect(toHex(events.continuation_token)).to.be.equal("0x1");
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
});
