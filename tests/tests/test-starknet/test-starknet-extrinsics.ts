import "@keep-starknet-strange/madara-api-augment";

import { expect } from "chai";

import { describeDevMadara } from "../../util/setup-dev-tests";
import {
  declare,
  deploy,
  initialize,
  mint,
  transfer,
} from "../../util/starknet";
import { jumpBlocks } from "../../util/block";

const mintAmount =
  "0x0000000000000000000000000000000000000000000000000000000000000001";
const contractAddress =
  "0x0000000000000000000000000000000000000000000000000000000000000101";
const tokenClassHash =
  "0x025ec026985a3bf9d0cc1fe17326b245bfdc3ff89b8fde106242a3ea56c5a918";

describeDevMadara("Pallet Starknet - Extrinsics", (context) => {
  it("should connect to local node", async function () {
    const rdy = context.polkadotApi.isConnected;
    expect(rdy).to.be.true;
  });

  it.only("should jump 10 blocks", async function () {
    const rdy = context.polkadotApi.isConnected;
    expect(rdy).to.be.true;

    await jumpBlocks(context, 10);
  });

  it("should declare a new contract class", async function () {
    const blockHash = await declare(
      context.polkadotApi,
      context.alice,
      contractAddress,
      tokenClassHash
    );

    console.log("blockhash: ", blockHash);

    expect(blockHash).to.not.be.undefined;
  });

  // it("should deploy a new contract", async function () {
  //   const address = await deploy(
  //     context.polkadotApi,
  //     context.alice,
  //     contractAddress,
  //     tokenClassHash
  //   );

  //   console.log("address: ", address);

  //   expect(address).to.not.be.undefined;
  // });

  it("should deploy, initialize, mint and then transfer", async function () {
    const address = await deploy(
      context.polkadotApi,
      context.alice,
      contractAddress,
      tokenClassHash
    );

    console.log("address: ", address);

    expect(address).to.not.be.undefined;

    await initialize(
      context.polkadotApi,
      context.alice,
      contractAddress,
      address
    );

    await mint(
      context.polkadotApi,
      context.alice,
      contractAddress,
      address,
      "0x0000000000000000000000000000000000000000000000000000000000000100"
    );

    await transfer(
      context.polkadotApi,
      context.alice,
      contractAddress,
      address,
      contractAddress,
      mintAmount
    );
  });
});
