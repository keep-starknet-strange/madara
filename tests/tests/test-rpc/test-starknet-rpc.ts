import "@keep-starknet-strange/madara-api-augment";
import chai, { expect } from "chai";
import { createAndFinalizeBlock } from "../../util/block";
import deepEqualInAnyOrder from "deep-equal-in-any-order";
import {
  LibraryError,
  RpcProvider,
  Account,
  stark,
  ec,
  hash,
  constants,
  validateAndParseAddress,
} from "starknet";
import { jumpBlocks } from "../../util/block";
import { describeDevMadara } from "../../util/setup-dev-tests";
import { transfer } from "../../util/starknet";
import {
  ACCOUNT_CONTRACT,
  ACCOUNT_CONTRACT_CLASS_HASH,
  ARGENT_CONTRACT_ADDRESS,
  CHAIN_ID_STARKNET_TESTNET,
  CONTRACT_ADDRESS,
  FEE_TOKEN_ADDRESS,
  MINT_AMOUNT,
  TEST_CONTRACT,
  TEST_CONTRACT_CLASS_HASH,
  TOKEN_CLASS_HASH,
  ARGENT_ACCOUNT_CLASS_HASH,
  ARGENT_PROXY_CLASS_HASH,
  SIGNER_PRIVATE,
  SIGNER_PUBLIC,
  SALT,
} from "../constants";
import { toHex } from "../../util/utils";

chai.use(deepEqualInAnyOrder);

