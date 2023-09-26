"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
require("@keep-starknet-strange/madara-api-augment");
const chai_1 = require("chai");
const util_1 = require("@polkadot/util");
const block_1 = require("../../util/block");
const setup_dev_tests_1 = require("../../util/setup-dev-tests");
const starknet_1 = require("../../util/starknet");
const constants_1 = require("../constants");
const starknet_2 = require("starknet");
(0, setup_dev_tests_1.describeDevMadara)("Pallet Starknet - Extrinsics", (context) => {
    let providerRPC;
    before(async function () {
        providerRPC = new starknet_2.RpcProvider({
            nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
            retries: 3,
        });
    });
    it("should connect to local node", async function () {
        const rdy = context.polkadotApi.isConnected;
        (0, chai_1.expect)(rdy).to.be.true;
    });
    it("should jump 10 blocks", async function () {
        const rdy = context.polkadotApi.isConnected;
        (0, chai_1.expect)(rdy).to.be.true;
        await (0, block_1.jumpBlocks)(context, 10);
    });
    it.skip("should declare a new contract class", async function () {
        const { result: { events }, } = await context.createBlock((0, starknet_1.declare)(context.polkadotApi, constants_1.CONTRACT_ADDRESS, constants_1.TOKEN_CLASS_HASH));
        (0, chai_1.expect)(events.find(({ event: { section, method } }) => section == "system" && method == "ExtrinsicSuccess")).to.exist;
    });
    it("should deploy a new contract", async function () {
        const deployedContractAddress = starknet_2.hash.calculateContractAddressFromHash("0x0000000000000000000000000000000000000000000000000000000000000001", "0x0000000000000000000000000000000000000000000000000000000000010000", [
            "0x000000000000000000000000000000000000000000000000000000000000000A",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
            "0x0000000000000000000000000000000000000000000000000000000000000002",
            "0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
            "0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
            "0x0000000000000000000000000000000000000000000000000000000000001111",
        ], 0);
        const storageAddress = "0x72943352085ed3fbe3b8ff53a6aef9da8d893ccdab99bd5223d765f1a22735f";
        const { result: { events }, } = await context.createBlock((0, starknet_1.deploy)(context.polkadotApi, constants_1.CONTRACT_ADDRESS, constants_1.TOKEN_CLASS_HASH));
        const classHash = await providerRPC.getClassHashAt(deployedContractAddress, "latest");
        (0, chai_1.expect)((0, util_1.hexFixLength)(classHash, 256, true)).to.equal(constants_1.TOKEN_CLASS_HASH);
        const balance = await providerRPC.getStorageAt(deployedContractAddress, storageAddress, "latest");
        (0, chai_1.expect)(balance).to.equal("0xfffffffffffffffffffffffffffffff");
        (0, chai_1.expect)(events.find(({ event: { section, method } }) => section == "system" && method == "ExtrinsicSuccess")).to.exist;
    });
    it("should execute a transfer", async function () {
        const recepientAddress = "0x00000000000000000000000000000000000000000000000000000000deadbeef";
        const storageKey = "0x4c761778f11aa10fc40190ff3127637fe00dc59bfa557bd4c8beb30a178f016";
        const balanceBefore = await providerRPC.getStorageAt(constants_1.FEE_TOKEN_ADDRESS, storageKey, "latest");
        (0, chai_1.expect)(balanceBefore).to.equal("0x0");
        const nonce = 1;
        const { result: { events }, } = await context.createBlock((0, starknet_1.transfer)(context.polkadotApi, constants_1.CONTRACT_ADDRESS, constants_1.FEE_TOKEN_ADDRESS, recepientAddress, constants_1.MINT_AMOUNT, nonce));
        const balanceAfter = await providerRPC.getStorageAt(constants_1.FEE_TOKEN_ADDRESS, storageKey, "latest");
        (0, chai_1.expect)(balanceAfter).to.equal("0x1");
        (0, chai_1.expect)(events.find(({ event: { section, method } }) => section == "system" && method == "ExtrinsicSuccess")).to.exist;
    });
    it("mint NFTs", async function () {
        const recepientAddress = "0x00000000000000000000000000000000000000000000000000000000deadbeef";
        const storageKey = "0x1a564c2a8ac0aa99f656ca20cae9b7ed3aff27fa129aea20969feb46dd94e73";
        const balanceBefore = await providerRPC.getStorageAt(constants_1.NFT_CONTRACT_ADDRESS, storageKey, "latest");
        (0, chai_1.expect)(balanceBefore).to.equal("0x0");
        const { result: { events }, } = await context.createBlock((0, starknet_1.mintERC721)(context.polkadotApi, constants_1.CONTRACT_ADDRESS, recepientAddress, (0, util_1.numberToHex)(1, 256), 2));
        const balanceAfter = await providerRPC.getStorageAt(constants_1.NFT_CONTRACT_ADDRESS, storageKey, "latest");
        (0, chai_1.expect)(balanceAfter).to.equal("0x1");
        (0, chai_1.expect)(events.find(({ event: { section, method } }) => section == "system" && method == "ExtrinsicSuccess")).to.exist;
    });
    it("deploys ERC20 contract via UDC", async function () {
        const deployedContractAddress = starknet_2.hash.calculateContractAddressFromHash("0x0000000000000000000000000000000000000000000000000000000000000001", "0x0000000000000000000000000000000000000000000000000000000000010000", [
            "0x000000000000000000000000000000000000000000000000000000000000000A",
            "0x000000000000000000000000000000000000000000000000000000000000000B",
            "0x0000000000000000000000000000000000000000000000000000000000000002",
            "0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
            "0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
            "0x0000000000000000000000000000000000000000000000000000000000001111",
        ], 0);
        const { result: { events }, } = await context.createBlock((0, starknet_1.deployTokenContractUDC)(context.polkadotApi, constants_1.CONTRACT_ADDRESS, "0x0000000000000000000000000000000000000000000000000000000000010000", "0x0000000000000000000000000000000000000000000000000000000000000001", false, 3));
        const storageAddress = "0x72943352085ed3fbe3b8ff53a6aef9da8d893ccdab99bd5223d765f1a22735f";
        const classHash = await providerRPC.getClassHashAt(deployedContractAddress, "latest");
        (0, chai_1.expect)((0, util_1.hexFixLength)(classHash, 256, true)).to.equal(constants_1.TOKEN_CLASS_HASH);
        const balance = await providerRPC.getStorageAt(deployedContractAddress, storageAddress, "latest");
        (0, chai_1.expect)(balance).to.equal("0xfffffffffffffffffffffffffffffff");
        (0, chai_1.expect)(events.find(({ event: { section, method } }) => section == "system" && method == "ExtrinsicSuccess")).to.exist;
    });
}, { runNewNode: true });
//# sourceMappingURL=test-starknet-extrinsics.js.map