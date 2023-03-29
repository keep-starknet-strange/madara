"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
exports.__esModule = true;
exports.transfer = exports.mint = exports.initialize = exports.deploy = exports.declare = exports.init = void 0;
var api_1 = require("@polkadot/api");
var keyring_1 = require("@polkadot/keyring");
var util_1 = require("@polkadot/util");
var erc20_json_1 = require("../../ressources/erc20.json");
// import "@polkadot/api/augment";
// import "@polkadot/types/augment";
// import * as definitions from "../interfaces/definitions";
function init() {
    return __awaiter(this, void 0, void 0, function () {
        var wsProvider, api, keyring, user;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    wsProvider = new api_1.WsProvider();
                    return [4 /*yield*/, api_1.ApiPromise.create({ provider: wsProvider /* types */ })];
                case 1:
                    api = _a.sent();
                    return [4 /*yield*/, api.isReady];
                case 2:
                    _a.sent();
                    console.log(api.genesisHash.toHex());
                    keyring = new keyring_1.Keyring({ type: "sr25519" });
                    user = keyring.addFromUri("//Alice");
                    return [2 /*return*/, { api: api, user: user }];
            }
        });
    });
}
exports.init = init;
function declare(api, user, contractAddress, tokenClassHash) {
    return __awaiter(this, void 0, void 0, function () {
        var tx_declare, extrisinc_declare, signedTxDeclare, resultDeclare, error_1;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    tx_declare = {
                        version: 1,
                        hash: "",
                        signature: [],
                        events: [],
                        sender_address: contractAddress,
                        nonce: 0,
                        callEntrypoint: {
                            // call entrypoint
                            classHash: tokenClassHash,
                            entrypointSelector: null,
                            calldata: [],
                            storageAddress: contractAddress,
                            callerAddress: contractAddress
                        },
                        contractClass: {
                            program: (0, util_1.u8aToHex)(new TextEncoder().encode(JSON.stringify(erc20_json_1["default"].program))),
                            entryPointsByType: (0, util_1.u8aToHex)(new TextEncoder().encode(JSON.stringify(erc20_json_1["default"].entry_points_by_type)))
                        }
                    };
                    extrisinc_declare = api.tx.starknet.addDeclareTransaction(tx_declare);
                    _a.label = 1;
                case 1:
                    _a.trys.push([1, 4, , 5]);
                    return [4 /*yield*/, extrisinc_declare.signAsync(user, {
                            nonce: -1
                        })];
                case 2:
                    signedTxDeclare = _a.sent();
                    return [4 /*yield*/, signedTxDeclare.send()];
                case 3:
                    resultDeclare = _a.sent();
                    console.log(resultDeclare.toHuman());
                    return [3 /*break*/, 5];
                case 4:
                    error_1 = _a.sent();
                    console.error("Eror while declaring : ", error_1);
                    return [3 /*break*/, 5];
                case 5: return [2 /*return*/];
            }
        });
    });
}
exports.declare = declare;
function deploy(api, user, contractAddress, tokenClassHash) {
    var _a;
    return __awaiter(this, void 0, void 0, function () {
        var tx_deploy, extrisinc_deploy, signedTxDeploy, resultDeploy, error_2;
        return __generator(this, function (_b) {
            switch (_b.label) {
                case 0:
                    tx_deploy = {
                        version: 1,
                        hash: "",
                        signature: [],
                        events: [],
                        sender_address: contractAddress,
                        nonce: 1,
                        callEntrypoint: {
                            // call entrypoint
                            classHash: tokenClassHash,
                            entrypointSelector: null,
                            calldata: [
                                "0x0000000000000000000000000000000000000000000000000000000000001111",
                                "0x0169f135eddda5ab51886052d777a57f2ea9c162d713691b5e04a6d4ed71d47f",
                                "0x0000000000000000000000000000000000000000000000000000000000000004",
                                tokenClassHash,
                                "0x0000000000000000000000000000000000000000000000000000000000000001",
                                "0x0000000000000000000000000000000000000000000000000000000000000000",
                                "0x0000000000000000000000000000000000000000000000000000000000000001",
                            ],
                            storageAddress: contractAddress,
                            callerAddress: contractAddress
                        },
                        contractClass: null
                    };
                    _b.label = 1;
                case 1:
                    _b.trys.push([1, 4, , 5]);
                    extrisinc_deploy = api.tx.starknet.addInvokeTransaction(tx_deploy);
                    return [4 /*yield*/, extrisinc_deploy.signAsync(user, {
                            nonce: -1
                        })];
                case 2:
                    signedTxDeploy = _b.sent();
                    return [4 /*yield*/, signedTxDeploy.send()];
                case 3:
                    resultDeploy = _b.sent();
                    return [2 /*return*/, (_a = resultDeploy.toHuman()) === null || _a === void 0 ? void 0 : _a.toString()];
                case 4:
                    error_2 = _b.sent();
                    console.error("Eror while deploying : ", error_2);
                    return [2 /*return*/];
                case 5: return [2 /*return*/];
            }
        });
    });
}
exports.deploy = deploy;
function initialize(api, user, contractAddress, tokenAddress) {
    var _a;
    return __awaiter(this, void 0, void 0, function () {
        var tx_initialize, extrisinc_init, signedTxInit, resultInit, error_3;
        return __generator(this, function (_b) {
            switch (_b.label) {
                case 0:
                    tx_initialize = {
                        version: 1,
                        hash: "",
                        signature: [],
                        events: [],
                        sender_address: contractAddress,
                        nonce: 1,
                        callEntrypoint: {
                            // call entrypoint
                            classHash: null,
                            entrypointSelector: null,
                            calldata: [
                                tokenAddress,
                                "0x0079dc0da7c54b95f10aa182ad0a46400db63156920adb65eca2654c0945a463",
                                "0x0000000000000000000000000000000000000000000000000000000000000005",
                                "0x0000000000000000000000000000000000000000000000000000000000000004",
                                "0x0000000000000000000000000000000000000000000000000000000054455354",
                                "0x0000000000000000000000000000000000000000000000000000000054455354",
                                "0x0000000000000000000000000000000000000000000000000000000000000012",
                                contractAddress, // PERMISSIONED MINTER
                            ],
                            storageAddress: contractAddress,
                            callerAddress: contractAddress
                        },
                        contractClass: null
                    };
                    _b.label = 1;
                case 1:
                    _b.trys.push([1, 4, , 5]);
                    extrisinc_init = api.tx.starknet.addInvokeTransaction(tx_initialize);
                    return [4 /*yield*/, extrisinc_init.signAsync(user, {
                            nonce: -1
                        })];
                case 2:
                    signedTxInit = _b.sent();
                    return [4 /*yield*/, signedTxInit.send()];
                case 3:
                    resultInit = _b.sent();
                    return [2 /*return*/, (_a = resultInit.toHuman()) === null || _a === void 0 ? void 0 : _a.toString()];
                case 4:
                    error_3 = _b.sent();
                    console.error("Eror while initializing : ", error_3);
                    return [2 /*return*/];
                case 5: return [2 /*return*/];
            }
        });
    });
}
exports.initialize = initialize;
function mint(api, user, contractAddress, tokenAddress, mintAmount) {
    var _a;
    return __awaiter(this, void 0, void 0, function () {
        var tx_mint, extrisinc_mint, signedTxMint, resultMint, error_4;
        return __generator(this, function (_b) {
            switch (_b.label) {
                case 0:
                    tx_mint = {
                        version: 1,
                        hash: "",
                        signature: [],
                        events: [],
                        sender_address: contractAddress,
                        nonce: 1,
                        callEntrypoint: {
                            // call entrypoint
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
                            callerAddress: contractAddress
                        },
                        contractClass: null
                    };
                    _b.label = 1;
                case 1:
                    _b.trys.push([1, 4, , 5]);
                    extrisinc_mint = api.tx.starknet.addInvokeTransaction(tx_mint);
                    return [4 /*yield*/, extrisinc_mint.signAsync(user, {
                            nonce: -1
                        })];
                case 2:
                    signedTxMint = _b.sent();
                    return [4 /*yield*/, signedTxMint.send()];
                case 3:
                    resultMint = _b.sent();
                    return [2 /*return*/, (_a = resultMint.toHuman()) === null || _a === void 0 ? void 0 : _a.toString()];
                case 4:
                    error_4 = _b.sent();
                    console.error("Eror while initializing : ", error_4);
                    return [2 /*return*/];
                case 5: return [2 /*return*/];
            }
        });
    });
}
exports.mint = mint;
function transfer(api, user, contractAddress, tokenAddress, recipientAddress, transferAmount) {
    var _a;
    return __awaiter(this, void 0, void 0, function () {
        var tx_transfer, extrisinc_transfer, signedTxTransfer, resultTransfer, error_5;
        return __generator(this, function (_b) {
            switch (_b.label) {
                case 0:
                    tx_transfer = {
                        version: 1,
                        hash: "",
                        signature: [],
                        events: [],
                        sender_address: contractAddress,
                        nonce: 3,
                        callEntrypoint: {
                            // call entrypoint
                            classHash: null,
                            entrypointSelector: null,
                            calldata: [
                                tokenAddress,
                                "0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e",
                                "0x0000000000000000000000000000000000000000000000000000000000000003",
                                recipientAddress,
                                transferAmount,
                                "0x0000000000000000000000000000000000000000000000000000000000000000",
                            ],
                            storageAddress: contractAddress,
                            callerAddress: contractAddress
                        },
                        contractClass: null
                    };
                    _b.label = 1;
                case 1:
                    _b.trys.push([1, 4, , 5]);
                    extrisinc_transfer = api.tx.starknet.addInvokeTransaction(tx_transfer);
                    return [4 /*yield*/, extrisinc_transfer.signAsync(user, {
                            nonce: -1
                        })];
                case 2:
                    signedTxTransfer = _b.sent();
                    return [4 /*yield*/, signedTxTransfer.send()];
                case 3:
                    resultTransfer = _b.sent();
                    return [2 /*return*/, (_a = resultTransfer.toHuman()) === null || _a === void 0 ? void 0 : _a.toString()];
                case 4:
                    error_5 = _b.sent();
                    console.error("Error while transfer : ", error_5);
                    return [2 /*return*/];
                case 5: return [2 /*return*/];
            }
        });
    });
}
exports.transfer = transfer;
function e2e_erc20_test() {
    return __awaiter(this, void 0, void 0, function () {
        var _a, api, user, contractAddress, tokenClassHash;
        return __generator(this, function (_b) {
            switch (_b.label) {
                case 0: return [4 /*yield*/, init()];
                case 1:
                    _a = _b.sent(), api = _a.api, user = _a.user;
                    contractAddress = "0x0000000000000000000000000000000000000000000000000000000000000101";
                    tokenClassHash = "0x025ec026985a3bf9d0cc1fe17326b245bfdc3ff89b8fde106242a3ea56c5a918";
                    return [4 /*yield*/, declare(api, user, contractAddress, tokenClassHash)];
                case 2:
                    _b.sent();
                    return [4 /*yield*/, deploy(api, user, contractAddress, tokenClassHash)];
                case 3:
                    _b.sent();
                    return [4 /*yield*/, initialize(api, user, contractAddress, "0x040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d00")];
                case 4:
                    _b.sent();
                    return [2 /*return*/];
            }
        });
    });
}
function erc20_init_test() {
    return __awaiter(this, void 0, void 0, function () {
        var _a, api, user, contractAddress, tokenAddress, mintAmount;
        return __generator(this, function (_b) {
            switch (_b.label) {
                case 0: return [4 /*yield*/, init()];
                case 1:
                    _a = _b.sent(), api = _a.api, user = _a.user;
                    contractAddress = "0x0000000000000000000000000000000000000000000000000000000000000101";
                    tokenAddress = "0x040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d00";
                    mintAmount = "0x0000000000000000000000000000000000000000000000000000000000000001";
                    return [4 /*yield*/, initialize(api, user, contractAddress, tokenAddress)];
                case 2:
                    _b.sent();
                    // await mint(api, user, contractAddress, tokenAddress, mintAmount);
                    return [4 /*yield*/, transfer(api, user, contractAddress, tokenAddress, contractAddress, mintAmount)];
                case 3:
                    // await mint(api, user, contractAddress, tokenAddress, mintAmount);
                    _b.sent();
                    return [2 /*return*/];
            }
        });
    });
}
void erc20_init_test();
