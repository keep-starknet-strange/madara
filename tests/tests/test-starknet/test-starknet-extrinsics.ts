import "@keep-starknet-strange/madara-api-augment";

import { expect } from "chai";

import { numberToHex } from "@polkadot/util";
import { jumpBlocks } from "../../util/block";
import { describeDevMadara } from "../../util/setup-dev-tests";
import {
  declare,
  deploy,
  deployTokenContractUDC,
  mintERC721,
  transfer,
} from "../../util/starknet";
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

  it("mint NFTs", async function () {
    const {
      result: { events },
    } = await context.createBlock(
      mintERC721(
        context.polkadotApi, // api
        CONTRACT_ADDRESS, // senderAddress
        CONTRACT_ADDRESS, // recipientAddress
        numberToHex(1, 256), // tokenID
        2 // nonce
      )
    );

    expect(
      events.find(
        ({ event: { section, method } }) =>
          section == "system" && method == "ExtrinsicSuccess"
      )
    ).to.exist;
  });

  it("deploys ERC20 contract via UDC", async function () {
    const {
      result: { events },
    } = await context.createBlock(
      deployTokenContractUDC(
        context.polkadotApi,
        CONTRACT_ADDRESS,
        TOKEN_CLASS_HASH,
        numberToHex(1, 256),
        false,
        3
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
