import "@keep-starknet-strange/madara-api-augment";
import { expect } from "chai";
import { LibraryError, RpcProvider } from "starknet";
import { describeDevMadara } from "../../util/setup-dev-tests";
import { FEE_TOKEN_ADDRESS } from "../constants";

describeDevMadara("Starknet RPC - Storage Test", (context) => {
  let providerRPC: RpcProvider;

  before(async function () {
    providerRPC = new RpcProvider({
      nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
      retries: 3,
    }); // substrate node
  });

  describe("getStorageAt", async () => {
    it("should return value from the fee contract storage", async function () {
      const value = await providerRPC.getStorageAt(
        FEE_TOKEN_ADDRESS,
        // ERC20_balances(0x02).low
        "0x1d8bbc4f93f5ab9858f6c0c0de2769599fb97511503d5bf2872ef6846f2146f",
        "latest",
      );
      // fees were paid during the transfer in the previous test so the value should be < u128::MAX
      expect(parseInt(value, 16)).to.be.greaterThan(0);
    });

    it("should return 0 if the storage slot is not set", async function () {
      const value = await providerRPC.getStorageAt(
        FEE_TOKEN_ADDRESS,
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        "latest",
      );
      expect(value).to.be.equal("0x0");
    });

    it("should raise if the contract does not exist", async function () {
      const storage = providerRPC.getStorageAt(
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        "latest",
      );
      await expect(storage)
        .to.eventually.be.rejectedWith("20: Contract not found")
        .and.be.an.instanceOf(LibraryError);
    });
  });
});
