import "@keep-starknet-strange/madara-api-augment";
import { expect } from "chai";
import {
  Account,
  AccountInvocationItem,
  RpcProvider,
  hash,
  Sequencer,
} from "starknet";
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
  TEST_CAIRO_1_SIERRA,
  TEST_CAIRO_1_CASM,
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

      const res = await providerRPC.getSimulateTransaction([invocation], {
        blockIdentifier: "latest",
      });
      expect(res.length).to.be.equal(1);
      expect(
        res[0].transaction_trace as Sequencer.TransactionTraceResponse,
      ).to.have.property("execute_invocation");
      expect(
        res[0].transaction_trace as Sequencer.TransactionTraceResponse,
      ).to.have.property("validate_invocation");
      //expect(res[0].transaction_trace as Sequencer.TransactionTraceResponse).to.have.property("fee_transfer_invocation");
    });

    it("should simulate account deploy transaction successfully", async function () {
      const calldata = [0x123]; // Public key
      const deployedContractAddress = hash.calculateContractAddressFromHash(
        SALT,
        CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH, // NoValidate account contract (Cairo 1)
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

      const res = await providerRPC.getSimulateTransaction([invocation], {
        blockIdentifier: "latest",
      });
      expect(res.length).to.be.equal(1);
      expect(
        res[0].transaction_trace as Sequencer.TransactionTraceResponse,
      ).to.have.property("constructor_invocation");
      expect(
        res[0].transaction_trace as Sequencer.TransactionTraceResponse,
      ).to.have.property("validate_invocation");
      //expect(res[0].transaction_trace as Sequencer.TransactionTraceResponse).to.have.property("fee_transfer_invocation");
    });

    it("should simulate declare transaction successfully and not mutate the state", async function () {
      // computed via: starkli class-hash ./cairo-contracts/build/ERC20.json
      // the above command should be used at project root
      const classHash =
        "0x372ee6669dc86563007245ed7343d5180b96221ce28f44408cff2898038dbd4";

      const account = new Account(
        providerRPC,
        ARGENT_CONTRACT_ADDRESS,
        SIGNER_PRIVATE,
      );

      const res = await account.simulateTransaction(
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
      expect(res.length).to.be.equal(1);
      expect(
        res[0].transaction_trace as Sequencer.TransactionTraceResponse,
      ).to.have.property("validate_invocation");
      //expect(res[0].transaction_trace as Sequencer.TransactionTraceResponse).to.have.property("fee_transfer_invocation");

      // Making sure simulation actually leaves no changes

      await jumpBlocks(context, 1);

      await expect(
        providerRPC.getClass(classHash, "latest"),
      ).to.be.rejectedWith("28: Class hash not found");
    });

    it("should run simulation on a specified block in the past", async function () {
      // Note that simulating declare v1 (apparently) does not throw errors if class is already declared
      // Thus here we are using declare v2 with cairo v1 contract
      const account = new Account(
        providerRPC,
        ARGENT_CONTRACT_ADDRESS,
        SIGNER_PRIVATE,
      );

      const nonce = await providerRPC.getNonceForAddress(
        ARGENT_CONTRACT_ADDRESS,
        "latest",
      );

      await account.declare(
        {
          casm: TEST_CAIRO_1_CASM,
          contract: TEST_CAIRO_1_SIERRA,
        },
        { nonce, version: 2 },
      );

      await jumpBlocks(context, 1);

      // Make sure simulation would fail now

      await expect(
        account.simulateTransaction(
          [
            {
              type: "DECLARE",
              casm: TEST_CAIRO_1_CASM,
              contract: TEST_CAIRO_1_SIERRA,
            },
          ],
          {
            blockIdentifier: "latest",
          },
        ),
      ).to.be.rejectedWith("40: Contract error");

      // But if we rewind 1 block back, everything should be ok
      // We need to set the nonce manually though

      const block_hash_and_number = await providerRPC.getBlockHashAndNumber();

      const res = await account.simulateTransaction(
        [
          {
            type: "DECLARE",
            casm: TEST_CAIRO_1_CASM,
            contract: TEST_CAIRO_1_SIERRA,
          },
        ],
        {
          blockIdentifier: block_hash_and_number.block_number - 1,
          nonce,
        },
      );
      expect(res.length).to.be.equal(1);
      expect(
        res[0].transaction_trace as Sequencer.TransactionTraceResponse,
      ).to.have.property("validate_invocation");
      //expect(res[0].transaction_trace as Sequencer.TransactionTraceResponse).to.have.property("fee_transfer_invocation");
    });
  });
});
