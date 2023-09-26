"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.expectSubstrateEvents = exports.expectSubstrateEvent = exports.expectOk = void 0;
const chai_1 = require("chai");
async function expectOk(call) {
    const block = await call;
    if (Array.isArray(block.result)) {
        block.result.forEach((r, idx) => {
            (0, chai_1.expect)(r.successful, `tx[${idx}] - ${r.error?.name}${r.extrinsic
                ? `\n\t\t${r.extrinsic.method.section}.${r.extrinsic.method.method}(${r.extrinsic.args.map((d) => d.toHuman()).join("; ")})`
                : ""}`).to.be.true;
        });
    }
    else {
        (0, chai_1.expect)(block.result.successful, block.result.error?.name).to.be.true;
    }
    return block;
}
exports.expectOk = expectOk;
function expectSubstrateEvent(block, section, method) {
    let event = null;
    if (Array.isArray(block.result)) {
        block.result.forEach((r) => {
            const foundEvents = r.events.filter(({ event }) => event.section.toString() == section &&
                event.method.toString() == method);
            if (foundEvents.length > 0) {
                (0, chai_1.expect)(event, `Event ${section.toString()}.${method.toString()} appeared multiple times`).to.be.null;
                (0, chai_1.expect)(foundEvents, `Event ${section.toString()}.${method.toString()} appeared multiple times`).to.be.length(1);
                event = foundEvents[0];
            }
        });
    }
    else {
        const foundEvents = block.result.events.filter(({ event }) => event.section.toString() == section &&
            event.method.toString() == method);
        if (foundEvents.length > 0) {
            (0, chai_1.expect)(foundEvents, `Event ${section.toString()}.${method.toString()} appeared multiple times`).to.be.length(1);
            event = foundEvents[0];
        }
    }
    (0, chai_1.expect)(event).to.not.be.null;
    return event.event;
}
exports.expectSubstrateEvent = expectSubstrateEvent;
function expectSubstrateEvents(block, section, method, count = 0) {
    const events = [];
    if (Array.isArray(block.result)) {
        block.result.forEach((r) => {
            const foundEvents = r.events.filter(({ event }) => event.section.toString() == section &&
                event.method.toString() == method);
            if (foundEvents.length > 0) {
                events.push(...foundEvents);
            }
        });
    }
    else {
        const foundEvents = block.result.events.filter(({ event }) => event.section.toString() == section &&
            event.method.toString() == method);
        if (foundEvents.length > 0) {
            events.push(...foundEvents);
        }
    }
    (0, chai_1.expect)(events.length > 0).to.not.be.null;
    (0, chai_1.expect)(count === 0 || events.length === count).to.be.true;
    return events.map(({ event }) => event);
}
exports.expectSubstrateEvents = expectSubstrateEvents;
//# sourceMappingURL=expect.js.map