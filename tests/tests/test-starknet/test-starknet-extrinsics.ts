import "@keep-starknet-strange/madara-api-augment";

import { expect } from "chai";

import { hexFixLength, numberToHex } from "@polkadot/util";
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
  ERC_20_CONTRACT_CLASS_HASH,
  FEE_TOKEN_ADDRESS,
  MINT_AMOUNT,
  NFT_CONTRACT_ADDRESS,
} from "../constants";
import { RpcProvider, hash } from "starknet";

describeDevMadara(
  "Pallet Starknet - Extrinsics",
  (context) => {
    let providerRPC: RpcProvider;

    before(async function () {
      providerRPC = new RpcProvider({
        nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
        retries: 3,
      }); // substrate node
    });

    it("should connect to local node", async function () {
      const rdy = context.polkadotApi.isConnected;
      expect(rdy).to.be.true;
    });

    it("should jump 10 blocks", async function () {
      const rdy = context.polkadotApi.isConnected;
      expect(rdy).to.be.true;

      await jumpBlocks(context, 10);
    });

    // TODO: fix testing for declare
    it.skip("should declare a new contract class", async function () {
      const {
        result: { events },
      } = await context.createBlock(
        declare(
          context.polkadotApi,
          CONTRACT_ADDRESS,
          ERC_20_CONTRACT_CLASS_HASH,
        ),
      );

      expect(
        events.find(
          ({ event: { section, method } }) =>
            section == "system" && method == "ExtrinsicSuccess",
        ),
      ).to.exist;
    });

    it("should deploy a new contract", async function () {
      const deployedContractAddress = hash.calculateContractAddressFromHash(
        "0x0000000000000000000000000000000000000000000000000000000000000001",
        ERC_20_CONTRACT_CLASS_HASH,
        [
          "0x000000000000000000000000000000000000000000000000000000000000000A", // Name
          "0x0000000000000000000000000000000000000000000000000000000000000001", // Symbol
          "0x0000000000000000000000000000000000000000000000000000000000000002", // Decimals
          "0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", // Initial supply low
          "0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", // Initial supply high
          "0x0000000000000000000000000000000000000000000000000000000000001111", // recipient
        ],
        0,
      );
      // ERC20_balances(0x1111).low = 0x72943352085ed3fbe3b8ff53a6aef9da8d893ccdab99bd5223d765f1a22735f
      const storageAddress =
        "0x72943352085ed3fbe3b8ff53a6aef9da8d893ccdab99bd5223d765f1a22735f";

      const {
        result: { events },
      } = await context.createBlock(
        deploy(
          context.polkadotApi,
          CONTRACT_ADDRESS,
          ERC_20_CONTRACT_CLASS_HASH,
        ),
      );

      const classHash = await providerRPC.getClassHashAt(
        deployedContractAddress,
        "latest",
      );
      expect(hexFixLength(classHash, 256, true)).to.equal(
        ERC_20_CONTRACT_CLASS_HASH,
      );

      const balance = await providerRPC.getStorageAt(
        deployedContractAddress,
        storageAddress,
        "latest",
      );
      expect(balance).to.equal("0xfffffffffffffffffffffffffffffff");

      expect(
        events.find(
          ({ event: { section, method } }) =>
            section == "system" && method == "ExtrinsicSuccess",
        ),
      ).to.exist;
    });

    it("should execute a transfer", async function () {
      const recepientAddress =
        "0x00000000000000000000000000000000000000000000000000000000deadbeef";
      // ERC20_balances(0xdeadbeef).low = 0x4c761778f11aa10fc40190ff3127637fe00dc59bfa557bd4c8beb30a178f016
      const storageKey =
        "0x4c761778f11aa10fc40190ff3127637fe00dc59bfa557bd4c8beb30a178f016";

      const balanceBefore = await providerRPC.getStorageAt(
        FEE_TOKEN_ADDRESS,
        storageKey,
        "latest",
      );
      expect(balanceBefore).to.equal("0x0");

      const nonce = 1;
      const {
        result: { events },
      } = await context.createBlock(
        transfer(
          context.polkadotApi,
          CONTRACT_ADDRESS,
          FEE_TOKEN_ADDRESS,
          recepientAddress,
          MINT_AMOUNT,
          nonce,
        ),
      );

      const balanceAfter = await providerRPC.getStorageAt(
        FEE_TOKEN_ADDRESS,
        storageKey,
        "latest",
      );
      expect(balanceAfter).to.equal("0x1");

      expect(
        events.find(
          ({ event: { section, method } }) =>
            section == "system" && method == "ExtrinsicSuccess",
        ),
      ).to.exist;
    });

    it("mint NFTs", async function () {
      const recepientAddress =
        "0x00000000000000000000000000000000000000000000000000000000deadbeef";
      // ERC721_balances(0xdeadbeef).low = 0x1a564c2a8ac0aa99f656ca20cae9b7ed3aff27fa129aea20969feb46dd94e73
      const storageKey =
        "0x1a564c2a8ac0aa99f656ca20cae9b7ed3aff27fa129aea20969feb46dd94e73";
      // ERC721_owners(1).low = 0x79c7fb99f54e3fcd8f9894e87b6004eaf8a3a51318d79db735475363c130030

      const balanceBefore = await providerRPC.getStorageAt(
        NFT_CONTRACT_ADDRESS,
        storageKey,
        "latest",
      );
      expect(balanceBefore).to.equal("0x0");

      const {
        result: { events },
      } = await context.createBlock(
        mintERC721(
          context.polkadotApi, // api
          CONTRACT_ADDRESS, // senderAddress
          recepientAddress, // recipientAddress
          numberToHex(1, 256), // tokenID
          2, // nonce
        ),
      );

      const balanceAfter = await providerRPC.getStorageAt(
        NFT_CONTRACT_ADDRESS,
        storageKey,
        "latest",
      );
      expect(balanceAfter).to.equal("0x1");

      expect(
        events.find(
          ({ event: { section, method } }) =>
            section == "system" && method == "ExtrinsicSuccess",
        ),
      ).to.exist;
    });

    it("deploys ERC20 contract via UDC", async function () {
      const deployedContractAddress = hash.calculateContractAddressFromHash(
        "0x0000000000000000000000000000000000000000000000000000000000000001",
        ERC_20_CONTRACT_CLASS_HASH,
        [
          "0x000000000000000000000000000000000000000000000000000000000000000A", // Name
          "0x000000000000000000000000000000000000000000000000000000000000000B", // Symbol
          "0x0000000000000000000000000000000000000000000000000000000000000002", // Decimals
          "0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", // Initial supply low
          "0x000000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", // Initial supply high
          "0x0000000000000000000000000000000000000000000000000000000000001111", // recipient
        ],
        0,
      );

      const {
        result: { events },
      } = await context.createBlock(
        deployTokenContractUDC(
          context.polkadotApi,
          CONTRACT_ADDRESS,
          ERC_20_CONTRACT_CLASS_HASH,
          "0x0000000000000000000000000000000000000000000000000000000000000001",
          false,
          3,
        ),
      );
      // ERC20_balances(0x1111).low = 0x72943352085ed3fbe3b8ff53a6aef9da8d893ccdab99bd5223d765f1a22735f
      const storageAddress =
        "0x72943352085ed3fbe3b8ff53a6aef9da8d893ccdab99bd5223d765f1a22735f";

      const classHash = await providerRPC.getClassHashAt(
        deployedContractAddress,
        "latest",
      );
      expect(hexFixLength(classHash, 256, true)).to.equal(
        ERC_20_CONTRACT_CLASS_HASH,
      );

      const balance = await providerRPC.getStorageAt(
        deployedContractAddress,
        storageAddress,
        "latest",
      );
      expect(balance).to.equal("0xfffffffffffffffffffffffffffffff");

      expect(
        events.find(
          ({ event: { section, method } }) =>
            section == "system" && method == "ExtrinsicSuccess",
        ),
      ).to.exist;
    });
  },
  { runNewNode: true },
);
