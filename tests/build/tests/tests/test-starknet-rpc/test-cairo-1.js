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
const CAIRO_1_NO_VALIDATE_ACCOUNT = { value: 0 };
(0, setup_dev_tests_1.describeDevMadara)("Starknet RPC - Cairo 1 Test", (context) => {
    let providerRPC;
    before(async function () {
        providerRPC = new starknet_1.RpcProvider({
            nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
            retries: 3,
        });
    });
    describe("Cairo 1 full flow", async () => {
        it("should deploy a Cairo 1 account", async () => {
            const CONSTRUCTOR_CALLDATA = ["0x123"];
            const accountAddress = starknet_1.hash.calculateContractAddressFromHash(constants_1.SALT, constants_1.CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH, CONSTRUCTOR_CALLDATA, 0);
            const account = new starknet_1.Account(providerRPC, accountAddress, constants_1.SIGNER_PRIVATE, "1");
            await (0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, accountAddress, "0xfffffffffffffffffffffffff");
            await (0, block_1.jumpBlocks)(context, 1);
            await account.deploySelf({
                classHash: constants_1.CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH,
                addressSalt: constants_1.SALT,
                constructorCalldata: CONSTRUCTOR_CALLDATA,
            }, { maxFee: "12345678" });
            await (0, block_1.jumpBlocks)(context, 1);
            (0, chai_1.expect)(await providerRPC.getClassHashAt(accountAddress)).to.be.equal(constants_1.CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH);
        });
        it("should declare and deploy erc20 contract then transfer some tokens", async () => {
            const account = new starknet_1.Account(providerRPC, constants_1.CAIRO_1_ACCOUNT_CONTRACT, constants_1.SIGNER_PRIVATE, "1");
            const classHash = "0x4596fa4856bbf13f3448a376d607f8852148b0e6be4b958cde2ca8471a72ede";
            const res = await account.declare({
                casm: constants_1.ERC20_CAIRO_1_CASM,
                contract: constants_1.ERC20_CAIRO_1_SIERRA,
            }, {
                nonce: CAIRO_1_NO_VALIDATE_ACCOUNT.value,
                version: 1,
            });
            CAIRO_1_NO_VALIDATE_ACCOUNT.value += 1;
            await (0, block_1.jumpBlocks)(context, 1);
            const contractClassActual = await providerRPC.getClass(classHash, "latest");
            (0, chai_1.expect)(contractClassActual).to.have.property("entry_points_by_type");
            (0, chai_1.expect)(contractClassActual).to.have.property("sierra_program");
            (0, chai_1.expect)(contractClassActual).to.have.property("contract_class_version");
            (0, chai_1.expect)(contractClassActual).to.have.property("abi");
            (0, chai_1.expect)(res.class_hash).to.be.eq(classHash);
            const deployRes = await account.deploy({
                classHash,
                constructorCalldata: [
                    1,
                    1,
                    1,
                    "0xffffffffffffffffffffffffffffffff",
                    "0xffffffffffffffffffffffffffffffff",
                    constants_1.CAIRO_1_ACCOUNT_CONTRACT,
                ],
            }, {
                nonce: CAIRO_1_NO_VALIDATE_ACCOUNT.value,
                version: 1,
            });
            CAIRO_1_NO_VALIDATE_ACCOUNT.value += 1;
            await (0, block_1.jumpBlocks)(context, 1);
            const balance = await providerRPC.getStorageAt(deployRes.contract_address[0], "0x617243ac31335377b9d26d1a6b02f47b419ad593e1ae67660dd27ec77635906", "latest");
            (0, chai_1.expect)((0, utils_1.toHex)(balance)).to.be.equal("0xffffffffffffffffffffffffffffffff");
            await account.execute([
                {
                    contractAddress: deployRes.contract_address[0],
                    entrypoint: "transfer",
                    calldata: [
                        1,
                        "0xffffffffffffffffffffffffffffffff",
                        0,
                    ],
                },
            ], undefined, {
                nonce: CAIRO_1_NO_VALIDATE_ACCOUNT.value,
                version: 1,
            });
            await (0, block_1.jumpBlocks)(context, 1);
            const balanceSender = await providerRPC.getStorageAt(deployRes.contract_address[0], "0x617243ac31335377b9d26d1a6b02f47b419ad593e1ae67660dd27ec77635906", "latest");
            (0, chai_1.expect)((0, utils_1.toHex)(balanceSender)).to.be.equal("0x0");
            const balanceRecipient = await providerRPC.getStorageAt(deployRes.contract_address[0], "0x753d37842b9cfa00ee311ab2564951681d89ee4d5596e84e74030de35018c8a", "latest");
            (0, chai_1.expect)((0, utils_1.toHex)(balanceRecipient)).to.be.equal("0xffffffffffffffffffffffffffffffff");
        });
    });
}, { runNewNode: true });
//# sourceMappingURL=test-cairo-1.js.map