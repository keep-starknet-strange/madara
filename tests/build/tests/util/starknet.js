"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deployTokenContractUDC = exports.mintERC721 = exports.batchTransfer = exports.transfer = exports.mint = exports.initialize = exports.deploy = exports.declare = exports.sendTransaction = exports.sendTransactionBatchNoValidation = exports.sendTransactionNoValidation = void 0;
const tslib_1 = require("tslib");
require("@keep-starknet-strange/madara-api-augment");
const util_1 = require("@polkadot/util");
const starknet_1 = require("starknet");
const ERC20_json_1 = tslib_1.__importDefault(require("../../cairo-contracts/build/ERC20.json"));
const constants_1 = require("../tests/constants");
const utils_1 = require("./utils");
async function sendTransactionNoValidation(transaction) {
    await transaction.send();
}
exports.sendTransactionNoValidation = sendTransactionNoValidation;
async function sendTransactionBatchNoValidation(api, transactions) {
    await api.tx.utility.batch(transactions).send();
}
exports.sendTransactionBatchNoValidation = sendTransactionBatchNoValidation;
async function sendTransaction(api, transaction) {
    return await new Promise((resolve, reject) => {
        let unsubscribe;
        const SPAWNING_TIME = 500000;
        const timeout = setTimeout(() => {
            reject(new Error("Transaction timeout"));
        }, SPAWNING_TIME);
        let transaction_success_event = false;
        let block_hash;
        transaction
            .send(async ({ events = [], status, dispatchError }) => {
            console.log(`Current status is ${status.type}`);
            if (dispatchError) {
                if (dispatchError.isModule) {
                    const decoded = api.registry.findMetaError(dispatchError.asModule);
                    const { docs, name, section } = decoded;
                    reject(Error(`${section}.${name}: ${docs.join(" ")}`));
                }
                else {
                    reject(Error(dispatchError.toString()));
                }
            }
            if (status.isInBlock) {
                block_hash = status.asInBlock.toHex();
                console.log("Included at block hash", block_hash);
                console.log("Events:");
                events.forEach(({ event: { data, method, section }, phase }) => {
                    console.log("\t", phase.toString(), `: ${section}.${method}`, data.toString());
                    if (section == "system" && method == "ExtrinsicSuccess") {
                        transaction_success_event = true;
                    }
                });
            }
            if (transaction_success_event) {
                if (unsubscribe) {
                    unsubscribe();
                }
                clearTimeout(timeout);
                resolve(block_hash);
            }
        })
            .then((unsub) => {
            unsubscribe = unsub;
        })
            .catch((error) => {
            console.error(error);
            reject(error);
        });
    });
}
exports.sendTransaction = sendTransaction;
function declare(api, contractAddress, tokenClassHash) {
    const tx_declare = {
        version: 1,
        signature: [],
        sender_address: contractAddress,
        nonce: 0,
        callEntrypoint: {
            classHash: tokenClassHash,
            entrypointSelector: null,
            calldata: [],
            storageAddress: contractAddress,
            callerAddress: contractAddress,
        },
        contractClass: {
            program: (0, util_1.u8aWrapBytes)(Buffer.from((0, util_1.stringify)(ERC20_json_1.default.program))),
            entryPointsByType: (0, util_1.u8aWrapBytes)(Buffer.from((0, util_1.stringify)(ERC20_json_1.default.entry_points_by_type))),
        },
    };
    const extrisinc_declare = api.tx.starknet.declare(tx_declare);
    return extrisinc_declare;
}
exports.declare = declare;
function deploy(api, contractAddress, tokenClassHash) {
    const tx_deploy = {
        version: 1,
        signature: [],
        sender_address: contractAddress,
        nonce: 0,
        account_class_hash: tokenClassHash,
        calldata: [
            "0x0000000000000000000000000000000000000000000000000000000000001111",
            "0x0169f135eddda5ab51886052d777a57f2ea9c162d713691b5e04a6d4ed71d47f",
            "0x000000000000000000000000000000000000000000000000000000000000000A",
            "0x0000000000000000000000000000000000000000000000000000000000010000",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
            "0x0000000000000000000000000000000000000000000000000000000000000006",
            "0x000000000000000000000000000000000000000000000000000000000000000A",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
            "0x0000000000000000000000000000000000000000000000000000000000000002",
            "0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
            "0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
            "0x0000000000000000000000000000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        ],
        max_fee: "0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
    };
    const extrisinc_deploy = api.tx.starknet.invoke(tx_deploy);
    return extrisinc_deploy;
}
exports.deploy = deploy;
async function initialize(api, contractAddress, tokenAddress) {
    const tx_initialize = {
        version: 1,
        hash: "",
        signature: [],
        sender_address: contractAddress,
        nonce: 1,
        callEntrypoint: {
            classHash: null,
            entrypointSelector: null,
            calldata: [
                tokenAddress,
                "0x0079dc0da7c54b95f10aa182ad0a46400db63156920adb65eca2654c0945a463",
                5,
                4,
                1413829460,
                1413829460,
                18,
                contractAddress,
            ],
            storageAddress: contractAddress,
            callerAddress: contractAddress,
        },
        contractClass: null,
    };
    const extrisinc_init = api.tx.starknet.invoke(tx_initialize);
    return await sendTransaction(api, extrisinc_init);
}
exports.initialize = initialize;
async function mint(api, contractAddress, tokenAddress, mintAmount) {
    const tx_mint = {
        version: 1,
        hash: "",
        signature: [],
        sender_address: contractAddress,
        nonce: 1,
        callEntrypoint: {
            classHash: null,
            entrypointSelector: null,
            calldata: [
                tokenAddress,
                "0x00151e58b29179122a728eab07c8847e5baf5802379c5db3a7d57a8263a7bd1d",
                "0x0000000000000000000000000000000000000000000000000000000000000003",
                contractAddress,
                mintAmount,
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            ],
            storageAddress: contractAddress,
            callerAddress: contractAddress,
        },
        contractClass: null,
    };
    const extrisinc_mint = api.tx.starknet.invoke(tx_mint);
    return await sendTransaction(api, extrisinc_mint);
}
exports.mint = mint;
function transfer(api, contractAddress, tokenAddress, recipientAddress, transferAmount, nonce) {
    const tx_transfer = {
        version: 1,
        signature: [],
        sender_address: contractAddress,
        nonce: (0, utils_1.numberToU832Bytes)(nonce ? nonce : 0),
        calldata: [
            tokenAddress,
            "0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e",
            "0x0000000000000000000000000000000000000000000000000000000000000003",
            recipientAddress,
            transferAmount,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        ],
    };
    const extrisinc_transfer = api.tx.starknet.invoke(tx_transfer);
    return extrisinc_transfer;
}
exports.transfer = transfer;
function batchTransfer(api, contractAddress, tokenAddress, recipientAddress, transferAmount) {
    const tx_transfer = {
        version: 1,
        signature: [],
        sender_address: contractAddress,
        nonce: 0,
        calldata: [
            tokenAddress,
            "0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e",
            "0x0000000000000000000000000000000000000000000000000000000000000003",
            recipientAddress,
            transferAmount,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        ],
    };
    const extrisinc_transfer = api.tx.starknet.invoke(tx_transfer);
    const extrisinc_transfers = Array(200).fill(extrisinc_transfer);
    return extrisinc_transfers;
}
exports.batchTransfer = batchTransfer;
function mintERC721(api, senderAddress, recipientAddress, tokenID, nonce) {
    const tx_mint = {
        version: 1,
        signature: [],
        sender_address: senderAddress,
        nonce: (0, utils_1.numberToU832Bytes)(nonce ? nonce : 0),
        calldata: [
            constants_1.NFT_CONTRACT_ADDRESS,
            "0x" + starknet_1.hash.getSelectorFromName("mint").slice(2).padStart(64, "0"),
            "0x0000000000000000000000000000000000000000000000000000000000000003",
            recipientAddress,
            tokenID,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        ],
    };
    return api.tx.starknet.invoke(tx_mint);
}
exports.mintERC721 = mintERC721;
function deployTokenContractUDC(api, contractAddress, classHash, salt, unique, nonce) {
    const tx_udc_deploy = {
        version: 1,
        signature: [],
        sender_address: contractAddress,
        nonce: (0, utils_1.numberToU832Bytes)(nonce ? nonce : 0),
        calldata: [
            constants_1.UDC_CONTRACT_ADDRESS,
            "0x01987cbd17808b9a23693d4de7e246a443cfe37e6e7fbaeabd7d7e6532b07c3d",
            (0, util_1.numberToHex)(10, 256),
            classHash,
            salt,
            unique ? (0, util_1.numberToHex)(1, 256) : (0, util_1.numberToHex)(0, 256),
            "0x0000000000000000000000000000000000000000000000000000000000000006",
            "0x000000000000000000000000000000000000000000000000000000000000000A",
            "0x000000000000000000000000000000000000000000000000000000000000000B",
            "0x0000000000000000000000000000000000000000000000000000000000000002",
            "0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
            "0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
            "0x0000000000000000000000000000000000000000000000000000000000001111",
        ],
    };
    const extrisinc_udc_deploy = api.tx.starknet.invoke(tx_udc_deploy);
    return extrisinc_udc_deploy;
}
exports.deployTokenContractUDC = deployTokenContractUDC;
//# sourceMappingURL=starknet.js.map