"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.extractPreimageDeposit = exports.getBlockArray = exports.fetchHistoricBlockNum = exports.getBlockTime = exports.jumpBlocks = exports.createAndFinalizeBlock = void 0;
const tslib_1 = require("tslib");
require("@keep-starknet-strange/madara-api-augment/sharingan");
const bottleneck_1 = tslib_1.__importDefault(require("bottleneck"));
const debug_1 = tslib_1.__importDefault(require("debug"));
const debug = (0, debug_1.default)("test:blocks");
async function createAndFinalizeBlock(api, parentHash, finalize = true) {
    const startTime = Date.now();
    const block = parentHash
        ? await api.rpc.engine.createBlock(true, finalize, parentHash)
        : await api.rpc.engine.createBlock(true, finalize);
    return {
        duration: Date.now() - startTime,
        hash: block.toJSON().hash,
    };
}
exports.createAndFinalizeBlock = createAndFinalizeBlock;
async function jumpBlocks(context, blockCount) {
    while (blockCount > 0) {
        (await context.createBlock()).block.hash.toString();
        blockCount--;
    }
}
exports.jumpBlocks = jumpBlocks;
const getBlockTime = (signedBlock) => signedBlock.block.extrinsics
    .find((item) => item.method.section == "timestamp")
    .method.args[0].toNumber();
exports.getBlockTime = getBlockTime;
const fetchBlockTime = async (api, blockNum) => {
    const hash = await api.rpc.chain.getBlockHash(blockNum);
    const block = await api.rpc.chain.getBlock(hash);
    return (0, exports.getBlockTime)(block);
};
const fetchHistoricBlockNum = async (api, blockNumber, targetTime) => {
    if (blockNumber <= 1) {
        return 1;
    }
    const time = await fetchBlockTime(api, blockNumber);
    if (time <= targetTime) {
        return blockNumber;
    }
    return (0, exports.fetchHistoricBlockNum)(api, blockNumber - Math.ceil((time - targetTime) / 30000), targetTime);
};
exports.fetchHistoricBlockNum = fetchHistoricBlockNum;
const getBlockArray = async (api, timePeriod, limiter) => {
    if (limiter == null) {
        limiter = new bottleneck_1.default({ maxConcurrent: 10, minTime: 100 });
    }
    const finalizedHead = await limiter.schedule(async () => await api.rpc.chain.getFinalizedHead());
    const signedBlock = await limiter.schedule(async () => await api.rpc.chain.getBlock(finalizedHead));
    const lastBlockNumber = signedBlock.block.header.number.toNumber();
    const lastBlockTime = (0, exports.getBlockTime)(signedBlock);
    const firstBlockTime = lastBlockTime - timePeriod;
    debug(`Searching for the block at: ${new Date(firstBlockTime)}`);
    const firstBlockNumber = (await limiter.wrap(exports.fetchHistoricBlockNum)(api, lastBlockNumber, firstBlockTime));
    const length = lastBlockNumber - firstBlockNumber;
    return Array.from({ length }, (_, i) => firstBlockNumber + i);
};
exports.getBlockArray = getBlockArray;
function extractPreimageDeposit(request) {
    const deposit = "deposit" in request ? request.deposit : request;
    if ("isSome" in deposit) {
        return {
            accountId: deposit.unwrap()[0].toHex(),
            amount: deposit.unwrap()[1],
        };
    }
    return {
        accountId: deposit[0].toHex(),
        amount: deposit[1],
    };
}
exports.extractPreimageDeposit = extractPreimageDeposit;
//# sourceMappingURL=block.js.map