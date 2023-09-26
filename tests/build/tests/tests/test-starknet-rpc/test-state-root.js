"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const starknet_1 = require("starknet");
const setup_dev_tests_1 = require("../../util/setup-dev-tests");
const chai_1 = require("chai");
const block_1 = require("../../util/block");
(0, setup_dev_tests_1.describeDevMadara)(
  "Starknet RPC - State Root Enabled Test",
  (context) => {
    let providerRPC;
    before(async function () {
      providerRPC = new starknet_1.RpcProvider({
        nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
        retries: 3,
      });
    });
    describe("state root", async () => {
      it("should return default when disabled", async function () {
        const latestBlock = await providerRPC.getBlock("latest");
        (0, chai_1.expect)(latestBlock.new_root).to.eq("0x0");
      });
      it("should return default when enabled", async function () {
        await (0, block_1.jumpBlocks)(context, 2);
        const latestBlock = await providerRPC.getBlock("latest");
        (0, chai_1.expect)(latestBlock.new_root).to.eq(
          "0x4e65560d4b1751b0c3455f9f4e3e0ae0c41c4929796659ceec256f1aea08e28",
        );
      });
    });
    describe("getProof", async () => {
      it("should return proof of non-membership", async function () {
        await (0, block_1.jumpBlocks)(context, 1);
        const params = {
          get_proof_input: {
            block_id: "latest",
            contract_address: "0x111222333",
            keys: ["0x1", "0xfffffffff"],
          },
        };
        let storage_proof = await providerRPC.fetch(
          "starknet_getProof",
          params,
        );
        storage_proof = await storage_proof.json();
        (0, chai_1.expect)(storage_proof["result"]["contract_data"]).to.be.null;
      });
      it("should return proof of membership", async function () {
        await (0, block_1.jumpBlocks)(context, 1);
        const params = {
          get_proof_input: {
            block_id: "latest",
            contract_address: "0x2",
            keys: ["0x1", "0xfffffffff"],
          },
        };
        let storage_proof = await providerRPC.fetch(
          "starknet_getProof",
          params,
        );
        storage_proof = await storage_proof.json();
        (0, chai_1.expect)(
          storage_proof["result"]["contract_data"]["root"],
        ).to.be.eq(
          "2137650382361045467996332368791861747902403628779494221252963710317158396736",
        );
      });
    });
  },
  { runNewNode: true },
);
//# sourceMappingURL=test-state-root.js.map
