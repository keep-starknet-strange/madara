import "@madara/api-augment";
import { u8aToHex } from "@polkadot/util";

import { expect } from "chai";

import { describeDevMadara } from "../../util/setup-dev-tests";
import { declare, deploy, initialize } from "../../util/starknet";

const mintAmount =
  "0x0000000000000000000000000000000000000000000000000000000000000001";
const contractAddress =
  "0x0000000000000000000000000000000000000000000000000000000000000101";
const tokenClassHash =
  "0x025ec026985a3bf9d0cc1fe17326b245bfdc3ff89b8fde106242a3ea56c5a918";

describeDevMadara("Pallet Starknet - block", (context) => {
  it("should connect to local node", async function () {
    const rdy = context.polkadotApi.isConnected;
    expect(rdy).to.be.true;
  });

  it("should declare a new contract class", async function () {
    await declare(context.polkadotApi, context.alice, contractAddress, tokenClassHash);
  });

  it("should declare then deploy", async function () {
    await declare(context.polkadotApi, context.alice, contractAddress, tokenClassHash);
    await deploy(context.polkadotApi, context.alice, contractAddress, tokenClassHash);
  });
});
