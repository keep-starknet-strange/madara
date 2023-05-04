import "@keep-starknet-strange/madara-api-augment";

import { expect } from "chai";

import { RpcProvider } from "starknet";
import { jumpBlocks } from "../../util/block";
import { describeDevMadara } from "../../util/setup-dev-tests";
import { TEST_CONTRACT, TEST_CLASS_HASH } from "./constants";

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
      TEST_CLASS_HASH,
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
  });
});
