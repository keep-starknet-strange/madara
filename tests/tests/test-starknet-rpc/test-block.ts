import "@keep-starknet-strange/madara-api-augment";
import { expect } from "chai";
import { LibraryError, RpcProvider, hash } from "starknet";
import { jumpBlocks } from "../../util/block";
import { describeDevMadara } from "../../util/setup-dev-tests";
import { rpcTransfer, toHex } from "../../util/utils";
import {
  ARGENT_CONTRACT_ADDRESS,
  FEE_TOKEN_ADDRESS,
  MINT_AMOUNT,
} from "../constants";
import { Block, InvokeTransaction } from "./types";

// chai.use(deepEqualInAnyOrder);
// chai.use(chaiAsPromised);

// keep "let" over "const" as the nonce is passed by reference
// to abstract the increment
// eslint-disable-next-line prefer-const
let ARGENT_CONTRACT_NONCE = { value: 0 };

describeDevMadara("Starknet RPC - Block Test", (context) => {
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
          MINT_AMOUNT,
        ),
        {
          finalize: true,
        },
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
        "latest",
      );

      await context.createBlock(
        rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT,
        ),
      );

      nonce = await providerRPC.getNonceForAddress(
        ARGENT_CONTRACT_ADDRESS,
        "latest",
      );

      expect(nonce).to.not.be.undefined;
      expect(nonce).to.be.equal(toHex(ARGENT_CONTRACT_NONCE.value));
    });
  });

  describe("syncing", async () => {
    it("should return starting setup and current_block info", async function () {
      await jumpBlocks(context, 10);

      const status = await providerRPC.getSyncingStats();
      const current_block = await providerRPC.getBlockHashAndNumber();

      // starknet starting block number should be 0 with this test setup
      expect(status["starting_block_num"]).to.be.equal(0);
      // starknet current and highest block number should be equal to
      // the current block with this test setup
      expect(parseInt(status["current_block_num"])).to.be.equal(
        current_block["block_number"],
      );
      expect(parseInt(status["highest_block_num"])).to.be.equal(
        current_block["block_number"],
      );

      // the starknet block hash for number 0 starts with "0x31eb" with this test setup
      expect(status["starting_block_hash"]).to.contain("0x31eb");
      // starknet current and highest block number should be equal to
      // the current block with this test setup
      expect(status["current_block_hash"]).to.be.equal(
        current_block["block_hash"],
      );
      expect(status["highest_block_hash"]).to.be.equal(
        current_block["block_hash"],
      );
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
      const latestBlock: Block =
        await providerRPC.getBlockWithTxHashes("latest");
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
          MINT_AMOUNT,
        ),
      );

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const blockWithTxHashes: Block =
        await providerRPC.getBlockWithTxHashes("latest");
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
          MINT_AMOUNT,
        ),
      );

      const blockHash = await providerRPC.getBlockHashAndNumber();
      await jumpBlocks(context, 10);

      const blockWithTxHashes = await providerRPC.getBlockWithTxs(
        blockHash.block_hash,
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
        ].map(toHex),
      );
    });

    it("should raise with invalid block id", async function () {
      const block = providerRPC.getBlockWithTxs("0x123");
      await expect(block)
        .to.eventually.be.rejectedWith("24: Block not found")
        .and.be.an.instanceOf(LibraryError);
    });
  });

  describe("Fix #551: Madara RPC doesn't handle 'pending' block id", async () => {
    it("should support 'pending' block id", async function () {
      const nonce = await providerRPC.getNonceForAddress(
        ARGENT_CONTRACT_ADDRESS,
        "pending",
      );
      expect(nonce).to.not.be.undefined;
    });

    it("should support 'latest' block id", async function () {
      const nonce = await providerRPC.getNonceForAddress(
        ARGENT_CONTRACT_ADDRESS,
        "latest",
      );
      expect(nonce).to.not.be.undefined;
    });
  });
});
