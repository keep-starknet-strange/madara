import "@madara/api-augment";

import { expect } from "chai";

import { alice } from "../../util/accounts";
import { describeDevMadara } from "../../util/setup-dev-tests";


describeDevMadara("Pallet Starknet - block", (context) => {
  it("should work", async function () {
    const rdy = context.polkadotApi.isConnected;
    console.log(rdy);
  });
});
