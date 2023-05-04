import "@keep-starknet-strange/madara-api-augment";
import chai, { expect } from "chai";
import { describeDevMadara } from "../../util/setup-dev-tests";
import { LibraryError, RPC, RpcProvider } from "starknet";
import { jumpBlocks } from "../../util/block";
import {
  TEST_CONTRACT,
  CONTRACT_ADDRESS,
  FEE_TOKEN_ADDRESS,
  MINT_AMOUNT,
  ACCOUNT_CONTRACT,
  ACCOUNT_CONTRACT_CLASS_HASH,
  TEST_CONTRACT_CLASS_HASH,
  TOKEN_CLASS_HASH,
} from "./constants";
import deepEqualInAnyOrder from "deep-equal-in-any-order";
import { transfer } from "../../util/starknet";

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
    const block = await providerRPC.getBlockHashAndNumber();

    const transactionCount = await providerRPC.getTransactionCount(
      block.block_hash
    );

    expect(transactionCount).to.not.be.undefined;
    expect(transactionCount).to.be.equal(0);
  });

  it("call", async function () {
    const block = await providerRPC.getBlockHashAndNumber();


    const call = await providerRPC.callContract(
      {
        contractAddress: TEST_CONTRACT,
        entrypoint: "return_result",
        calldata: ["0x19"],
      },
      block.block_hash
    );

    expect(call.result).to.contain("0x19");
  });

  it("getClassAt", async function () {
    const blockHashAndNumber = await providerRPC.getBlockHashAndNumber();
    const block_number: number = blockHashAndNumber.block_number;

    const contract_class = await providerRPC.getClassAt(
      TEST_CONTRACT,
      block_number
    );

    expect(contract_class).to.not.be.undefined;
  });

  it("getClassHashAt", async function () {
    const blockHashAndNumber = await providerRPC.getBlockHashAndNumber();
    const block_hash = blockHashAndNumber.block_hash;

    // Account Contract
    const account_contract_class_hash = await providerRPC.getClassHashAt(
      ACCOUNT_CONTRACT,
      block_hash
    );

    expect(account_contract_class_hash).to.not.be.undefined;
    expect(account_contract_class_hash).to.be.equal(
      ACCOUNT_CONTRACT_CLASS_HASH
    );

    const test_contract_class_hash = await providerRPC.getClassHashAt(
      TEST_CONTRACT,
      block_hash
    );

    expect(test_contract_class_hash).to.not.be.undefined;
    expect(test_contract_class_hash).to.be.equal(TEST_CONTRACT_CLASS_HASH);

    // Invalid block id
    try {
      await providerRPC.getClassHashAt(TEST_CONTRACT, "0x123");
    } catch (error) {
      expect(error).to.be.instanceOf(LibraryError);
      expect(error.message).to.equal("24: Block not found");
    }

    // Invalid/un-deployed contract address
    try {
      await providerRPC.getClassHashAt("0x123", block_hash);
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
    expect(status["sync_status"]["starting_block_num"]).to.be.equal(0);
    // starknet current and highest block number should be equal to
    // the current block with this test setup
    expect(status["sync_status"]["current_block_num"]).to.be.equal(
      current_block["block_number"]
    );
    expect(status["sync_status"]["highest_block_num"]).to.be.equal(
      current_block["block_number"]
    );

    // the starknet block hash for number 0 starts with "0xaf" with this test setup
    expect(status["sync_status"]["starting_block_hash"]).to.contain("0xaf");
    // starknet current and highest block number should be equal to
    // the current block with this test setup
    expect(status["sync_status"]["current_block_hash"]).to.be.equal(
      current_block["block_hash"]
    );
    expect(status["sync_status"]["highest_block_hash"]).to.be.equal(
      current_block["block_hash"]
    );
  });

  it("getClass", async function () {
    const blockHashAndNumber = await providerRPC.getBlockHashAndNumber();
    const block_number: number = blockHashAndNumber.block_number;

    const contract_class = await providerRPC.getClass(
      TOKEN_CLASS_HASH,
      block_number
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
            MINT_AMOUNT
          ),
          { parentHash: undefined, finalize: true }
        );

        const latestBlockCreated = await providerRPC.getBlockHashAndNumber();
        const getBlockWithTxsHashesResponse: RPC.GetBlockWithTxHashesResponse =
          await providerRPC.getBlockWithTxHashes(latestBlockCreated.block_hash);

        const block_with_tx_hashes = getBlockWithTxsHashesResponse["Block"];

        expect(block_with_tx_hashes).to.not.be.undefined;
        expect(block_with_tx_hashes.status).to.be.equal("ACCEPTED_ON_L2");
        expect(block_with_tx_hashes.transactions.length).to.be.equal(1);
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
      "giving a valid block without txs" +
        "when call getBlockWithTxHashes " +
        "then returns an object with empty transactions",
      async function () {
        await context.createBlock(undefined, {
          parentHash: undefined,
          finalize: true,
        });

        const latestBlockCreated = await providerRPC.getBlockHashAndNumber();


        const getBlockWithTxsHashesResponse: RPC.GetBlockWithTxHashesResponse =
          await providerRPC.getBlockWithTxHashes(latestBlockCreated.block_hash);
        const block_with_tx_hashes = getBlockWithTxsHashesResponse["Block"];
        const latestBlock = (await providerRPC.getBlockWithTxHashes("latest"))[
          "Block"
        ];
        // Weird that we need that.
        expect(latestBlock).to.deep.equalInAnyOrder(block_with_tx_hashes);
        expect(block_with_tx_hashes).to.not.be.undefined;
        expect(block_with_tx_hashes.status).to.be.equal("ACCEPTED_ON_L2");
        expect(block_with_tx_hashes.transactions.length).to.deep.equal(0);
      }
    );
  });

  it("syncing", async function () {
    await jumpBlocks(context, 10);

    const status = await providerRPC.getSyncingStats();
    const current_block = await providerRPC.getBlockHashAndNumber();

    // starknet starting block number should be 0 with this test setup
    expect(status["sync_status"]["starting_block_num"]).to.be.equal(0);
    // starknet current and highest block number should be equal to
    // the current block with this test setup
    expect(status["sync_status"]["current_block_num"]).to.be.equal(
      current_block["block_number"]
    );
    expect(status["sync_status"]["highest_block_num"]).to.be.equal(
      current_block["block_number"]
    );

    // the starknet block hash for number 0 starts with "0xaf" with this test setup
    expect(status["sync_status"]["starting_block_hash"]).to.contain("0xaf");
    // starknet current and highest block number should be equal to
    // the current block with this test setup
    expect(status["sync_status"]["current_block_hash"]).to.be.equal(
      current_block["block_hash"]
    );
    expect(status["sync_status"]["highest_block_hash"]).to.be.equal(
      current_block["block_hash"]
    );
  });
});
