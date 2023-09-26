"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
require("@keep-starknet-strange/madara-api-augment");
const chai_1 = require("chai");
const starknet_1 = require("starknet");
const setup_dev_tests_1 = require("../../util/setup-dev-tests");
const constants_1 = require("../constants");
(0, setup_dev_tests_1.describeDevMadara)("Starknet RPC - Storage Test", (context) => {
    let providerRPC;
    before(async function () {
        providerRPC = new starknet_1.RpcProvider({
            nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
            retries: 3,
        });
    });
    describe("getStorageAt", async () => {
        it("should return value from the fee contract storage", async function () {
            const value = await providerRPC.getStorageAt(constants_1.FEE_TOKEN_ADDRESS, "0x1d8bbc4f93f5ab9858f6c0c0de2769599fb97511503d5bf2872ef6846f2146f", "latest");
            (0, chai_1.expect)(parseInt(value, 16)).to.be.greaterThan(0);
        });
        it("should return 0 if the storage slot is not set", async function () {
            const value = await providerRPC.getStorageAt(constants_1.FEE_TOKEN_ADDRESS, "0x0000000000000000000000000000000000000000000000000000000000000000", "latest");
            (0, chai_1.expect)(value).to.be.equal("0x0");
        });
        it("should raise if the contract does not exist", async function () {
            const storage = providerRPC.getStorageAt("0x0000000000000000000000000000000000000000000000000000000000000000", "0x0000000000000000000000000000000000000000000000000000000000000000", "latest");
            await (0, chai_1.expect)(storage)
                .to.eventually.be.rejectedWith("20: Contract not found")
                .and.be.an.instanceOf(starknet_1.LibraryError);
        });
    });
});
//# sourceMappingURL=test-storage.js.map