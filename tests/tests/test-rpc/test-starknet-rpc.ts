import "@keep-starknet-strange/madara-api-augment";
import { expect } from "chai";
import { LibraryError, RpcProvider, RPC } from "starknet";
import { describeDevMadara } from "../../util/setup-dev-tests";
import { jumpBlocks } from "../../util/block";
import { TEST_CONTRACT, CONTRACT_ADDRESS, FEE_TOKEN_ADDRESS, MINT_AMOUNT, TOKEN_CLASS_HASH, ACCOUNT_CONTRACT, ACCOUNT_CONTRACT_CLASS_HASH, TEST_CONTRACT_CLASS_HASH } from "./constants";
import {
  transfer,
} from "../../util/starknet";

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
      TOKEN_CLASS_HASH,
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
        CONTRACT_ADDRESS,
        FEE_TOKEN_ADDRESS,
        CONTRACT_ADDRESS,
        MINT_AMOUNT
      )
    );

    const chain_result = await context.polkadotApi.rpc.chain.getBlock(result.block.hash);
    console.log("extrensics: ", chain_result.block.extrinsics[0].method.args);
    const block_number = chain_result.block.header.number.toNumber();

    // happy path test
    const block_with_tx_hashes = await providerRPC.getBlockWithTxHashes(block_number);
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
  it("giving a valid block with txs " +
    "when call getBlockWithTxHashes " +
    "then returns an object with transactions", async function () {
      const createdBlockResponse = await context.createBlock(
        transfer(context.polkadotApi, CONTRACT_ADDRESS, FEE_TOKEN_ADDRESS, CONTRACT_ADDRESS, MINT_AMOUNT), //
        { parentHash: undefined, finalize: true }
      );
      it("giving a valid block with txs " +
        "when call getBlockWithTxHashes " +
        "then returns an object with transactions", async function () {
          await context.createBlock(
            transfer(context.polkadotApi, CONTRACT_ADDRESS, FEE_TOKEN_ADDRESS, CONTRACT_ADDRESS, MINT_AMOUNT),
            { parentHash: undefined, finalize: true }
          );

          const latestBlockCreated = await providerRPC.getBlockHashAndNumber();
          const getBlockWithTxsHashesResponse: RPC.GetBlockWithTxHashesResponse = await providerRPC.getBlockWithTxHashes(latestBlockCreated.block_hash)

          const block_with_tx_hashes = getBlockWithTxsHashesResponse['Block'];

          console.log(block_with_tx_hashes)
          expect(block_with_tx_hashes).to.not.be.undefined;
          expect(block_with_tx_hashes.status).to.be.equal("ACCEPTED_ON_L2");
          expect(block_with_tx_hashes.transactions.length).to.be.equal(1);
        });

      it("giving a valid block without txs" +
        "when call getBlockWithTxHashes " +
        "then returns an object with empty transactions", async function () {
          await context.createBlock(
            undefined,
            { parentHash: undefined, finalize: true });

          const latestBlockCreated = await providerRPC.getBlockHashAndNumber();
          console.log("then call providerRPC.getBlockHashAndNumber()", await providerRPC.getBlockHashAndNumber())


          // It should be able to use the getBlockWithTxsHashes
          // method with a createdBlockResponse.block.hash.toHex() | toString()? but obtain 'LibraryError: 24: Block not found'
          // even using providerRPC.getBlock() returns a 'Block not found'
          // const getBlockWithTxsHashesResponse: RPC.GetBlockWithTxHashesResponse = await providerRPC.getBlockWithTxHashes(signedBlock.block.hash.toHex())

          // I search for the block by hash in the chain to validate that it actually exists (it returns me a block)
          // Tried to review the transactions of the obtained block, I find the 'transfer' previously made
          // (2 transactions). However, if instead of sending the 'transfer' at the time of creating the block,
          // I send 'undefined', then only 1 transaction appears (the one with the timestamp).
          const signedBlock = await context.polkadotApi.rpc.chain.getBlock(createdBlockResponse.block.hash);
          console.log("signedBlock (obtained from context.polkadotApi.rpc.chain.getBlock) { number:", signedBlock.block.header.number.toNumber(), ", hash:", signedBlock.block.hash.toHex(), "}")
          console.log("extrinsics.hashes => {")
          signedBlock.block.extrinsics.forEach(value => {
            // console.log("extrinsics.hashes => value {", value.method.args, "}")
            console.log("  ", value.method.args[0]['hash'].toHex(), ",")
          })
          console.log("}")

          const getBlockWithTxsHashesResponse: RPC.GetBlockWithTxHashesResponse = await providerRPC.getBlockWithTxHashes(latestBlockCreated.block_hash)
          const block_with_tx_hashes = getBlockWithTxsHashesResponse['Block'];

          expect(block_with_tx_hashes).to.not.be.undefined;
          expect(block_with_tx_hashes.status).to.be.equal("ACCEPTED_ON_L2");
          expect(block_with_tx_hashes.transactions.length).to.be.equal(0);
        });

      it("giving an invalid block " +
        "when call getBlockWithTxHashes " +
        "then throw 'Block not found error'", async function () {
          try {
            await providerRPC.getBlockWithTxHashes("invalid_block_hash");
          } catch (error) {
            expect(error).to.be.instanceOf(LibraryError);
            expect(error.message).to.equal("24: Block not found");
          }
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
    });
  });
});