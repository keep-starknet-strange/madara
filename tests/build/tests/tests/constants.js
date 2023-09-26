"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ERC20_CAIRO_1_CASM = exports.TEST_CAIRO_1_CASM = exports.ERC20_CAIRO_1_SIERRA = exports.TEST_CAIRO_1_SIERRA = exports.TEST_CONTRACT = exports.ERC721_CONTRACT = exports.ERC20_CONTRACT = exports.UDC_CLASS_HASH = exports.UDC_CONTRACT_ADDRESS = exports.NFT_CLASS_HASH = exports.NFT_CONTRACT_ADDRESS = exports.CHAIN_ID_STARKNET_TESTNET = exports.SEQUENCER_ADDRESS = exports.ARGENT_CONTRACT_ADDRESS = exports.TOKEN_CLASS_HASH = exports.FEE_TOKEN_ADDRESS = exports.CONTRACT_ADDRESS = exports.DEPLOY_ACCOUNT_COST = exports.MINT_AMOUNT = exports.TEST_CONTRACT_CLASS_HASH = exports.SALT = exports.SIGNER_PRIVATE = exports.SIGNER_PUBLIC = exports.ARGENT_ACCOUNT_CLASS_HASH = exports.ARGENT_PROXY_CLASS_HASH = exports.ACCOUNT_CONTRACT_CLASS_HASH = exports.CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH = exports.CAIRO_1_ACCOUNT_CONTRACT = exports.ACCOUNT_CONTRACT = exports.TEST_CONTRACT_ADDRESS = void 0;
const tslib_1 = require("tslib");
const fs_1 = tslib_1.__importDefault(require("fs"));
const starknet_1 = require("starknet");
exports.TEST_CONTRACT_ADDRESS = "0x0000000000000000000000000000000000000000000000000000000000001111";
exports.ACCOUNT_CONTRACT = "0x0000000000000000000000000000000000000000000000000000000000000001";
exports.CAIRO_1_ACCOUNT_CONTRACT = "0x0000000000000000000000000000000000000000000000000000000000000004";
exports.CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH = "0x35ccefcf9d5656da623468e27e682271cd327af196785df99e7fee1436b6276";
exports.ACCOUNT_CONTRACT_CLASS_HASH = "0x0279d77db761fba82e0054125a6fdb5f6baa6286fa3fb73450cc44d193c2d37f";
exports.ARGENT_PROXY_CLASS_HASH = "0x0424b7f61e3c5dfd74400d96fdea7e1f0bf2757f31df04387eaa957f095dd7b9";
exports.ARGENT_ACCOUNT_CLASS_HASH = "0x06f0d6f6ae72e1a507ff4b65181291642889742dbf8f1a53e9ec1c595d01ba7d";
exports.SIGNER_PUBLIC = "0x03603a2692a2ae60abb343e832ee53b55d6b25f02a3ef1565ec691edc7a209b2";
exports.SIGNER_PRIVATE = "0x00c1cf1490de1352865301bb8705143f3ef938f97fdf892f1090dcb5ac7bcd1d";
exports.SALT = "0x0000000000000000000000000000000000000000000000000000000000001111";
exports.TEST_CONTRACT_CLASS_HASH = "0x0000000000000000000000000000000000000000000000000000000000001000";
exports.MINT_AMOUNT = "0x0000000000000000000000000000000000000000000000000000000000000001";
exports.DEPLOY_ACCOUNT_COST = "0x00000000000000000000000000000000000000000000000000000000ffffffff";
exports.CONTRACT_ADDRESS = "0x0000000000000000000000000000000000000000000000000000000000000001";
exports.FEE_TOKEN_ADDRESS = "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7";
exports.TOKEN_CLASS_HASH = "0x0000000000000000000000000000000000000000000000000000000000010000";
exports.ARGENT_CONTRACT_ADDRESS = "0x0000000000000000000000000000000000000000000000000000000000000002";
exports.SEQUENCER_ADDRESS = "0x000000000000000000000000000000000000000000000000000000000000dead";
exports.CHAIN_ID_STARKNET_TESTNET = "0x534e5f474f45524c49";
exports.NFT_CONTRACT_ADDRESS = "0x040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d02";
exports.NFT_CLASS_HASH = "0x90000";
exports.UDC_CONTRACT_ADDRESS = "0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf";
exports.UDC_CLASS_HASH = "0x90000";
exports.ERC20_CONTRACT = starknet_1.json.parse(fs_1.default.readFileSync("../cairo-contracts/build/ERC20.json").toString("ascii"));
exports.ERC721_CONTRACT = starknet_1.json.parse(fs_1.default.readFileSync("../cairo-contracts/build/ERC721.json").toString("ascii"));
exports.TEST_CONTRACT = starknet_1.json.parse(fs_1.default.readFileSync("../cairo-contracts/build/test.json").toString("ascii"));
exports.TEST_CAIRO_1_SIERRA = starknet_1.json.parse(fs_1.default
    .readFileSync("../cairo-contracts/build/cairo_1/HelloStarknet.sierra.json")
    .toString("ascii"));
exports.ERC20_CAIRO_1_SIERRA = starknet_1.json.parse(fs_1.default
    .readFileSync("../cairo-contracts/build/cairo_1/erc20.sierra.json")
    .toString("ascii"));
exports.TEST_CAIRO_1_CASM = starknet_1.json.parse(fs_1.default
    .readFileSync("../cairo-contracts/build/cairo_1/HelloStarknet.casm.json")
    .toString("ascii"));
exports.ERC20_CAIRO_1_CASM = starknet_1.json.parse(fs_1.default
    .readFileSync("../cairo-contracts/build/cairo_1/erc20.casm.json")
    .toString("ascii"));
//# sourceMappingURL=constants.js.map