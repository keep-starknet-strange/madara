"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.rpcTransfer = exports.cleanHex = exports.starknetKeccak = exports.numberToU832Bytes = exports.toHex = void 0;
const starknet_1 = require("starknet");
const constants_1 = require("../tests/constants");
const util_1 = require("@polkadot/util");
function toHex(value) {
    return starknet_1.num.toHex(value);
}
exports.toHex = toHex;
function numberToU832Bytes(value) {
    return (0, util_1.numberToU8a)(value, 256);
}
exports.numberToU832Bytes = numberToU832Bytes;
function starknetKeccak(value) {
    return starknet_1.hash.starknetKeccak(value);
}
exports.starknetKeccak = starknetKeccak;
function cleanHex(value) {
    const cleaned = starknet_1.number.cleanHex(value);
    if (cleaned === "0x") {
        return "0x0";
    }
    return cleaned;
}
exports.cleanHex = cleanHex;
async function rpcTransfer(providerRPC, nonce, recipient, transferAmount, maxFee) {
    const account = new starknet_1.Account(providerRPC, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.SIGNER_PRIVATE);
    const invokeResponse = account.execute({
        contractAddress: constants_1.FEE_TOKEN_ADDRESS,
        entrypoint: "transfer",
        calldata: [recipient, transferAmount, "0x0"],
    }, undefined, {
        nonce: nonce.value,
        maxFee: maxFee ?? "12345678",
    });
    nonce.value++;
    return invokeResponse;
}
exports.rpcTransfer = rpcTransfer;
//# sourceMappingURL=utils.js.map