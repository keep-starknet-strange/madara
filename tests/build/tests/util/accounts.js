"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.alice = void 0;
const tslib_1 = require("tslib");
const keyring_1 = tslib_1.__importDefault(require("@polkadot/keyring"));
const keyringSr25519 = new keyring_1.default({ type: "sr25519" });
exports.alice = keyringSr25519.addFromUri("//Alice");
//# sourceMappingURL=accounts.js.map