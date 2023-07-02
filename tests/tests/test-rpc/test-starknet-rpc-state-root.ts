import { RpcProvider } from "starknet";
import { describeDevMadara } from "../../util/setup-dev-tests";
import { expect } from "chai";
import { jumpBlocks } from "../../util/block";

describeDevMadara(
  "Starknet RPC - State Root Enabled",
  (context) => {
    let providerRPC: RpcProvider;

    before(async function () {
      providerRPC = new RpcProvider({
        nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
        retries: 3,
      }); // substrate node
    });

    describe.only("state root", async () => {
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

        const query = {
          jsonrpc: "2.0",
          method: "pathfinder_getProof",
          params: {
            block_id: "latest",
            contract_address:
              "0x23371b227eaecd8e8920cd429d2cd0f3fee6abaacca08d3ab82a7cdd",
            keys: ["0x1", "0xfffffffff"],
          },
          id: 0,
        };
        const storage_proof = providerRPC.fetch("POST", query);
        console.log(storage_proof);
      });

      it("should return proof of membership", async function () {
        await jumpBlocks(context, 1);

        const query = {
          jsonrpc: "2.0",
          method: "pathfinder_getProof",
          params: {
            block_id: "latest",
            contract_address:
              "0x23371b227eaecd8e8920cd429d2cd0f3fee6abaacca08d3ab82a7cdd",
            keys: ["0x1", "0xfffffffff"],
          },
          id: 0,
        };
        const storage_proof = providerRPC.fetch("POST", query);
        console.log(storage_proof);
      });
    });
  },
  "madara-state-root"
);
