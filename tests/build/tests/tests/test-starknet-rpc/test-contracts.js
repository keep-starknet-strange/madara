"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
require("@keep-starknet-strange/madara-api-augment");
const chai_1 = require("chai");
const starknet_1 = require("starknet");
const pako_1 = require("pako");
const setup_dev_tests_1 = require("../../util/setup-dev-tests");
const constants_1 = require("../constants");
function atobUniversal(a) {
    return starknet_1.encode.IS_BROWSER
        ? stringToArrayBuffer(atob(a))
        : Buffer.from(a, "base64");
}
function stringToArrayBuffer(s) {
    return Uint8Array.from(s, (c) => c.charCodeAt(0));
}
function decompressProgram(base64) {
    if (Array.isArray(base64))
        return base64;
    return starknet_1.encode.arrayBufferToString((0, pako_1.ungzip)(atobUniversal(base64)));
}
(0, setup_dev_tests_1.describeDevMadara)("Starknet RPC - Contracts Test", (context) => {
    let providerRPC;
    before(async function () {
        providerRPC = new starknet_1.RpcProvider({
            nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
            retries: 3,
        });
    });
    describe("call", async () => {
        it("should return calldata on return_result entrypoint", async function () {
            const call = await providerRPC.callContract({
                contractAddress: constants_1.TEST_CONTRACT_ADDRESS,
                entrypoint: "return_result",
                calldata: ["0x19"],
            }, "latest");
            (0, chai_1.expect)(call.result).to.contain("0x19");
        });
        it("should raise with invalid entrypoint", async () => {
            const callResult = providerRPC.callContract({
                contractAddress: constants_1.TEST_CONTRACT_ADDRESS,
                entrypoint: "return_result_WRONG",
                calldata: ["0x19"],
            }, "latest");
            await (0, chai_1.expect)(callResult)
                .to.eventually.be.rejectedWith("40: Contract error")
                .and.be.an.instanceOf(starknet_1.LibraryError);
        });
    });
    describe("getClassAt", async () => {
        it("should not be undefined", async function () {
            const contract_class = await providerRPC.getClassAt(constants_1.TEST_CONTRACT_ADDRESS, "latest");
            (0, chai_1.expect)(contract_class).to.not.be.undefined;
            (0, chai_1.expect)(contract_class.entry_points_by_type).to.deep.equal(constants_1.TEST_CONTRACT.entry_points_by_type);
        });
    });
    describe("getClassHashAt", async () => {
        it("should return correct class hashes for account and test contract", async function () {
            const account_contract_class_hash = await providerRPC.getClassHashAt(constants_1.ACCOUNT_CONTRACT, "latest");
            (0, chai_1.expect)(account_contract_class_hash).to.not.be.undefined;
            (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(account_contract_class_hash)).to.be.equal(constants_1.ACCOUNT_CONTRACT_CLASS_HASH);
            const test_contract_class_hash = await providerRPC.getClassHashAt(constants_1.TEST_CONTRACT_ADDRESS, "latest");
            (0, chai_1.expect)(test_contract_class_hash).to.not.be.undefined;
            (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(test_contract_class_hash)).to.be.equal(constants_1.TEST_CONTRACT_CLASS_HASH);
        });
        it("should raise with invalid block id", async () => {
            const classHash = providerRPC.getClassHashAt(constants_1.TEST_CONTRACT_ADDRESS, "0x123");
            await (0, chai_1.expect)(classHash)
                .to.eventually.be.rejectedWith("24: Block not found")
                .and.be.an.instanceOf(starknet_1.LibraryError);
        });
        it("should raise with invalid contract address", async () => {
            const classHash = providerRPC.getClassHashAt("0x123", "latest");
            await (0, chai_1.expect)(classHash)
                .to.eventually.be.rejectedWith("20: Contract not found")
                .and.be.an.instanceOf(starknet_1.LibraryError);
        });
    });
    describe("getClass", async () => {
        it("should return ERC_20 contract at class 0x10000", async function () {
            const contract_class = (await providerRPC.getClass(constants_1.TOKEN_CLASS_HASH, "latest"));
            (0, chai_1.expect)(contract_class.entry_points_by_type).to.deep.equal(constants_1.ERC20_CONTRACT.entry_points_by_type);
            const program = starknet_1.json.parse(decompressProgram(contract_class.program));
        });
    });
});
//# sourceMappingURL=test-contracts.js.map