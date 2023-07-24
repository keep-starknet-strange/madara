import "@keep-starknet-strange/madara-api-augment";
import { expect } from "chai";
import { RpcProvider } from "starknet";
import { describeDevMadara } from "../../util/setup-dev-tests";
import { CHAIN_ID_STARKNET_TESTNET } from "../constants";

describeDevMadara("Starknet RPC - Config Test", (context) => {
  let providerRPC: RpcProvider;

  before(async function () {
    providerRPC = new RpcProvider({
      nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
      retries: 3,
    }); // substrate node
  });

  describe("getChainId", async () => {
    it("should return the correct value", async function () {
      const chainId = await providerRPC.getChainId();

      expect(chainId).to.not.be.undefined;
      expect(chainId).to.be.equal(CHAIN_ID_STARKNET_TESTNET);
    });
  });
});
