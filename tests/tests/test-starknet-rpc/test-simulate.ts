import "@keep-starknet-strange/madara-api-augment";
import { expect } from "chai";
import {
  Account,
  AccountInvocationItem,
  LibraryError,
  RpcProvider,
  constants,
  hash,
  validateAndParseAddress,
  Signer,
} from "starknet";
import { createAndFinalizeBlock, jumpBlocks } from "../../util/block";
import { describeDevMadara } from "../../util/setup-dev-tests";
import { rpcTransfer, toHex } from "../../util/utils";
import {
  ACCOUNT_CONTRACT,
  ARGENT_ACCOUNT_CLASS_HASH,
  ARGENT_CONTRACT_ADDRESS,
  ARGENT_PROXY_CLASS_HASH,
  ERC721_CONTRACT,
  ERC20_CONTRACT,
  FEE_TOKEN_ADDRESS,
  MINT_AMOUNT,
  SALT,
  SIGNER_PRIVATE,
  SIGNER_PUBLIC,
  TEST_CONTRACT_ADDRESS,
  TOKEN_CLASS_HASH,
  UDC_CONTRACT_ADDRESS,
  DEPLOY_ACCOUNT_COST,
  TEST_CAIRO_1_SIERRA,
  TEST_CAIRO_1_CASM,
  CAIRO_1_ACCOUNT_CONTRACT,
} from "../constants";
import { InvokeTransaction } from "./types";
import { numberToHex } from "@polkadot/util";

// keep "let" over "const" as the nonce is passed by reference
// to abstract the increment
// eslint-disable-next-line prefer-const
let ARGENT_CONTRACT_NONCE = { value: 0 };
const CAIRO_1_NO_VALIDATE_ACCOUNT = { value: 0 };

describeDevMadara("Starknet RPC - Transactions Test", (context) => {
  let providerRPC: RpcProvider;

  before(async function () {
    providerRPC = new RpcProvider({
      nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
      retries: 3,
    }); // substrate node
  });

  describe("simulateTransaction", async () => {
    it("should simulate invoke transaction successfully", async function () {
      const tx = {
        contractAddress: ACCOUNT_CONTRACT,
        calldata: [
          TEST_CONTRACT_ADDRESS,
          "0x36fa6de2810d05c3e1a0ebe23f60b9c2f4629bbead09e5a9704e1c5632630d5",
          "0x0",
        ],
        signature: [],
      };

      const nonce = await providerRPC.getNonceForAddress(
        ACCOUNT_CONTRACT,
        "latest",
      );

      const txDetails = {
        nonce: nonce,
        version: "0x1",
      };

      const invocation: AccountInvocationItem = {
        type: "INVOKE_FUNCTION",
        ...tx,
        ...txDetails,
      };

      const simulationResults = await providerRPC.getSimulateTransaction([invocation], {
        blockIdentifier: "latest",
      });
      console.log(simulationResults);
    });
  });
});
