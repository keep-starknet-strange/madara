import "@madara/api-augment";

import { expect } from "chai";

import { alice } from "../../util/accounts";
import { describeDevMadara } from "../../util/setup-dev-tests";


describeDevMadara("Pallet Starknet - block", (context) => {
  it("should connect to local node", async function () {
    const rdy = context.polkadotApi.isConnected;
    expect(rdy).to.be.true;
  });
});
