"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
require("@keep-starknet-strange/madara-api-augment");
const chai_1 = require("chai");
const starknet_1 = require("starknet");
const setup_dev_tests_1 = require("../../util/setup-dev-tests");
const utils_1 = require("../../util/utils");
const constants_1 = require("../constants");
const util_1 = require("@polkadot/util");
let ARGENT_CONTRACT_NONCE = { value: 0 };
(0, setup_dev_tests_1.describeDevMadara)("Starknet RPC - State Update Test", (context) => {
    let providerRPC;
    before(async function () {
        providerRPC = new starknet_1.RpcProvider({
            nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
            retries: 3,
        });
    });
    describe("getStateUpdate", async () => {
        it("should return latest block state update", async function () {
            await context.createBlock((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT), {
                finalize: true,
            });
            const stateUpdate = await providerRPC.getStateUpdate("latest");
            const latestBlock = await providerRPC.getBlockHashAndNumber();
            (0, chai_1.expect)(stateUpdate).to.not.be.undefined;
            (0, util_1.assert)("block_hash" in stateUpdate, "block_hash is not in stateUpdate which means it's still pending");
            (0, chai_1.expect)(stateUpdate.block_hash).to.be.equal(latestBlock.block_hash);
            (0, chai_1.expect)(stateUpdate.state_diff).to.deep.equal({
                storage_diffs: [],
                deprecated_declared_classes: [],
                declared_classes: [],
                deployed_contracts: [],
                replaced_classes: [],
                nonces: [],
            });
        });
        it("should return anterior block state update", async function () {
            const anteriorBlock = await providerRPC.getBlockHashAndNumber();
            await context.createBlock((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT), {
                finalize: true,
            });
            const stateUpdate = await providerRPC.getStateUpdate(anteriorBlock.block_hash);
            (0, chai_1.expect)(stateUpdate).to.not.be.undefined;
            (0, util_1.assert)("block_hash" in stateUpdate, "block_hash is not in stateUpdate which means it's still pending");
            (0, chai_1.expect)(stateUpdate.block_hash).to.be.equal(anteriorBlock.block_hash);
            (0, chai_1.expect)(stateUpdate.state_diff).to.deep.equal({
                storage_diffs: [],
                deprecated_declared_classes: [],
                declared_classes: [],
                deployed_contracts: [],
                replaced_classes: [],
                nonces: [],
            });
        });
        it("should throw block not found error", async function () {
            const transaction = providerRPC.getStateUpdate("0x123");
            await (0, chai_1.expect)(transaction)
                .to.eventually.be.rejectedWith("24: Block not found")
                .and.be.an.instanceOf(starknet_1.LibraryError);
        });
    });
}, { runNewNode: true });
//# sourceMappingURL=test-state-update.js.map