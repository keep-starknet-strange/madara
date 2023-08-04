import "@keep-starknet-strange/madara-api-augment";
import { Account, AccountInvocationItem, RpcProvider, hash } from "starknet";
import { jumpBlocks } from "../../util/block";
import { describeDevMadara } from "../../util/setup-dev-tests";
import { rpcTransfer } from "../../util/utils";
import {
  ACCOUNT_CONTRACT,
  ARGENT_CONTRACT_ADDRESS,
  ERC20_CONTRACT,
  SALT,
  SIGNER_PRIVATE,
  TEST_CONTRACT_ADDRESS,
  DEPLOY_ACCOUNT_COST,
  CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH,
} from "../constants";

// In order to run just this test suite:
// MADARA_LOG=jsonrpsee_core=trace DISPLAY_LOG=1 npx mocha -r ts-node/register --require 'tests/setup-tests.ts' 'tests/test-starknet-rpc/test-simulate.ts'

describeDevMadara("Starknet RPC - Simulation Test", (context) => {
  let providerRPC: RpcProvider;

  before(async function () {
    providerRPC = new RpcProvider({
      nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
      retries: 3,
    }); // substrate node

    // Otherwise we get "ERC20: cannot transfer to the zero address"
    await context.createBlock();
  });

  describe("simulateTransaction", async () => {
    it("should simulate invoke transaction successfully", async function () {
      const nonce = await providerRPC.getNonceForAddress(
        ACCOUNT_CONTRACT,
        "latest",
      );

      const invocation: AccountInvocationItem = {
        type: "INVOKE_FUNCTION",
        contractAddress: ACCOUNT_CONTRACT,
        calldata: [
          TEST_CONTRACT_ADDRESS,
          "0x36fa6de2810d05c3e1a0ebe23f60b9c2f4629bbead09e5a9704e1c5632630d5",
          "0x0",
        ],
        signature: [],
        nonce,
        version: 1,
      };

      await providerRPC.getSimulateTransaction([invocation], {
        blockIdentifier: "latest",
        // skipValidate: true,
      });
    });

    it("should simulate account deploy transaction successfully", async function () {
      const calldata = [0x123];  // Public key
      const deployedContractAddress = hash.calculateContractAddressFromHash(
        SALT,
        CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH,  // NoValidate account contract (Cairo 1)
        calldata,
        0,
      );

      // fund address
      await rpcTransfer(
        providerRPC,
        { value: 0 },
        deployedContractAddress,
        DEPLOY_ACCOUNT_COST,
      );
      await jumpBlocks(context, 1);

      const invocation: AccountInvocationItem = {
        type: "DEPLOY_ACCOUNT",
        constructorCalldata: calldata,
        classHash: CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH,
        addressSalt: SALT,
        signature: [],
        nonce: 0,
        version: 1,
      };

      await providerRPC.getSimulateTransaction([invocation], {
        blockIdentifier: "latest",
        // skipValidate: true,
      });
    });

    it("should should simulate declare transaction", async function () {
      // computed via: starkli class-hash ./cairo-contracts/build/ERC20.json
      // the above command should be used at project root
      const classHash =
        "0x372ee6669dc86563007245ed7343d5180b96221ce28f44408cff2898038dbd4";

      const account = new Account(
        providerRPC,
        ARGENT_CONTRACT_ADDRESS,
        SIGNER_PRIVATE,
      );

      await account.simulateTransaction(
        [
          {
            type: "DECLARE",
            contract: ERC20_CONTRACT,
            classHash,
          },
        ],
        {
          blockIdentifier: "latest",
        },
      );
    });
  });
});
