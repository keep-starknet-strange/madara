"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const tslib_1 = require("tslib");
const dev_node_1 = require("../util/dev-node");
const process_1 = tslib_1.__importDefault(require("process"));
let madaraProcess;
exports.mochaGlobalSetup = async function () {
    const { p2pPort, rpcPort, runningNode } = await (0, dev_node_1.startMadaraDevNode)();
    madaraProcess = runningNode;
    process_1.default.env.P2P_PORT = `${p2pPort}`;
    process_1.default.env.RPC_PORT = `${rpcPort}`;
};
exports.mochaGlobalTeardown = async function () {
    await new Promise((resolve) => {
        madaraProcess.once("exit", resolve);
        madaraProcess.kill();
        madaraProcess = null;
    });
};
//# sourceMappingURL=setup-tests.js.map