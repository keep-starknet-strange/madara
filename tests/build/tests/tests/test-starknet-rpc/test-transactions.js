"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
require("@keep-starknet-strange/madara-api-augment");
const chai_1 = require("chai");
const starknet_1 = require("starknet");
const block_1 = require("../../util/block");
const setup_dev_tests_1 = require("../../util/setup-dev-tests");
const utils_1 = require("../../util/utils");
const constants_1 = require("../constants");
const util_1 = require("@polkadot/util");
let ARGENT_CONTRACT_NONCE = { value: 0 };
const CAIRO_1_NO_VALIDATE_ACCOUNT = { value: 0 };
(0, setup_dev_tests_1.describeDevMadara)("Starknet RPC - Transactions Test", (context) => {
    let providerRPC;
    before(async function () {
        providerRPC = new starknet_1.RpcProvider({
            nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
            retries: 3,
        });
    });
    describe("getTransactionByBlockIdAndIndex", async () => {
        it("should returns 1 transaction", async function () {
            await context.createBlock((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT));
            const tx = await providerRPC.getTransactionByBlockIdAndIndex("latest", 0);
            (0, chai_1.expect)(tx).to.not.be.undefined;
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
        it("should throws block not found error", async function () {
            const transaction = providerRPC.getTransactionByBlockIdAndIndex("0x123", 2);
            await (0, chai_1.expect)(transaction)
                .to.eventually.be.rejectedWith("24: Block not found")
                .and.be.an.instanceOf(starknet_1.LibraryError);
        });
        it("should throws invalid transaction index error", async function () {
            await context.createBlock(undefined, {
                parentHash: undefined,
                finalize: true,
            });
            const latestBlockCreated = await providerRPC.getBlockHashAndNumber();
            const transaction = providerRPC.getTransactionByBlockIdAndIndex(latestBlockCreated.block_hash, 2);
            await (0, chai_1.expect)(transaction)
                .to.eventually.be.rejectedWith("27: Invalid transaction index in a block")
                .and.be.an.instanceOf(starknet_1.LibraryError);
        });
    });
    describe("addInvokeTransaction", async () => {
        it("should invoke successfully", async function () {
            const account = new starknet_1.Account(providerRPC, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.SIGNER_PRIVATE);
            await account.execute({
                contractAddress: constants_1.FEE_TOKEN_ADDRESS,
                entrypoint: "transfer",
                calldata: ["0xdeadbeef", "0x123", "0x0"],
            }, undefined, {
                nonce: ARGENT_CONTRACT_NONCE.value,
            });
            ARGENT_CONTRACT_NONCE.value += 1;
            await (0, block_1.jumpBlocks)(context, 1);
            const balance = await providerRPC.getStorageAt(constants_1.FEE_TOKEN_ADDRESS, "0x04c761778f11aa10fc40190ff3127637fe00dc59bfa557bd4c8beb30a178f016", "latest");
            (0, chai_1.expect)((0, utils_1.toHex)(balance)).to.be.equal("0x123");
        });
        it("should deploy ERC20 via UDC", async function () {
            const account = new starknet_1.Account(providerRPC, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.SIGNER_PRIVATE);
            const calldata = [
                (0, util_1.numberToHex)(1, 256),
                (0, util_1.numberToHex)(1, 256),
                (0, util_1.numberToHex)(18, 256),
                (0, util_1.numberToHex)(42, 256),
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                "0xdeadbeef",
            ];
            const deployedContractAddress = starknet_1.hash.calculateContractAddressFromHash(constants_1.SALT, constants_1.TOKEN_CLASS_HASH, calldata, 0);
            await account.execute({
                contractAddress: constants_1.UDC_CONTRACT_ADDRESS,
                entrypoint: "deployContract",
                calldata: [constants_1.TOKEN_CLASS_HASH, constants_1.SALT, "0x0", "0x6", ...calldata],
            }, undefined, {
                nonce: ARGENT_CONTRACT_NONCE.value,
            });
            ARGENT_CONTRACT_NONCE.value += 1;
            await (0, block_1.jumpBlocks)(context, 1);
            const balance = await providerRPC.getStorageAt(deployedContractAddress, "0x04c761778f11aa10fc40190ff3127637fe00dc59bfa557bd4c8beb30a178f016", "latest");
            (0, chai_1.expect)((0, utils_1.toHex)(balance)).to.be.equal("0x2a");
        });
        it("should fail on invalid nonce", async function () {
            const invalid_nonce = { value: ARGENT_CONTRACT_NONCE.value + 1 };
            let balance = await providerRPC.getStorageAt(constants_1.FEE_TOKEN_ADDRESS, "0x72943352085ed3fbe3b8ff53a6aef9da8d893ccdab99bd5223d765f1a22735f", "latest");
            (0, chai_1.expect)((0, utils_1.toHex)(balance)).to.be.equal("0x0");
            await (0, utils_1.rpcTransfer)(providerRPC, invalid_nonce, constants_1.TEST_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT),
                await (0, block_1.jumpBlocks)(context, 1);
            balance = await providerRPC.getStorageAt(constants_1.FEE_TOKEN_ADDRESS, "0x72943352085ed3fbe3b8ff53a6aef9da8d893ccdab99bd5223d765f1a22735f", "latest");
            (0, chai_1.expect)((0, utils_1.toHex)(balance)).to.be.equal("0x0");
            await (0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.TEST_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT),
                await (0, block_1.jumpBlocks)(context, 1);
            balance = await providerRPC.getStorageAt(constants_1.FEE_TOKEN_ADDRESS, "0x72943352085ed3fbe3b8ff53a6aef9da8d893ccdab99bd5223d765f1a22735f", "latest");
            (0, chai_1.expect)((0, utils_1.toHex)(balance)).to.be.equal("0x2");
            ARGENT_CONTRACT_NONCE.value += 1;
        });
    });
    describe("addDeployAccountTransaction", async () => {
        it("should deploy successfully", async function () {
            const selector = starknet_1.hash.getSelectorFromName("initialize");
            const calldata = [
                constants_1.ARGENT_ACCOUNT_CLASS_HASH,
                selector,
                2,
                constants_1.SIGNER_PUBLIC,
                0,
            ];
            const deployedContractAddress = starknet_1.hash.calculateContractAddressFromHash(constants_1.SALT, constants_1.ARGENT_PROXY_CLASS_HASH, calldata, 0);
            await (0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, deployedContractAddress, constants_1.DEPLOY_ACCOUNT_COST);
            await (0, block_1.jumpBlocks)(context, 1);
            const invocationDetails = {
                nonce: "0x0",
                maxFee: "0x1111111111111111111111",
                version: "0x1",
            };
            const signer = new starknet_1.Signer(constants_1.SIGNER_PRIVATE);
            const signature = await signer.signDeployAccountTransaction({
                classHash: constants_1.ARGENT_PROXY_CLASS_HASH,
                contractAddress: deployedContractAddress,
                constructorCalldata: calldata,
                addressSalt: constants_1.SALT,
                maxFee: invocationDetails.maxFee,
                version: invocationDetails.version,
                chainId: starknet_1.constants.StarknetChainId.SN_GOERLI,
                nonce: invocationDetails.nonce,
            });
            const txDeployAccount = {
                signature: signature,
                contractAddress: deployedContractAddress,
                addressSalt: constants_1.SALT,
                classHash: constants_1.ARGENT_PROXY_CLASS_HASH,
                constructorCalldata: calldata,
            };
            await providerRPC.deployAccountContract(txDeployAccount, invocationDetails);
            await (0, block_1.createAndFinalizeBlock)(context.polkadotApi);
            const accountContractClassHash = await providerRPC.getClassHashAt(deployedContractAddress);
            (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(accountContractClassHash)).to.be.equal(constants_1.ARGENT_PROXY_CLASS_HASH);
        });
    });
    describe("estimateFee", async () => {
        it("should estimate fee", async function () {
            const tx = {
                contractAddress: constants_1.ACCOUNT_CONTRACT,
                calldata: [
                    constants_1.TEST_CONTRACT_ADDRESS,
                    "0x36fa6de2810d05c3e1a0ebe23f60b9c2f4629bbead09e5a9704e1c5632630d5",
                    "0x0",
                ],
                signature: [],
            };
            const nonce = await providerRPC.getNonceForAddress(constants_1.ACCOUNT_CONTRACT, "latest");
            const txDetails = {
                nonce: nonce,
            };
            const invocation = {
                type: "INVOKE_FUNCTION",
                ...tx,
                ...txDetails,
            };
            const fee_estimates = await providerRPC.getEstimateFeeBulk([invocation], {
                blockIdentifier: "latest",
            });
            (0, chai_1.expect)(fee_estimates[0].overall_fee > 0n).to.be.equal(true);
            (0, chai_1.expect)(fee_estimates[0].gas_consumed > 0n).to.be.equal(true);
        });
        it("should fail estimate fee if version is 1", async function () {
            const tx = {
                contractAddress: constants_1.ACCOUNT_CONTRACT,
                calldata: [
                    constants_1.TEST_CONTRACT_ADDRESS,
                    "0x36fa6de2810d05c3e1a0ebe23f60b9c2f4629bbead09e5a9704e1c5632630d5",
                    "0x0",
                ],
                signature: [],
            };
            const nonce = await providerRPC.getNonceForAddress(constants_1.ACCOUNT_CONTRACT, "latest");
            const txDetails = {
                nonce: nonce,
                version: 1,
            };
            const invocation = {
                type: "INVOKE_FUNCTION",
                ...tx,
                ...txDetails,
            };
            await (0, chai_1.expect)(providerRPC.getEstimateFeeBulk([invocation], {
                blockIdentifier: "latest",
            }))
                .to.eventually.be.rejectedWith("61: The transaction version is not supported")
                .and.be.an.instanceOf(starknet_1.LibraryError);
        });
        it("should raise if contract does not exist", async function () {
            const tx = {
                contractAddress: constants_1.ACCOUNT_CONTRACT,
                calldata: [
                    "0x000000000000000000000000000000000000000000000000000000000000DEAD",
                    "0x36fa6de2810d05c3e1a0ebe23f60b9c2f4629bbead09e5a9704e1c5632630d5",
                    "0x0",
                ],
                signature: [],
            };
            const nonce = await providerRPC.getNonceForAddress(constants_1.ACCOUNT_CONTRACT, "latest");
            const txDetails = {
                nonce: nonce,
                version: "0x1",
            };
            const invocation = {
                type: "INVOKE_FUNCTION",
                ...tx,
                ...txDetails,
            };
            const fee_estimates = providerRPC.getEstimateFeeBulk([invocation], {
                blockIdentifier: "latest",
            });
            (0, chai_1.expect)(fee_estimates)
                .to.eventually.be.rejectedWith("40: Contract error")
                .and.be.an.instanceOf(starknet_1.LibraryError);
        });
        it("should estimate fees for multiple invocations", async function () {
            const tx = {
                contractAddress: constants_1.ACCOUNT_CONTRACT,
                calldata: [
                    constants_1.TEST_CONTRACT_ADDRESS,
                    "0x36fa6de2810d05c3e1a0ebe23f60b9c2f4629bbead09e5a9704e1c5632630d5",
                    "0x0",
                ],
                signature: [],
            };
            const nonce = await providerRPC.getNonceForAddress(constants_1.ACCOUNT_CONTRACT, "latest");
            const txDetails = {
                nonce: nonce,
            };
            const invocation = {
                type: "INVOKE_FUNCTION",
                ...tx,
                ...txDetails,
            };
            const fee_estimates = await providerRPC.getEstimateFeeBulk([invocation, invocation], {
                blockIdentifier: "latest",
            });
            (0, chai_1.expect)(fee_estimates[0].overall_fee > 0n).to.be.equal(true);
            (0, chai_1.expect)(fee_estimates[0].gas_consumed > 0n).to.be.equal(true);
            (0, chai_1.expect)(fee_estimates[1].overall_fee > 0n).to.be.equal(true);
            (0, chai_1.expect)(fee_estimates[1].gas_consumed > 0n).to.be.equal(true);
        });
        it("should return empty array if no invocations", async function () {
            const fee_estimates = await providerRPC.getEstimateFeeBulk([], {
                blockIdentifier: "latest",
            });
            (0, chai_1.expect)(fee_estimates.length == 0).to.be.equal(true);
        });
        it("should be possible for an account to estimateInvokeFee", async function () {
            const account = new starknet_1.Account(providerRPC, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.SIGNER_PRIVATE);
            const { suggestedMaxFee } = await account.estimateInvokeFee({
                contractAddress: constants_1.TEST_CONTRACT_ADDRESS,
                entrypoint: "test_storage_var",
                calldata: [],
            });
            (0, chai_1.expect)(suggestedMaxFee > 0n).to.be.equal(true);
        });
        it("should be possible for an account to estimateDeclareFee", async function () {
            const account = new starknet_1.Account(providerRPC, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.SIGNER_PRIVATE);
            const { suggestedMaxFee } = await account.estimateDeclareFee({
                contract: constants_1.ERC20_CONTRACT,
            });
            (0, chai_1.expect)(suggestedMaxFee > 0n).to.be.equal(true);
        });
        it("should be possible for an account to estimateAccountDeployFee", async function () {
            const account = new starknet_1.Account(providerRPC, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.SIGNER_PRIVATE);
            const { suggestedMaxFee } = await account.estimateAccountDeployFee({
                classHash: constants_1.CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH,
                constructorCalldata: ["0x123"],
                addressSalt: constants_1.SALT,
            });
            (0, chai_1.expect)(suggestedMaxFee > 0n).to.be.equal(true);
        });
    });
    describe("addDeclareTransaction", async () => {
        it("should set class at given class hash (legacy)", async function () {
            const account = new starknet_1.Account(providerRPC, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.SIGNER_PRIVATE);
            const classHash = "0x372ee6669dc86563007245ed7343d5180b96221ce28f44408cff2898038dbd4";
            const res = await account.declare({
                classHash: classHash,
                contract: constants_1.ERC20_CONTRACT,
            }, { nonce: ARGENT_CONTRACT_NONCE.value });
            ARGENT_CONTRACT_NONCE.value += 1;
            await (0, block_1.jumpBlocks)(context, 1);
            const contractClassActual = await providerRPC.getClass(classHash, "latest");
            (0, chai_1.expect)(contractClassActual.entry_points_by_type).to.deep.equal(constants_1.ERC20_CONTRACT.entry_points_by_type);
            (0, chai_1.expect)(res.class_hash).to.be.eq(classHash);
        });
        it("should set class at given class hash and deploy a new contract (cairo 1)", async function () {
            const account = new starknet_1.Account(providerRPC, constants_1.CAIRO_1_ACCOUNT_CONTRACT, "0x123");
            const classHash = "0x9cf5ef6166edaa87767d05bbfd54ad02fd110028597343a200e82949ce05cf";
            const res = await account.declare({
                casm: constants_1.TEST_CAIRO_1_CASM,
                contract: constants_1.TEST_CAIRO_1_SIERRA,
            }, {
                nonce: CAIRO_1_NO_VALIDATE_ACCOUNT.value,
            });
            CAIRO_1_NO_VALIDATE_ACCOUNT.value += 1;
            await (0, block_1.jumpBlocks)(context, 1);
            const contractClassActual = await providerRPC.getClass(classHash, "latest");
            (0, chai_1.expect)(contractClassActual).to.have.property("entry_points_by_type");
            (0, chai_1.expect)(contractClassActual).to.have.property("sierra_program");
            (0, chai_1.expect)(contractClassActual).to.have.property("contract_class_version");
            (0, chai_1.expect)(contractClassActual).to.have.property("abi");
            (0, chai_1.expect)(res.class_hash).to.be.eq(classHash);
        });
        it("should fail to declare duplicate class", async function () {
            const account = new starknet_1.Account(providerRPC, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.SIGNER_PRIVATE);
            const classHash = "0x372ee6669dc86563007245ed7343d5180b96221ce28f44408cff2898038dbd4";
            await (0, chai_1.expect)(account.declare({
                classHash: classHash,
                contract: constants_1.ERC20_CONTRACT,
            }, {
                nonce: ARGENT_CONTRACT_NONCE.value,
            })).to.be.rejectedWith("51: Class already declared");
        });
    });
    describe("pendingTransactions", async () => {
        it("should return all the starknet invoke transactions", async function () {
            await (0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT);
            const txs = await providerRPC.getPendingTransactions();
            (0, chai_1.expect)(txs.length).equals(1);
            (0, chai_1.expect)(txs[0]).to.include({ type: "INVOKE" });
            (0, chai_1.expect)(txs[0]).that.includes.all.keys([
                "transaction_hash",
                "max_fee",
                "version",
                "signature",
                "nonce",
                "type",
                "sender_address",
                "calldata",
            ]);
            await (0, block_1.jumpBlocks)(context, 10);
        });
        it("should return all starknet declare transactions", async function () {
            const account = new starknet_1.Account(providerRPC, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.SIGNER_PRIVATE);
            const classHash = "0x077cc28ed3c661419fda16bf120fb81f1f8f28617f5543b05a86d63b0926bbf4";
            await account.declare({
                classHash: classHash,
                contract: constants_1.ERC721_CONTRACT,
            }, { nonce: ARGENT_CONTRACT_NONCE.value });
            const txs = await providerRPC.getPendingTransactions();
            (0, chai_1.expect)(txs.length).equals(1);
            (0, chai_1.expect)(txs[0]).to.include({ type: "DECLARE" });
            (0, chai_1.expect)(txs[0]).that.includes.all.keys([
                "sender_address",
                "class_hash",
                "max_fee",
                "nonce",
                "signature",
                "transaction_hash",
                "type",
                "version",
            ]);
            await (0, block_1.jumpBlocks)(context, 10);
        });
        it("should return all starknet deploy_account transactions", async function () {
            const selector = starknet_1.hash.getSelectorFromName("initialize");
            const calldata = [
                constants_1.ARGENT_ACCOUNT_CLASS_HASH,
                selector,
                2,
                constants_1.SIGNER_PUBLIC,
                0,
            ];
            const deployedContractAddress = starknet_1.hash.calculateContractAddressFromHash(constants_1.SALT, constants_1.ARGENT_PROXY_CLASS_HASH, calldata, 0);
            const invocationDetails = {
                nonce: "0x0",
                maxFee: "0x1111111111111111111111",
                version: "0x1",
            };
            const signer = new starknet_1.Signer(constants_1.SIGNER_PRIVATE);
            const signature = await signer.signDeployAccountTransaction({
                classHash: constants_1.ARGENT_PROXY_CLASS_HASH,
                contractAddress: deployedContractAddress,
                constructorCalldata: calldata,
                addressSalt: constants_1.SALT,
                maxFee: invocationDetails.maxFee,
                version: invocationDetails.version,
                chainId: starknet_1.constants.StarknetChainId.SN_GOERLI,
                nonce: invocationDetails.nonce,
            });
            const txDeployAccount = {
                signature: signature,
                contractAddress: deployedContractAddress,
                addressSalt: constants_1.SALT,
                classHash: constants_1.ARGENT_PROXY_CLASS_HASH,
                constructorCalldata: calldata,
            };
            await providerRPC.deployAccountContract(txDeployAccount, invocationDetails);
            const txs = await providerRPC.getPendingTransactions();
            (0, chai_1.expect)(txs.length).equals(1);
            (0, chai_1.expect)(txs[0]).to.include({ type: "DEPLOY_ACCOUNT" });
            (0, chai_1.expect)(txs[0]).that.includes.all.keys([
                "class_hash",
                "constructor_calldata",
                "contract_address_salt",
                "max_fee",
                "nonce",
                "signature",
                "transaction_hash",
                "type",
                "version",
            ]);
            await (0, block_1.jumpBlocks)(context, 10);
        });
        it("should return transactions from the ready and future queues", async function () {
            const transactionNonceOffset = 1000;
            await (0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT);
            await (0, utils_1.rpcTransfer)(providerRPC, { value: ARGENT_CONTRACT_NONCE.value + transactionNonceOffset }, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT);
            const readyExtrinsics = await context.polkadotApi.rpc.author.pendingExtrinsics();
            const readyTxs = readyExtrinsics.map((pending) => {
                const obj = pending.toHuman();
                return {
                    type: obj.method.method.toUpperCase(),
                    nonce: (0, utils_1.toHex)(obj.method.args.transaction.nonce),
                };
            });
            const txs = await providerRPC.getPendingTransactions();
            (0, chai_1.expect)(readyExtrinsics.length).to.be.equal(1);
            (0, chai_1.expect)(txs.length).to.be.equal(2);
            (0, chai_1.expect)(readyTxs[0]).to.include({
                type: "INVOKE",
                nonce: (0, utils_1.toHex)(ARGENT_CONTRACT_NONCE.value - 1),
            });
            (0, chai_1.expect)(txs[0]).to.include({
                type: "INVOKE",
                nonce: (0, utils_1.toHex)(ARGENT_CONTRACT_NONCE.value - 1),
            });
            (0, chai_1.expect)(txs[1]).to.include({
                type: "INVOKE",
                nonce: (0, utils_1.toHex)(ARGENT_CONTRACT_NONCE.value + transactionNonceOffset),
            });
            await (0, block_1.jumpBlocks)(context, 10);
        });
    });
    describe("getTransactionByHash", () => {
        it("should return a transaction", async function () {
            await (0, block_1.createAndFinalizeBlock)(context.polkadotApi);
            const b = await context.createBlock((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT), {
                finalize: true,
            });
            const r = await providerRPC.getTransactionByHash(b.result.hash);
            (0, chai_1.expect)(r).to.not.be.undefined;
        });
        it("should return transaction hash not found", async function () {
            await context.createBlock((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT));
            const transaction = providerRPC.getTransactionByHash("0x1234");
            await (0, chai_1.expect)(transaction)
                .to.eventually.be.rejectedWith("25: Transaction hash not found")
                .and.be.an.instanceOf(starknet_1.LibraryError);
        });
        it("should return transaction hash not found when a transaction is in the pool", async function () {
            await (0, block_1.createAndFinalizeBlock)(context.polkadotApi);
            const b = await (0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT);
            const transaction = providerRPC.getTransactionByHash(b.transaction_hash);
            await (0, chai_1.expect)(transaction)
                .to.eventually.be.rejectedWith("25: Transaction hash not found")
                .and.be.an.instanceOf(starknet_1.LibraryError);
        });
    });
    describe("getTransactionReceipt", () => {
        it("should return a receipt", async function () {
            await (0, block_1.createAndFinalizeBlock)(context.polkadotApi);
            const b = await context.createBlock((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT), {
                finalize: true,
            });
            const block_hash_and_number = await providerRPC.getBlockHashAndNumber();
            const r = await providerRPC.getTransactionReceipt(b.result.hash);
            (0, chai_1.expect)(r).to.not.be.undefined;
            (0, chai_1.expect)(r.block_hash).to.be.equal(block_hash_and_number.block_hash);
            (0, chai_1.expect)(r.block_number).to.be.equal(block_hash_and_number.block_number);
        });
        it("should return transaction hash not found", async function () {
            await context.createBlock((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT));
            const transaction = providerRPC.getTransactionReceipt("0x1234");
            await (0, chai_1.expect)(transaction)
                .to.eventually.be.rejectedWith("25: Transaction hash not found")
                .and.be.an.instanceOf(starknet_1.LibraryError);
        });
    });
}, { runNewNode: true });
//# sourceMappingURL=test-transactions.js.map