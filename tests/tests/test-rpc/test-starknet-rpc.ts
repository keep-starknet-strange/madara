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
import { rpcTransfer, toBN, toHex } from "../../util/utils";
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
      const blockWithTxHashes: { status: string; transactions: string[] } =
        await providerRPC.getBlockWithTxHashes("latest");
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
      const tx: { type: string; sender_address: string; calldata: string[] } =
        blockWithTxHashes.transactions[0];
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
      const latestBlock: { status: string; transactions: string[] } =
        await providerRPC.getBlockWithTxHashes("latest");
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
      const latestBlock: { status: string; transactions: string[] } =
        await providerRPC.getBlockWithTxHashes("latest");
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
          classHash:
            "0x057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95",
          contract,
        },
        { nonce, version: 1, maxFee: "123456" }
      );

      expect(resp).to.not.be.undefined;
      expect(resp.transaction_hash).to.contain("0x");
    });
  });
});
