"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.describeDevMadara = void 0;
const tslib_1 = require("tslib");
const api_1 = require("@polkadot/api");
const block_1 = require("./block");
const constants_1 = require("./constants");
const dev_node_1 = require("./dev-node");
const providers_1 = require("./providers");
const substrate_rpc_1 = require("./substrate-rpc");
const debug_1 = tslib_1.__importDefault(require("debug"));
const chai_as_promised_1 = tslib_1.__importDefault(require("chai-as-promised"));
const chai_1 = tslib_1.__importDefault(require("chai"));
const deep_equal_in_any_order_1 = tslib_1.__importDefault(require("deep-equal-in-any-order"));
const process_1 = tslib_1.__importDefault(require("process"));
const debug = (0, debug_1.default)("test:setup");
function describeDevMadara(title, cb, options = {
    runNewNode: false,
    forkedMode: false,
}, runtime = "madara") {
    describe(title, function () {
        this.timeout(50000);
        chai_1.default.use(deep_equal_in_any_order_1.default);
        chai_1.default.use(chai_as_promised_1.default);
        const context = {};
        let madaraProcess;
        before("Starting Madara Test Node", async function () {
            this.timeout(constants_1.SPAWNING_TIME);
            const init = await getRunningNode(runtime, options);
            madaraProcess = init.runningNode;
            context.rpcPort = init.rpcPort;
            context._polkadotApis = [];
            madaraProcess = init.runningNode;
            context.createPolkadotApi = async () => {
                const apiPromise = await (0, providers_1.providePolkadotApi)(init.rpcPort);
                context._polkadotApis.push(apiPromise);
                await apiPromise.isReady;
                await new Promise((resolve) => {
                    setTimeout(resolve, 1000);
                });
                return apiPromise;
            };
            context.polkadotApi = await context.createPolkadotApi();
            const keyringSr25519 = new api_1.Keyring({ type: "sr25519" });
            context.alice = keyringSr25519.addFromUri("//Alice");
            context.createBlock = async (transactions, options = {}) => {
                const results = [];
                const txs = transactions == undefined
                    ? []
                    : Array.isArray(transactions)
                        ? transactions
                        : [transactions];
                for await (const call of txs) {
                    if (call.transaction_hash) {
                        results.push({
                            type: "starknet",
                            hash: call.transaction_hash,
                        });
                    }
                    else if (call.isSigned) {
                        const tx = context.polkadotApi.tx(call);
                        debug(`- Signed: ${tx.method.section}.${tx.method.method}(${tx.args
                            .map((d) => d.toHuman())
                            .join("; ")}) [ nonce: ${tx.nonce}]`);
                        results.push({
                            type: "sub",
                            hash: (await call.send()).toString(),
                        });
                    }
                    else {
                        const tx = context.polkadotApi.tx(call);
                        debug(`- Unsigned: ${tx.method.section}.${tx.method.method}(${tx.args
                            .map((d) => d.toHuman())
                            .join("; ")}) [ nonce: ${tx.nonce}]`);
                        results.push({
                            type: "sub",
                            hash: (await call.send()).toString(),
                        });
                    }
                }
                const { parentHash, finalize } = options;
                const blockResult = await (0, block_1.createAndFinalizeBlock)(context.polkadotApi, parentHash, finalize);
                if (results.length == 0) {
                    return {
                        block: blockResult,
                        result: null,
                    };
                }
                const allRecords = (await (await context.polkadotApi.at(blockResult.hash)).query.system
                    .events());
                const blockData = await context.polkadotApi.rpc.chain.getBlock(blockResult.hash);
                const result = results.map((result) => {
                    const extrinsicIndex = result.type == "starknet"
                        ? allRecords
                            .find(({ phase, event: { section, method, data } }) => phase.isApplyExtrinsic &&
                            section == "starknet" &&
                            method == "Executed" &&
                            data[2].toString() == result.hash)
                            ?.phase?.asApplyExtrinsic?.toNumber()
                        : blockData.block.extrinsics.findIndex((ext) => ext.hash.toHex() == result.hash);
                    const events = allRecords.filter(({ phase }) => phase.isApplyExtrinsic &&
                        phase.asApplyExtrinsic.toNumber() === extrinsicIndex);
                    const failure = (0, substrate_rpc_1.extractError)(events);
                    return {
                        extrinsic: extrinsicIndex >= 0
                            ? blockData.block.extrinsics[extrinsicIndex]
                            : null,
                        events,
                        error: failure &&
                            ((failure.isModule &&
                                context.polkadotApi.registry.findMetaError(failure.asModule)) ||
                                { name: failure.toString() }),
                        successful: extrinsicIndex !== undefined && !failure,
                        hash: result.hash,
                    };
                });
                if (results.find((r) => r.type == "starknet")) {
                    await new Promise((resolve) => setTimeout(resolve, 2));
                }
                return {
                    block: blockResult,
                    result: Array.isArray(transactions) ? result : result[0],
                };
            };
            debug(`Setup ready`);
        });
        after(async function () {
            await Promise.all(context._polkadotApis.map(async (p) => {
                await p.disconnect();
            }));
            if (madaraProcess) {
                await new Promise((resolve) => {
                    madaraProcess.once("exit", resolve);
                    madaraProcess.kill();
                    madaraProcess = null;
                });
            }
        });
        cb(context);
    });
}
exports.describeDevMadara = describeDevMadara;
const getRunningNode = async (runtime, options) => {
    if (options.forkedMode) {
        return await (0, dev_node_1.startMadaraForkedNode)(9933);
    }
    if (!constants_1.DEBUG_MODE) {
        if (!options.runNewNode) {
            const p2pPort = parseInt(process_1.default.env.P2P_PORT);
            const rpcPort = parseInt(process_1.default.env.RPC_PORT);
            return {
                runningNode: null,
                p2pPort,
                rpcPort,
            };
        }
        return await (0, dev_node_1.startMadaraDevNode)(options.withWasm, runtime);
    }
    return {
        runningNode: null,
        p2pPort: 19931,
        rpcPort: 9933,
    };
};
//# sourceMappingURL=setup-dev-tests.js.map