describeDevMadara("Starknet RPC", (context) => {
  let providerRPC: RpcProvider;

  before(async function () {
    providerRPC = new RpcProvider({
      nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
      retries: 3,
    }); // substrate node
  });

  it("getBlockhashAndNumber", async function () {
    const block = await providerRPC.getBlockHashAndNumber();

    expect(block).to.not.be.undefined;
  });

  it("getBlockNumber", async function () {
    const blockNumber = await providerRPC.getBlockNumber();

    expect(blockNumber).to.not.be.undefined;

    await jumpBlocks(context, 10);

    const blockNumber2 = await providerRPC.getBlockNumber();

    expect(blockNumber2).to.be.equal(blockNumber + 10);
  });

  it("getBlockTransactionCount", async function () {
    const transactionCount = await providerRPC.getTransactionCount("latest");

    expect(transactionCount).to.not.be.undefined;
    expect(transactionCount).to.be.equal(0);
  });

  it("call", async function () {
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

  it("getClassAt", async function () {
    const contract_class = await providerRPC.getClassAt(
      TEST_CONTRACT,
      "latest"
    );

    expect(contract_class).to.not.be.undefined;
  });

  it("getClassHashAt", async function () {
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

    // Invalid block id
    try {
      await providerRPC.getClassHashAt(TEST_CONTRACT, "0x123");
    } catch (error) {
      expect(error).to.be.instanceOf(LibraryError);
      expect(error.message).to.equal("24: Block not found");
    }

    // Invalid/un-deployed contract address
    try {
      await providerRPC.getClassHashAt("0x123", "latest");
    } catch (error) {
      expect(error).to.be.instanceOf(LibraryError);
      expect(error.message).to.equal("20: Contract not found");
    }
  });

  it("syncing", async function () {
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

  it("getClass", async function () {
    const contract_class = await providerRPC.getClass(
      TOKEN_CLASS_HASH,
      "latest"
    );

    expect(contract_class).to.not.be.undefined;
  });

  describe("Get block with transaction hashes", () => {
    it(
      "giving a valid block with txs " +
        "when call getBlockWithTxHashes " +
        "then returns an object with transactions",
      async function () {
        await context.createBlock(
          transfer(
            context.polkadotApi,
            CONTRACT_ADDRESS,
            FEE_TOKEN_ADDRESS,
            CONTRACT_ADDRESS,
            MINT_AMOUNT,
            0
          ),
          { parentHash: undefined, finalize: true }
        );

        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const blockWithTxHashes: { status: string; transactions: string[] } =
          await providerRPC.getBlockWithTxHashes("latest");
        expect(blockWithTxHashes).to.not.be.undefined;
        expect(blockWithTxHashes.status).to.be.equal("ACCEPTED_ON_L2");
        expect(blockWithTxHashes.transactions.length).to.be.equal(1);
      }
    );

    it(
      "giving an invalid block " +
        "when call getBlockWithTxHashes " +
        "then throw 'Block not found error'",
      async function () {
        await providerRPC.getBlockWithTxHashes("0x123").catch((error) => {
          expect(error).to.be.instanceOf(LibraryError);
          expect(error.message).to.equal("24: Block not found");
        });
      }
    );

    it(
      "giving a valid block without txs " +
        "when call getBlockWithTxHashes " +
        "then returns an object with empty transactions",
      async function () {
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
      }
    );
  });

  it("Gets value from the fee contract storage", async function () {
    const value = await providerRPC.getStorageAt(
      FEE_TOKEN_ADDRESS,
      // ERC20_balances(0x01).low
      "0x07b62949c85c6af8a50c11c22927f9302f7a2e40bc93b4c988415915b0f97f09",
      "latest"
    );
    expect(toHex(value)).to.be.equal("0xffffffffffffffffffffffffffffffff");
  });

  it("Returns 0 if the storage slot is not set", async function () {
    const value = await providerRPC.getStorageAt(
      FEE_TOKEN_ADDRESS,
      // ERC20_balances(0x01).low
      "0x0000000000000000000000000000000000000000000000000000000000000000",
      "latest"
    );
    expect(value).to.be.equal("0");
  });

  it("Returns an error if the contract does not exist", async function () {
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

  it("chainId", async function () {
    const chainId = await providerRPC.getChainId();

    expect(chainId).to.not.be.undefined;
    expect(chainId).to.be.equal(CHAIN_ID_STARKNET_TESTNET);
  });

  describe("Get transaction by blockId and index", () => {
    it(
      "giving a valid block with txs " +
        "when call getTransactionByBlockIdAndIndex " +
        "then returns a transaction",
      async function () {
        // Send a transaction
        await context.createBlock(
          transfer(
            context.polkadotApi,
            CONTRACT_ADDRESS,
            FEE_TOKEN_ADDRESS,
            CONTRACT_ADDRESS,
            MINT_AMOUNT,
            1
          ),
          { parentHash: undefined, finalize: true }
        );

        const latestBlockCreated = await providerRPC.getBlockHashAndNumber();

        const getTransactionByBlockIdAndIndexResponse =
          await providerRPC.getTransactionByBlockIdAndIndex(
            latestBlockCreated.block_hash,
            0
          );

        expect(getTransactionByBlockIdAndIndexResponse).to.not.be.undefined;
      }
    );

    it(
      "giving an invalid block " +
        "when call getTransactionByBlockIdAndIndex " +
        "then throw 'Block not found error'",
      async function () {
        await providerRPC
          .getTransactionByBlockIdAndIndex("0x123", 2)
          .catch((error) => {
            expect(error).to.be.instanceOf(LibraryError);
            expect(error.message).to.equal("24: Block not found");
          });
      }
    );

    it(
      "giving a valid block without txs" +
        "when call getTransactionByBlockIdAndIndex " +
        "then throw 'Invalid transaction index in a block'",
      async function () {
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
      }
    );
  });

  it("Adds an invocation transaction successfully", async function () {
    const priKey = stark.randomAddress();
    const keyPair = ec.getKeyPair(priKey);
    const account = new Account(providerRPC, ARGENT_CONTRACT_ADDRESS, keyPair);

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

  it("Returns error when invocation absent entrypoint", async function () {
    const priKey = stark.randomAddress();
    const keyPair = ec.getKeyPair(priKey);
    const account = new Account(providerRPC, ARGENT_CONTRACT_ADDRESS, keyPair);

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

  it("Adds an invocation transaction successfully", async function () {
    // Compute contract address
    const selector = hash.getSelectorFromName("initialize");
    const calldata = [ARGENT_ACCOUNT_CLASS_HASH, selector, 2, SIGNER_PUBLIC, 0];

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

    await providerRPC.deployAccountContract(txDeployAccount, invocationDetails);
    await createAndFinalizeBlock(context.polkadotApi);

    const accountContractClass = await providerRPC.getClassHashAt(
      deployedContractAddress
    );

    expect(validateAndParseAddress(accountContractClass)).to.be.equal(
      ARGENT_PROXY_CLASS_HASH
    );
  });
});
