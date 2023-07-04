import { RpcProvider } from "starknet";
import { describeDevMadara } from "../../util/setup-dev-tests";
import { expect } from "chai";
import { jumpBlocks } from "../../util/block";

describeDevMadara("Starknet RPC - State Root Enabled", (context) => {
  let providerRPC: RpcProvider;

  before(async function () {
    providerRPC = new RpcProvider({
      nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
      retries: 3,
    }); // substrate node
  });

  describe("state root", async () => {
    it("should return default when enabled", async function () {
      await jumpBlocks(context, 1);

      const latestBlock = await providerRPC.getBlock("latest");
      expect(latestBlock.new_root).to.eq(
        "0x46369f2573a77b6d4b4f3f14d3eab964b52886b55c3f5ece1a5ae6cb1f81e7b"
      );
    });
  });

  describe("getProof", async () => {
    it("should return proof of non-membership", async function () {
      await jumpBlocks(context, 1);

      const params = {
        get_proof_input: {
          block_id: "latest",
          contract_address: "0x111222333",
          keys: ["0x1", "0xfffffffff"],
        },
      };
      let storage_proof = await providerRPC.fetch("starknet_getProof", params);
      storage_proof = await storage_proof.json();

      // Check contract root
      expect(storage_proof["result"]["contract_data"]).to.be.null;
    });

    it("should return proof of membership", async function () {
      await jumpBlocks(context, 1);

      const params = {
        get_proof_input: {
          block_id: "latest",
          contract_address: "0x2",
          keys: ["0x1", "0xfffffffff"],
        },
      };
      let storage_proof = await providerRPC.fetch("starknet_getProof", params);
      storage_proof = await storage_proof.json();

      // Check contract root
      expect(storage_proof["result"]["contract_data"]["root"]).to.be.eq(
        "1245075994121459795339981889219606020533793304969303161130350131342227964700"
      );
    });
  });
});
