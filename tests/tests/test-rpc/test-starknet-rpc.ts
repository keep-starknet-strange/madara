import "@keep-starknet-strange/madara-api-augment";

import { expect } from "chai";

import { jumpBlocks } from "../../util/block";
import { describeDevMadara } from "../../util/setup-dev-tests";
import { RpcProvider, LibraryError } from "starknet";
import {
  ACCOUNT_CONTRACT,
  ACCOUNT_CONTRACT_CLASS_HASH,
  TEST_CONTRACT,
  TEST_CONTRACT_CLASS_HASH,
} from "./constants";

import {
  transfer,
} from "../../util/starknet";

const mintAmount =
  "0x0000000000000000000000000000000000000000000000000000000000000001";
const contractAddress =
  "0x0000000000000000000000000000000000000000000000000000000000000001";
const feeTokenAddress =
  "0x040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d00";

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

    console.log(block);

    expect(block).to.not.be.undefined;
  });

  it("getBlockNumber", async function () {
    const blockNumber = await providerRPC.getBlockNumber();

    console.log(blockNumber);

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

    console.log(transactionCount);

    expect(transactionCount).to.not.be.undefined;
    expect(transactionCount).to.be.equal(0);
  });

  it("call", async function () {
    const block = await providerRPC.getBlockHashAndNumber();

    const block_hash = `0x${block.block_hash.slice(2).padStart(64, "0")}`;

    const call = await providerRPC.callContract(
      {
        contractAddress: TEST_CONTRACT,
        entrypoint: "return_result",
        calldata: ["0x19"],
      },
      block_hash
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

    console.log(`Class Hash: ${account_contract_class_hash}`);

    expect(account_contract_class_hash).to.not.be.undefined;
    expect(account_contract_class_hash).to.be.equal(
      ACCOUNT_CONTRACT_CLASS_HASH
    );

    const test_contract_class_hash = await providerRPC.getClassHashAt(
      TEST_CONTRACT,
      block_hash
    );

    console.log(`Class Hash: ${test_contract_class_hash}`);

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
      TEST_CONTRACT_CLASS_HASH,
      block_number
    );

    expect(contract_class).to.not.be.undefined;
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
  it("getBlockWithTxHashes", async function () {
    const block = await providerRPC.getBlockHashAndNumber();
    const block_hash = `0x${block.block_hash.slice(2).padStart(64, "0")}`;
    const block_with_tx_hashes = await providerRPC.getBlockWithTxHashes(block_hash);

    console.log(`Block with tx hashes: ${block_with_tx_hashes}`);
    expect(block_with_tx_hashes).to.not.be.undefined;
  });
  it("getBlockWithTxHashes", async function () {
    // happy path test
    const block = await providerRPC.getBlockHashAndNumber();
    const block_hash = `0x${block.block_hash.slice(2).padStart(64, "0")}`;
    const block_with_tx_hashes = await providerRPC.getBlockWithTxHashes(block_hash);
    expect(block_with_tx_hashes).to.not.be.undefined;

    // Invalid block id test
    try {
      await providerRPC.getBlockWithTxHashes("0x123");
    } catch (error) {
      expect(error.message).to.equal("24: Block not found");
    }
  });
  it("getBlockWithTxHashes", async function () {
    const result = await context.createBlock(
      transfer(
        context.polkadotApi,
        contractAddress,
        feeTokenAddress,
        contractAddress,
        mintAmount
      )
    );

    const chain_result = await context.polkadotApi.rpc.chain.getBlock(result.block.hash);
    console.log("extrensics: ", chain_result.block.extrinsics[0].method.args);
    const block_number = chain_result.block.header.number.toNumber();

    // happy path test
    let block_with_tx_hashes = await providerRPC.getBlockWithTxHashes(block_number);
    console.log("getBlockWithTxHashes(): ", block_with_tx_hashes);
    expect(block_with_tx_hashes).to.not.be.undefined;
    // expect(block_with_tx_hashes.transactions.length).to.have.length(1);

    // Invalid block id test
    try {
      await providerRPC.getBlockWithTxHashes("0x123");
    } catch (error) {
      expect(error).to.be.instanceOf(LibraryError);
      expect(error.message).to.equal("24: Block not found");
    }

  });
});
});