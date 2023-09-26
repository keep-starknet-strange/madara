"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
require("@keep-starknet-strange/madara-api-augment");
const chai_1 = require("chai");
const starknet_1 = require("starknet");
const setup_dev_tests_1 = require("../../util/setup-dev-tests");
const utils_1 = require("../../util/utils");
const constants_1 = require("../constants");
let ARGENT_CONTRACT_NONCE = { value: 0 };
(0, setup_dev_tests_1.describeDevMadara)("Starknet RPC - Events Test", (context) => {
    let providerRPC;
    before(async function () {
        providerRPC = new starknet_1.RpcProvider({
            nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
            retries: 3,
        });
    });
    describe("getEvents", () => {
        it("should fail on invalid continuation token", async function () {
            const filter = {
                from_block: { block_number: 0 },
                to_block: { block_number: 1 },
                address: constants_1.FEE_TOKEN_ADDRESS,
                chunk_size: 1,
                continuation_token: "0xabdel",
                keys: [[]],
            };
            let events = providerRPC.getEvents(filter);
            await (0, chai_1.expect)(events)
                .to.eventually.be.rejectedWith("33: The supplied continuation token is invalid or unknown")
                .and.be.an.instanceOf(starknet_1.LibraryError);
            const transactions = [];
            for (let i = 0; i < 5; i++) {
                transactions.push((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT));
            }
            await context.createBlock(transactions);
            const block = await providerRPC.getBlockHashAndNumber();
            let filter2 = {
                from_block: { block_number: block.block_number },
                to_block: { block_number: block.block_number },
                address: constants_1.FEE_TOKEN_ADDRESS,
                chunk_size: 1,
                continuation_token: "0,100,1",
                keys: [[]],
            };
            events = providerRPC.getEvents(filter2);
            await (0, chai_1.expect)(events)
                .to.eventually.be.rejectedWith("33: The supplied continuation token is invalid or unknown")
                .and.be.an.instanceOf(starknet_1.LibraryError);
            filter2 = {
                from_block: { block_number: block.block_number },
                to_block: { block_number: block.block_number },
                address: constants_1.FEE_TOKEN_ADDRESS,
                chunk_size: 1,
                continuation_token: "0,0,100",
                keys: [[]],
            };
            events = providerRPC.getEvents(filter2);
            await (0, chai_1.expect)(events)
                .to.eventually.be.rejectedWith("33: The supplied continuation token is invalid or unknown")
                .and.be.an.instanceOf(starknet_1.LibraryError);
        });
        it("should fail on chunk size too big", async function () {
            const filter = {
                from_block: { block_number: 0 },
                to_block: { block_number: 1 },
                address: constants_1.FEE_TOKEN_ADDRESS,
                chunk_size: 1001,
                keys: [[]],
            };
            const events = providerRPC.getEvents(filter);
            await (0, chai_1.expect)(events)
                .to.eventually.be.rejectedWith("31: Requested page size is too big")
                .and.be.an.instanceOf(starknet_1.LibraryError);
        });
        it("should fail on keys too big", async function () {
            const filter = {
                from_block: { block_number: 0 },
                to_block: { block_number: 1 },
                address: constants_1.FEE_TOKEN_ADDRESS,
                chunk_size: 1,
                keys: Array(101).fill(["0x0"]),
            };
            const events = providerRPC.getEvents(filter);
            await (0, chai_1.expect)(events)
                .to.eventually.be.rejectedWith("34: Too many keys provided in a filter")
                .and.be.an.instanceOf(starknet_1.LibraryError);
        });
        it("returns expected events on correct filter", async function () {
            await context.createBlock((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT));
            const tx = await providerRPC.getTransactionByBlockIdAndIndex("latest", 0);
            const block_hash_and_number = await providerRPC.getBlockHashAndNumber();
            const filter = {
                from_block: "latest",
                to_block: "latest",
                address: constants_1.FEE_TOKEN_ADDRESS,
                chunk_size: 10,
            };
            const events = await providerRPC.getEvents(filter);
            (0, chai_1.expect)(events.events.length).to.be.equal(2);
            (0, chai_1.expect)(events.continuation_token).to.be.null;
            for (const event of events.events) {
                (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(event.from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
                (0, chai_1.expect)(event.transaction_hash).to.be.equal(tx.transaction_hash);
            }
            const transfer_event = events.events[0];
            (0, chai_1.expect)(transfer_event).to.deep.equal({
                transaction_hash: tx.transaction_hash,
                block_hash: block_hash_and_number.block_hash,
                block_number: block_hash_and_number.block_number,
                from_address: (0, utils_1.cleanHex)(constants_1.FEE_TOKEN_ADDRESS),
                keys: [(0, utils_1.toHex)((0, utils_1.starknetKeccak)("Transfer"))],
                data: [
                    constants_1.ARGENT_CONTRACT_ADDRESS,
                    constants_1.ARGENT_CONTRACT_ADDRESS,
                    constants_1.MINT_AMOUNT,
                    "0x0",
                ].map(utils_1.cleanHex),
            });
            const fee_event = events.events[1];
            (0, chai_1.expect)(fee_event).to.deep.equal({
                transaction_hash: tx.transaction_hash,
                block_hash: block_hash_and_number.block_hash,
                block_number: block_hash_and_number.block_number,
                from_address: (0, utils_1.cleanHex)(constants_1.FEE_TOKEN_ADDRESS),
                keys: [(0, utils_1.toHex)((0, utils_1.starknetKeccak)("Transfer"))],
                data: [
                    constants_1.ARGENT_CONTRACT_ADDRESS,
                    constants_1.SEQUENCER_ADDRESS,
                    "0x1a02c",
                    "0x0",
                ].map(utils_1.cleanHex),
            });
        });
        it("returns expected events on correct filter two blocks", async function () {
            const transactions = [];
            for (let i = 0; i < 5; i++) {
                transactions.push((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT));
            }
            await context.createBlock(transactions);
            const firstBlockCreated = await providerRPC.getBlockHashAndNumber();
            const transactions2 = [];
            for (let i = 0; i < 5; i++) {
                transactions2.push((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT));
            }
            await context.createBlock(transactions2);
            const secondBlockCreated = await providerRPC.getBlockHashAndNumber();
            const filter = {
                from_block: { block_number: firstBlockCreated.block_number },
                to_block: { block_number: secondBlockCreated.block_number },
                address: constants_1.FEE_TOKEN_ADDRESS,
                chunk_size: 100,
            };
            const events = await providerRPC.getEvents(filter);
            (0, chai_1.expect)(events.events.length).to.be.equal(20);
            (0, chai_1.expect)(events.continuation_token).to.be.null;
            for (let i = 0; i < 2; i++) {
                const tx = await providerRPC.getTransactionByBlockIdAndIndex(firstBlockCreated.block_hash, i);
                (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events.events[2 * i].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
                (0, chai_1.expect)(events.events[2 * i].transaction_hash).to.be.equal(tx.transaction_hash);
                (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events.events[2 * i + 1].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
                (0, chai_1.expect)(events.events[2 * i + 1].transaction_hash).to.be.equal(tx.transaction_hash);
            }
            for (let i = 0; i < 2; i++) {
                const tx_second_block = await providerRPC.getTransactionByBlockIdAndIndex(secondBlockCreated.block_hash, i);
                (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events.events[10 + 2 * i].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
                (0, chai_1.expect)(events.events[10 + 2 * i].transaction_hash).to.be.equal(tx_second_block.transaction_hash);
                (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events.events[10 + 2 * i + 1].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
                (0, chai_1.expect)(events.events[10 + 2 * i + 1].transaction_hash).to.be.equal(tx_second_block.transaction_hash);
            }
        });
        it("returns expected events on correct filter two blocks pagination", async function () {
            const transactions = [];
            for (let i = 0; i < 5; i++) {
                transactions.push((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT));
            }
            await context.createBlock(transactions);
            const firstBlockCreated = await providerRPC.getBlockHashAndNumber();
            const transactions2 = [];
            for (let i = 0; i < 5; i++) {
                transactions2.push((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT));
            }
            await context.createBlock(transactions2);
            const secondBlockCreated = await providerRPC.getBlockHashAndNumber();
            let filter = {
                from_block: { block_number: firstBlockCreated.block_number },
                to_block: { block_number: secondBlockCreated.block_number },
                address: constants_1.FEE_TOKEN_ADDRESS,
                chunk_size: 7,
                continuation_token: null,
            };
            let { events, continuation_token } = await providerRPC.getEvents(filter);
            (0, chai_1.expect)(events.length).to.be.equal(7);
            (0, chai_1.expect)(continuation_token).to.be.equal("0,3,1");
            for (let i = 0; i < 3; i++) {
                const tx = await providerRPC.getTransactionByBlockIdAndIndex(firstBlockCreated.block_hash, i);
                (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events[2 * i].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
                (0, chai_1.expect)(events[2 * i].transaction_hash).to.be.equal(tx.transaction_hash);
                (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events[2 * i + 1].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
                (0, chai_1.expect)(events[2 * i + 1].transaction_hash).to.be.equal(tx.transaction_hash);
            }
            const tx3 = await providerRPC.getTransactionByBlockIdAndIndex(firstBlockCreated.block_hash, 3);
            (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events[6].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
            (0, chai_1.expect)(events[6].transaction_hash).to.be.equal(tx3.transaction_hash);
            filter = {
                from_block: { block_number: firstBlockCreated.block_number },
                to_block: { block_number: secondBlockCreated.block_number },
                address: constants_1.FEE_TOKEN_ADDRESS,
                chunk_size: 7,
                continuation_token: continuation_token,
            };
            ({ events, continuation_token } = await providerRPC.getEvents(filter));
            (0, chai_1.expect)(events.length).to.be.equal(7);
            (0, chai_1.expect)(continuation_token).to.be.equal("1,1,3");
            (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events[0].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
            (0, chai_1.expect)(events[0].transaction_hash).to.be.equal(tx3.transaction_hash);
            const tx4 = await providerRPC.getTransactionByBlockIdAndIndex(firstBlockCreated.block_hash, 4);
            (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events[1].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
            (0, chai_1.expect)(events[1].transaction_hash).to.be.equal(tx4.transaction_hash);
            (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events[2].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
            (0, chai_1.expect)(events[2].transaction_hash).to.be.equal(tx4.transaction_hash);
            for (let i = 0; i < 2; i++) {
                const tx = await providerRPC.getTransactionByBlockIdAndIndex(secondBlockCreated.block_hash, i);
                (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events[2 * i + 3].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
                (0, chai_1.expect)(events[2 * i + 3].transaction_hash).to.be.equal(tx.transaction_hash);
                (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events[2 * i + 4].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
                (0, chai_1.expect)(events[2 * i + 4].transaction_hash).to.be.equal(tx.transaction_hash);
            }
            filter = {
                from_block: { block_number: firstBlockCreated.block_number },
                to_block: { block_number: secondBlockCreated.block_number },
                address: constants_1.FEE_TOKEN_ADDRESS,
                chunk_size: 7,
                continuation_token: continuation_token,
            };
            ({ events, continuation_token } = await providerRPC.getEvents(filter));
            (0, chai_1.expect)(events.length).to.be.equal(6);
            (0, chai_1.expect)(continuation_token).to.be.null;
            for (let i = 2; i < 5; i++) {
                const tx = await providerRPC.getTransactionByBlockIdAndIndex(secondBlockCreated.block_hash, i);
                (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events[2 * i - 4].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
                (0, chai_1.expect)(events[2 * i - 4].transaction_hash).to.be.equal(tx.transaction_hash);
                (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events[2 * i - 3].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
                (0, chai_1.expect)(events[2 * i - 3].transaction_hash).to.be.equal(tx.transaction_hash);
            }
        });
        it("returns expected events on correct filter many blocks pagination", async function () {
            const transactions = [];
            for (let i = 0; i < 5; i++) {
                transactions.push((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT));
            }
            await context.createBlock(transactions);
            const firstBlockCreated = await providerRPC.getBlockHashAndNumber();
            const empty_transactions = [];
            await context.createBlock(empty_transactions);
            await context.createBlock(empty_transactions);
            await context.createBlock(empty_transactions);
            const transactions2 = [];
            for (let i = 0; i < 5; i++) {
                transactions2.push((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT));
            }
            await context.createBlock(transactions2);
            const fifthBlockCreated = await providerRPC.getBlockHashAndNumber();
            let filter = {
                from_block: { block_number: firstBlockCreated.block_number },
                to_block: { block_number: fifthBlockCreated.block_number },
                address: constants_1.FEE_TOKEN_ADDRESS,
                chunk_size: 10,
                continuation_token: null,
            };
            let { events, continuation_token } = await providerRPC.getEvents(filter);
            (0, chai_1.expect)(events.length).to.be.equal(10);
            (0, chai_1.expect)(continuation_token).to.be.equal("0,4,3");
            for (let i = 0; i < 5; i++) {
                const tx = await providerRPC.getTransactionByBlockIdAndIndex(firstBlockCreated.block_hash, i);
                (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events[2 * i].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
                (0, chai_1.expect)(events[2 * i].transaction_hash).to.be.equal(tx.transaction_hash);
                (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events[2 * i + 1].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
                (0, chai_1.expect)(events[2 * i + 1].transaction_hash).to.be.equal(tx.transaction_hash);
            }
            filter = {
                from_block: { block_number: firstBlockCreated.block_number },
                to_block: { block_number: fifthBlockCreated.block_number },
                address: constants_1.FEE_TOKEN_ADDRESS,
                chunk_size: 10,
                continuation_token: continuation_token,
            };
            ({ events, continuation_token } = await providerRPC.getEvents(filter));
            (0, chai_1.expect)(events.length).to.be.equal(10);
            (0, chai_1.expect)(continuation_token).to.be.null;
            for (let i = 0; i < 5; i++) {
                const tx = await providerRPC.getTransactionByBlockIdAndIndex(fifthBlockCreated.block_hash, i);
                (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events[2 * i].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
                (0, chai_1.expect)(events[2 * i].transaction_hash).to.be.equal(tx.transaction_hash);
                (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events[2 * i + 1].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
                (0, chai_1.expect)(events[2 * i + 1].transaction_hash).to.be.equal(tx.transaction_hash);
            }
        });
        it("returns expected events on correct filter many empty blocks pagination", async function () {
            const transactions = [];
            for (let i = 0; i < 5; i++) {
                transactions.push((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT));
            }
            await context.createBlock(transactions);
            const firstBlockCreated = await providerRPC.getBlockHashAndNumber();
            const empty_transactions = [];
            await context.createBlock(empty_transactions);
            await context.createBlock(empty_transactions);
            await context.createBlock(empty_transactions);
            await context.createBlock(empty_transactions);
            const fifthBlockCreated = await providerRPC.getBlockHashAndNumber();
            let filter = {
                from_block: { block_number: firstBlockCreated.block_number },
                to_block: { block_number: fifthBlockCreated.block_number },
                address: constants_1.FEE_TOKEN_ADDRESS,
                chunk_size: 10,
                continuation_token: null,
            };
            let { events, continuation_token } = await providerRPC.getEvents(filter);
            (0, chai_1.expect)(events.length).to.be.equal(10);
            (0, chai_1.expect)(continuation_token).to.be.equal("0,4,3");
            filter = {
                from_block: { block_number: firstBlockCreated.block_number },
                to_block: { block_number: fifthBlockCreated.block_number },
                address: constants_1.FEE_TOKEN_ADDRESS,
                chunk_size: 10,
                continuation_token: continuation_token,
            };
            ({ events, continuation_token } = await providerRPC.getEvents(filter));
            (0, chai_1.expect)(events.length).to.be.equal(0);
            (0, chai_1.expect)(continuation_token).to.be.null;
        });
        it("returns expected events on correct filter with chunk size", async function () {
            const transactions = [];
            for (let i = 0; i < 5; i++) {
                transactions.push((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT));
            }
            await context.createBlock(transactions);
            const filter = {
                from_block: "latest",
                to_block: "latest",
                address: constants_1.FEE_TOKEN_ADDRESS,
                chunk_size: 4,
            };
            const events = await providerRPC.getEvents(filter);
            (0, chai_1.expect)(events.events.length).to.be.equal(4);
            (0, chai_1.expect)(events.continuation_token).to.be.equal("0,1,3");
            for (let i = 0; i < 2; i++) {
                const tx = await providerRPC.getTransactionByBlockIdAndIndex("latest", i);
                (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events.events[2 * i].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
                (0, chai_1.expect)(events.events[2 * i].transaction_hash).to.be.equal(tx.transaction_hash);
                (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events.events[2 * i + 1].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
                (0, chai_1.expect)(events.events[2 * i + 1].transaction_hash).to.be.equal(tx.transaction_hash);
            }
        });
        it("returns expected events on correct filter with continuation token", async function () {
            const transactions = [];
            for (let i = 0; i < 5; i++) {
                transactions.push((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT));
            }
            await context.createBlock(transactions);
            const skip = 3;
            const filter = {
                from_block: "latest",
                to_block: "latest",
                address: constants_1.FEE_TOKEN_ADDRESS,
                chunk_size: 4,
                continuation_token: `0,${skip - 1},${3}`,
            };
            const events = await providerRPC.getEvents(filter);
            (0, chai_1.expect)(events.events.length).to.be.equal(4);
            (0, chai_1.expect)(events.continuation_token).to.be.null;
            for (let i = 0; i < 2; i++) {
                const tx = await providerRPC.getTransactionByBlockIdAndIndex("latest", skip + i);
                (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events.events[2 * i].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
                (0, chai_1.expect)(events.events[2 * i].transaction_hash).to.be.equal(tx.transaction_hash);
                (0, chai_1.expect)((0, starknet_1.validateAndParseAddress)(events.events[2 * i + 1].from_address)).to.be.equal(constants_1.FEE_TOKEN_ADDRESS);
                (0, chai_1.expect)(events.events[2 * i + 1].transaction_hash).to.be.equal(tx.transaction_hash);
            }
        });
        it("returns expected events on correct filter with keys", async function () {
            await context.createBlock((0, utils_1.rpcTransfer)(providerRPC, ARGENT_CONTRACT_NONCE, constants_1.ARGENT_CONTRACT_ADDRESS, constants_1.MINT_AMOUNT));
            const tx = await providerRPC.getTransactionByBlockIdAndIndex("latest", 0);
            const block_hash_and_number = await providerRPC.getBlockHashAndNumber();
            const filter = {
                from_block: "latest",
                to_block: "latest",
                chunk_size: 1,
                keys: [[(0, utils_1.toHex)((0, utils_1.starknetKeccak)("transaction_executed"))]],
            };
            const events = await providerRPC.getEvents(filter);
            (0, chai_1.expect)(events.events.length).to.be.equal(1);
            (0, chai_1.expect)(events.continuation_token).to.be.equal("0,0,2");
            (0, chai_1.expect)(events.events[0]).to.deep.equal({
                transaction_hash: tx.transaction_hash,
                block_hash: block_hash_and_number.block_hash,
                block_number: block_hash_and_number.block_number,
                from_address: (0, utils_1.cleanHex)(constants_1.ARGENT_CONTRACT_ADDRESS),
                keys: [(0, utils_1.toHex)((0, utils_1.starknetKeccak)("transaction_executed"))],
                data: [tx.transaction_hash, "0x2", "0x1", "0x1"].map(utils_1.cleanHex),
            });
        });
    });
});
//# sourceMappingURL=test-events.js.map