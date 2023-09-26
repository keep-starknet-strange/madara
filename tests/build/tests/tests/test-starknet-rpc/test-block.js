"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
require("@keep-starknet-strange/madara-api-augment");
const chai_1 = require("chai");
const starknet_1 = require("starknet");
const block_1 = require("../../util/block");
const setup_dev_tests_1 = require("../../util/setup-dev-tests");
const utils_1 = require("../../util/utils");
const constants_1 = require("../constants");
let ARGENT_CONTRACT_NONCE = { value: 0 };
(0, setup_dev_tests_1.describeDevMadara)("Starknet RPC - Block Test", (context) => {
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
            (0, chai_1.expect)(block.block_hash).to.not.be.equal("");
            (0, chai_1.expect)(block.block_number).to.be.equal(0);
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
        it("should return 1 for 1 transaction", async function () {
            await context.createBlock((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT), {
                finalize: true,
            });
            const transactionCount = await providerRPC.getTransactionCount("latest");
            (0, chai_1.expect)(transactionCount).to.not.be.undefined;
            (0, chai_1.expect)(transactionCount).to.be.equal(1);
        });
        it("should raise on invalid block id", async () => {
            const count = providerRPC.getTransactionCount("0x123");
            await (0, chai_1.expect)(count)
                .to.eventually.be.rejectedWith("24: Block not found")
                .and.be.an.instanceOf(starknet_1.LibraryError);
        });
    });
    describe("getNonce", async () => {
        it("should increase after a transaction", async function () {
            let nonce = await providerRPC.getNonceForAddress(constants_1.ARGENT_CONTRACT_ADDRESS, "latest");
            await context.createBlock((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT));
            nonce = await providerRPC.getNonceForAddress(constants_1.ARGENT_CONTRACT_ADDRESS, "latest");
            (0, chai_1.expect)(nonce).to.not.be.undefined;
            (0, chai_1.expect)(nonce).to.be.equal((0, utils_1.toHex)(ARGENT_CONTRACT_NONCE.value));
        });
    });
    describe("syncing", async () => {
        it("should return starting setup and current_block info", async function () {
            await (0, block_1.jumpBlocks)(context, 10);
            const status = await providerRPC.getSyncingStats();
            const current_block = await providerRPC.getBlockHashAndNumber();
            (0, chai_1.expect)(status["starting_block_num"]).to.be.equal("0x0");
            (0, chai_1.expect)(parseInt(status["current_block_num"])).to.be.equal(current_block["block_number"]);
            (0, chai_1.expect)(parseInt(status["highest_block_num"])).to.be.equal(current_block["block_number"]);
            (0, chai_1.expect)(status["starting_block_hash"]).to.contain("0x31eb");
            (0, chai_1.expect)(status["current_block_hash"]).to.be.equal(current_block["block_hash"]);
            (0, chai_1.expect)(status["highest_block_hash"]).to.be.equal(current_block["block_hash"]);
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
        it("should returns transactions", async function () {
            await context.createBlock((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT));
            const blockWithTxHashes = await providerRPC.getBlockWithTxHashes("latest");
            (0, chai_1.expect)(blockWithTxHashes).to.not.be.undefined;
            (0, chai_1.expect)(blockWithTxHashes.status).to.be.equal("ACCEPTED_ON_L2");
            (0, chai_1.expect)(blockWithTxHashes.transactions.length).to.be.equal(1);
        });
        it("should raise with invalid block id", async function () {
            const block = providerRPC.getBlockWithTxHashes("0x123");
            await (0, chai_1.expect)(block)
                .to.eventually.be.rejectedWith("24: Block not found")
                .and.be.an.instanceOf(starknet_1.LibraryError);
        });
    });
    describe("getBlockWithTxs", async () => {
        it("should returns empty block", async function () {
            await context.createBlock(undefined, {
                parentHash: undefined,
                finalize: true,
            });
            const latestBlock = await providerRPC.getBlockWithTxs("latest");
            (0, chai_1.expect)(latestBlock).to.not.be.undefined;
            (0, chai_1.expect)(latestBlock.status).to.be.equal("ACCEPTED_ON_L2");
            (0, chai_1.expect)(latestBlock.transactions.length).to.be.equal(0);
        });
        it("should returns transactions", async function () {
            await context.createBlock((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT));
            const blockHash = await providerRPC.getBlockHashAndNumber();
            await (0, block_1.jumpBlocks)(context, 10);
            const blockWithTxHashes = await providerRPC.getBlockWithTxs(blockHash.block_hash);
            const tx = blockWithTxHashes.transactions[0];
            (0, chai_1.expect)(blockWithTxHashes).to.not.be.undefined;
            (0, chai_1.expect)(blockWithTxHashes.transactions.length).to.be.equal(1);
            (0, chai_1.expect)(tx.type).to.be.equal("INVOKE");
            (0, chai_1.expect)(tx.sender_address).to.be.equal((0, utils_1.toHex)(constants_1.ARGENT_CONTRACT_ADDRESS));
            (0, chai_1.expect)(tx.calldata).to.deep.equal([
                1,
                constants_1.FEE_TOKEN_ADDRESS,
                starknet_1.hash.getSelectorFromName("transfer"),
                0,
                3,
                3,
                constants_1.ARGENT_CONTRACT_ADDRESS,
                constants_1.MINT_AMOUNT,
                0,
            ].map(utils_1.toHex));
        });
        it("should raise with invalid block id", async function () {
            const block = providerRPC.getBlockWithTxs("0x123");
            await (0, chai_1.expect)(block)
                .to.eventually.be.rejectedWith("24: Block not found")
                .and.be.an.instanceOf(starknet_1.LibraryError);
        });
    });
    describe("Fix #551: Madara RPC doesn't handle 'pending' block id", async () => {
        it("should support 'pending' block id", async function () {
            const nonce = await providerRPC.getNonceForAddress(constants_1.ARGENT_CONTRACT_ADDRESS, "pending");
            (0, chai_1.expect)(nonce).to.not.be.undefined;
        });
        it("should support 'latest' block id", async function () {
            const nonce = await providerRPC.getNonceForAddress(constants_1.ARGENT_CONTRACT_ADDRESS, "latest");
            (0, chai_1.expect)(nonce).to.not.be.undefined;
        });
    });
});
//# sourceMappingURL=test-block.js.map