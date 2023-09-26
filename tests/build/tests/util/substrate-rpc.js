"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.extractInfo = exports.isExtrinsicSuccessful = exports.extractError = exports.getDispatchError = exports.filterAndApply = exports.createBlockWithExtrinsicParachain = exports.logEvents = exports.waitOneBlock = void 0;
const tslib_1 = require("tslib");
require("@keep-starknet-strange/madara-api-augment");
const util_1 = require("@polkadot/util");
const debug_1 = tslib_1.__importDefault(require("debug"));
const debug = (0, debug_1.default)("test:substrateEvents");
async function waitOneBlock(api, numberOfBlocks = 1) {
    await new Promise(async (res) => {
        let count = 0;
        const unsub = await api.derive.chain.subscribeNewHeads(async (header) => {
            console.log(`One block elapsed : #${header.number}: author : ${header.author}`);
            count += 1;
            if (count === 1 + numberOfBlocks) {
                unsub();
                res();
            }
        });
    });
}
exports.waitOneBlock = waitOneBlock;
async function logEvents(api, name) {
    api.derive.chain.subscribeNewHeads(async (header) => {
        debug(`------------- ${name} BLOCK#${header.number}: author ${header.author}, hash ${header.hash}`);
        const allRecords = (await (await api.at(header.hash)).query.system
            .events());
        allRecords.forEach((e, i) => {
            debug(`${name} Event :`, i, header.hash.toHex(), e.toHuman().event.section, e.toHuman().event.method);
        });
    });
}
exports.logEvents = logEvents;
async function lookForExtrinsicAndEvents(api, extrinsicHash) {
    const signedBlock = await api.rpc.chain.getBlock();
    const allRecords = (await (await api.at(signedBlock.block.header.hash)).query.system
        .events());
    const extrinsicIndex = signedBlock.block.extrinsics.findIndex((ext) => {
        return ext.hash.toHex() === (0, util_1.u8aToHex)(extrinsicHash);
    });
    if (extrinsicIndex < 0) {
        console.log(`Extrinsic ${extrinsicHash} is missing in the block ${signedBlock.block.header.hash}`);
    }
    const extrinsic = signedBlock.block.extrinsics[extrinsicIndex];
    const events = allRecords
        .filter(({ phase }) => phase.isApplyExtrinsic &&
        phase.asApplyExtrinsic.toNumber() === extrinsicIndex)
        .map(({ event }) => event);
    return { events, extrinsic };
}
async function tryLookingForEvents(api, extrinsicHash) {
    await waitOneBlock(api);
    const { extrinsic, events } = await lookForExtrinsicAndEvents(api, extrinsicHash);
    if (events.length > 0) {
        return {
            extrinsic,
            events,
        };
    }
    else {
        return await tryLookingForEvents(api, extrinsicHash);
    }
}
const createBlockWithExtrinsicParachain = async (api, sender, polkadotCall) => {
    console.log("-------------- EXTRINSIC CALL -------------------------------");
    const extrinsicHash = (await polkadotCall.signAndSend(sender));
    return await tryLookingForEvents(api, extrinsicHash);
};
exports.createBlockWithExtrinsicParachain = createBlockWithExtrinsicParachain;
function filterAndApply(events, section, methods, onFound) {
    return events
        .filter(({ event }) => section === event.section && methods.includes(event.method))
        .map((record) => onFound(record));
}
exports.filterAndApply = filterAndApply;
function getDispatchError({ event: { data: [dispatchError], }, }) {
    return dispatchError;
}
exports.getDispatchError = getDispatchError;
function getDispatchInfo({ event: { data, method }, }) {
    return method === "ExtrinsicSuccess"
        ? data[0]
        : data[1];
}
function extractError(events = []) {
    return filterAndApply(events, "system", ["ExtrinsicFailed"], getDispatchError)[0];
}
exports.extractError = extractError;
function isExtrinsicSuccessful(events = []) {
    return (filterAndApply(events, "system", ["ExtrinsicSuccess"], () => true).length >
        0);
}
exports.isExtrinsicSuccessful = isExtrinsicSuccessful;
function extractInfo(events = []) {
    return filterAndApply(events, "system", ["ExtrinsicFailed", "ExtrinsicSuccess"], getDispatchInfo)[0];
}
exports.extractInfo = extractInfo;
//# sourceMappingURL=substrate-rpc.js.map