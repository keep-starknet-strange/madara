import "@keep-starknet-strange/madara-api-augment";

import { expect } from "chai";

import { jumpBlocks } from "../../util/block";
import { describeDevMadara } from "../../util/setup-dev-tests";
import { declare, deploy, transfer } from "../../util/starknet";

const mintAmount =
  "0x0000000000000000000000000000000000000000000000000000000000000001";
const contractAddress =
  "0x0000000000000000000000000000000000000000000000000000000000000001";
const openzeppelinContractAddress =
  "0x0000000000000000000000000000000000000000000000000000000000011111";
const feeTokenAddress =
  "0x040e59c2c182a58fb0a74349bfa4769cbbcba32547591dd3fb1def8623997d00";
const tokenClassHash =
  "0x0000000000000000000000000000000000000000000000000000000000001000";

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
      declare(context.polkadotApi, contractAddress, tokenClassHash)
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
      deploy(context.polkadotApi, contractAddress, tokenClassHash)
    );

    expect(
      events.find(
        ({ event: { section, method } }) =>
          section == "system" && method == "ExtrinsicSuccess"
      )
    ).to.exist;
  });

  it("should deploy a new account and validate", async function () {
    const {
      result: { events },
    } = await context.createBlock(
      deploy(context.polkadotApi, openzeppelinContractAddress, tokenClassHash)
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
        contractAddress,
        feeTokenAddress,
        contractAddress,
        mintAmount,
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
