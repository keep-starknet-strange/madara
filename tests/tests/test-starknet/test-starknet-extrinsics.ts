import "@keep-starknet-strange/madara-api-augment";

import { expect } from "chai";

import { jumpBlocks } from "../../util/block";
import { describeDevMadara } from "../../util/setup-dev-tests";
import { declare, deploy, transfer } from "../../util/starknet";
import {
  CONTRACT_ADDRESS,
  FEE_TOKEN_ADDRESS,
  MINT_AMOUNT,
  TOKEN_CLASS_HASH,
} from "../constants";

describeDevMadara("Pallet Starknet - Extrinsics", (context) => {
  it("should connect to local node", async function () {
    const rdy = context.polkadotApi.isConnected;
    expect(rdy).to.be.true;
  });

  it("should jump 10 blocks", async function () {
    const rdy = context.polkadotApi.isConnected;
    expect(rdy).to.be.true;

    await jumpBlocks(context, 10);
  });

  it.skip("should declare a new contract class", async function () {
    const {
      result: { events },
    } = await context.createBlock(
      declare(context.polkadotApi, CONTRACT_ADDRESS, TOKEN_CLASS_HASH)
    );

    expect(
      events.find(
        ({ event: { section, method } }) =>
          section == "system" && method == "ExtrinsicSuccess"
      )
    ).to.exist;
  });

  it("should deploy a new contract", async function () {
    const {
      result: { events },
    } = await context.createBlock(
      deploy(context.polkadotApi, CONTRACT_ADDRESS, TOKEN_CLASS_HASH)
    );

    expect(
      events.find(
        ({ event: { section, method } }) =>
          section == "system" && method == "ExtrinsicSuccess"
      )
    ).to.exist;
  });

  it("should execute a transfer", async function () {
    const nonce = 1;
    const {
      result: { events },
    } = await context.createBlock(
      transfer(
        context.polkadotApi,
        CONTRACT_ADDRESS,
        FEE_TOKEN_ADDRESS,
        CONTRACT_ADDRESS,
        MINT_AMOUNT,
        nonce
      )
    );

    expect(
      events.find(
        ({ event: { section, method } }) =>
          section == "system" && method == "ExtrinsicSuccess"
      )
    ).to.exist;
  });
});
