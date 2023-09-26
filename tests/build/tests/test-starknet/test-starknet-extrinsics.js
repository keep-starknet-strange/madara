"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
require("@keep-starknet-strange/madara-api-augment");
const chai_1 = require("chai");
const block_1 = require("../../util/block");
const setup_dev_tests_1 = require("../../util/setup-dev-tests");
const starknet_1 = require("../../util/starknet");
const constants_1 = require("../constants");
(0, setup_dev_tests_1.describeDevMadara)(
  "Pallet Starknet - Extrinsics",
  (context) => {
    it("should connect to local node", async function () {
      const rdy = context.polkadotApi.isConnected;
      (0, chai_1.expect)(rdy).to.be.true;
    });
    it("should jump 10 blocks", async function () {
      const rdy = context.polkadotApi.isConnected;
      (0, chai_1.expect)(rdy).to.be.true;
      await (0, block_1.jumpBlocks)(context, 10);
    });
    it.skip("should declare a new contract class", async function () {
      const {
        result: { events },
      } = await context.createBlock(
        (0, starknet_1.declare)(
          context.polkadotApi,
          constants_1.CONTRACT_ADDRESS,
          constants_1.TOKEN_CLASS_HASH,
        ),
      );
      (0, chai_1.expect)(
        events.find(
          ({ event: { section, method } }) =>
            section == "system" && method == "ExtrinsicSuccess",
        ),
      ).to.exist;
    });
    it("should deploy a new contract", async function () {
      const {
        result: { events },
      } = await context.createBlock(
        (0, starknet_1.deploy)(
          context.polkadotApi,
          constants_1.CONTRACT_ADDRESS,
          constants_1.TOKEN_CLASS_HASH,
        ),
      );
      (0, chai_1.expect)(
        events.find(
          ({ event: { section, method } }) =>
            section == "system" && method == "ExtrinsicSuccess",
        ),
      ).to.exist;
    });
    it("should execute a transfer", async function () {
      const nonce = 1;
      const {
        result: { events },
      } = await context.createBlock(
        (0, starknet_1.transfer)(
          context.polkadotApi,
          constants_1.CONTRACT_ADDRESS,
          constants_1.FEE_TOKEN_ADDRESS,
          constants_1.CONTRACT_ADDRESS,
          constants_1.MINT_AMOUNT,
          nonce,
        ),
      );
      (0, chai_1.expect)(
        events.find(
          ({ event: { section, method } }) =>
            section == "system" && method == "ExtrinsicSuccess",
        ),
      ).to.exist;
    });
  },
);
//# sourceMappingURL=test-starknet-extrinsics.js.map
