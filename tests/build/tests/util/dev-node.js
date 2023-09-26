"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.startMadaraForkedNode = exports.startMadaraDevNode = exports.findAvailablePorts = void 0;
const tslib_1 = require("tslib");
const child_process_1 = require("child_process");
const tcp_port_used_1 = tslib_1.__importDefault(require("tcp-port-used"));
const constants_1 = require("./constants");
const debug_1 = tslib_1.__importDefault(require("debug"));
const debug = (0, debug_1.default)("test:dev-node");
async function findAvailablePorts() {
    const availablePorts = await Promise.all([null, null, null].map(async (_, index) => {
        let selectedPort = 0;
        let port = 1024 + index * 20000 + (process.pid % 20000);
        const endingPort = 65535;
        while (!selectedPort && port < endingPort) {
            const inUse = await tcp_port_used_1.default.check(port, "127.0.0.1");
            if (!inUse) {
                selectedPort = port;
            }
            port++;
        }
        if (!selectedPort) {
            throw new Error("No available port");
        }
        return selectedPort;
    }));
    return {
        p2pPort: availablePorts[0],
        rpcPort: availablePorts[1],
    };
}
exports.findAvailablePorts = findAvailablePorts;
let nodeStarted = false;
async function startMadaraDevNode(withWasm, runtime = "madara") {
    while (nodeStarted) {
        await new Promise((resolve) => {
            setTimeout(resolve, 100);
        });
    }
    nodeStarted = true;
    const { p2pPort, rpcPort } = await findAvailablePorts();
    if (process.env.FORCE_WASM_EXECUTION == "true") {
        withWasm = true;
    }
    const cmd = constants_1.BINARY_PATH;
    const args = [
        withWasm ? "--execution=Wasm" : "--execution=Native",
        process.env.FORCE_COMPILED_WASM
            ? "--wasm-execution=compiled"
            : "--wasm-execution=interpreted-i-know-what-i-do",
        "--no-telemetry",
        "--reserved-only",
        "--no-grandpa",
        "--no-prometheus",
        "--dev",
        "--rpc-cors=all",
        "--rpc-methods=unsafe",
        "--tx-ban-seconds=0",
        "--sealing=manual",
        `-l${constants_1.MADARA_LOG}`,
        `--port=${p2pPort}`,
        `--rpc-port=${rpcPort}`,
        `--madara-path=/tmp/${p2pPort}`,
    ];
    if (constants_1.WASM_RUNTIME_OVERRIDES != "") {
        args.push(`--wasm-runtime-overrides=${constants_1.WASM_RUNTIME_OVERRIDES}`);
        args.push("--blocks-pruning=archive");
    }
    debug(`Starting dev node: --port=${p2pPort} --rpc-port=${rpcPort}`);
    const onProcessExit = function () {
        runningNode && runningNode.kill();
    };
    const onProcessInterrupt = function () {
        process.exit(2);
    };
    let runningNode = null;
    process.once("exit", onProcessExit);
    process.once("SIGINT", onProcessInterrupt);
    runningNode = (0, child_process_1.spawn)(cmd, args);
    runningNode.once("exit", () => {
        process.removeListener("exit", onProcessExit);
        process.removeListener("SIGINT", onProcessInterrupt);
        nodeStarted = false;
        debug(`Exiting dev node: --port=${p2pPort} --rpc-port=${rpcPort}`);
    });
    runningNode.on("error", (err) => {
        if (err.errno == "ENOENT") {
            console.error("\x1b[31mMissing Madara binary " +
                `(${constants_1.BINARY_PATH}).\nPlease compile the Madara project\x1b[0m`);
        }
        else {
            console.error(err);
        }
        process.exit(1);
    });
    const binaryLogs = [];
    await new Promise((resolve) => {
        const timer = setTimeout(() => {
            console.error("\x1b[31m Failed to start Madara Test Node.\x1b[0m");
            console.error(`Command: ${cmd} ${args.join(" ")}`);
            console.error("Logs:");
            console.error(binaryLogs.map((chunk) => chunk.toString()).join("\n"));
            throw new Error("Failed to launch node");
        }, constants_1.SPAWNING_TIME - 2000);
        const onData = async (chunk) => {
            if (constants_1.DISPLAY_LOG) {
                console.log(chunk.toString());
            }
            binaryLogs.push(chunk);
            if (chunk.toString().match(/Madara Node/)) {
                clearTimeout(timer);
                if (!constants_1.DISPLAY_LOG) {
                    runningNode.stderr.off("data", onData);
                    runningNode.stdout.off("data", onData);
                }
                resolve();
            }
        };
        runningNode.stderr.on("data", onData);
        runningNode.stdout.on("data", onData);
    });
    return { p2pPort, rpcPort, runningNode };
}
exports.startMadaraDevNode = startMadaraDevNode;
async function startMadaraForkedNode(rpcPort) {
    while (nodeStarted) {
        await new Promise((resolve) => {
            setTimeout(resolve, 100);
        });
    }
    nodeStarted = true;
    const cmd = constants_1.BINARY_PATH;
    const args = [
        "--execution=Native",
        "--no-hardware-benchmarks",
        "--no-telemetry",
        "--database=paritydb",
        "--no-prometheus",
        "--alice",
        `--chain=${constants_1.CUSTOM_SPEC_PATH}`,
        "--sealing=manual",
        `-l${constants_1.MADARA_LOG}`,
        `--rpc-port=${rpcPort}`,
        "--trie-cache-size=0",
        "--db-cache=5000",
        "--collator",
        `--base-path=${constants_1.BASE_PATH}`,
    ];
    debug(`Starting dev node: --rpc-port=${rpcPort}`);
    const onProcessExit = function () {
        runningNode && runningNode.kill();
    };
    const onProcessInterrupt = function () {
        process.exit(2);
    };
    let runningNode = null;
    process.once("exit", onProcessExit);
    process.once("SIGINT", onProcessInterrupt);
    runningNode = (0, child_process_1.spawn)(cmd, args);
    runningNode.once("exit", () => {
        process.removeListener("exit", onProcessExit);
        process.removeListener("SIGINT", onProcessInterrupt);
        nodeStarted = false;
        debug(`Exiting dev node: --rpc-port=${rpcPort}`);
    });
    runningNode.on("error", (err) => {
        if (err.errno == "ENOENT") {
            console.error("\x1b[31mMissing Madara binary " +
                `(${constants_1.BINARY_PATH}).\nPlease compile the Madara project\x1b[0m`);
        }
        else {
            console.error(err);
        }
        process.exit(1);
    });
    const binaryLogs = [];
    await new Promise((resolve) => {
        const timer = setTimeout(() => {
            console.error("\x1b[31m Failed to start Madara Test Node.\x1b[0m");
            console.error(`Command: ${cmd} ${args.join(" ")}`);
            console.error("Logs:");
            console.error(binaryLogs.map((chunk) => chunk.toString()).join("\n"));
            throw new Error("Failed to launch node");
        }, constants_1.SPAWNING_TIME - 2000);
        const onData = async (chunk) => {
            if (constants_1.DISPLAY_LOG) {
                console.log(chunk.toString());
            }
            binaryLogs.push(chunk);
            if (chunk.toString().match(/Madara Node/)) {
                clearTimeout(timer);
                if (!constants_1.DISPLAY_LOG) {
                    runningNode.stderr.off("data", onData);
                    runningNode.stdout.off("data", onData);
                }
                resolve();
            }
        };
        runningNode.stderr.on("data", onData);
        runningNode.stdout.on("data", onData);
    });
    return { rpcPort, runningNode };
}
exports.startMadaraForkedNode = startMadaraForkedNode;
//# sourceMappingURL=dev-node.js.map