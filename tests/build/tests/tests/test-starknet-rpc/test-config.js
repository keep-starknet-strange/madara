"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
require("@keep-starknet-strange/madara-api-augment");
const chai_1 = require("chai");
const starknet_1 = require("starknet");
const setup_dev_tests_1 = require("../../util/setup-dev-tests");
const constants_1 = require("../constants");
(0, setup_dev_tests_1.describeDevMadara)("Starknet RPC - Config Test", (context) => {
    let providerRPC;
    before(async function () {
        providerRPC = new starknet_1.RpcProvider({
            nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
            retries: 3,
        });
    });
    describe("getChainId", async () => {
        it("should return the correct value", async function () {
            const chainId = await providerRPC.getChainId();
            (0, chai_1.expect)(chainId).to.not.be.undefined;
            (0, chai_1.expect)(chainId).to.be.equal(constants_1.CHAIN_ID_STARKNET_TESTNET);
        });
    });
});
//# sourceMappingURL=test-config.js.map