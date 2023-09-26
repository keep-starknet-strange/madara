"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.providePolkadotApi = void 0;
const api_1 = require("@polkadot/api");
const providePolkadotApi = async (port) => {
    return await api_1.ApiPromise.create({
        initWasm: false,
        provider: new api_1.WsProvider(`ws://localhost:${port}`),
        noInitWarn: true,
    });
};
exports.providePolkadotApi = providePolkadotApi;
//# sourceMappingURL=providers.js.map