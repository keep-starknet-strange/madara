import "@keep-starknet-strange/madara-api-augment";
import { expect } from "chai";
import { LibraryError, RpcProvider } from "starknet";
import { describeDevMadara } from "../../util/setup-dev-tests";
import { rpcTransfer } from "../../util/utils";
import { ARGENT_CONTRACT_ADDRESS, MINT_AMOUNT } from "../constants";
import { assert } from "@polkadot/util";

// keep "let" over "const" as the nonce is passed by reference
// to abstract the increment
// eslint-disable-next-line prefer-const
let ARGENT_CONTRACT_NONCE = { value: 0 };

describeDevMadara(
  "Starknet RPC - State Update Test",
  (context) => {
    let providerRPC: RpcProvider;

    before(async function () {
      providerRPC = new RpcProvider({
        nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
        retries: 3,
      }); // substrate node
    });

    describe("getStateUpdate", async () => {
      it("should return latest block state update", async function () {
        await context.createBlock(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT,
          ),
          {
            finalize: true,
          },
        );
        const stateUpdate = await providerRPC.getStateUpdate("latest");

        const latestBlock = await providerRPC.getBlockHashAndNumber();

        // TODO: Add real values
        expect(stateUpdate).to.not.be.undefined;
        assert(
          "block_hash" in stateUpdate,
          "block_hash is not in stateUpdate which means it's still pending",
        );
        expect(stateUpdate.block_hash).to.be.equal(latestBlock.block_hash);
        expect(stateUpdate.state_diff).to.deep.equal({
          storage_diffs: [],
          deprecated_declared_classes: [],
          declared_classes: [],
          deployed_contracts: [],
          replaced_classes: [],
          nonces: [],
        });
      });

      it("should return anterior block state update", async function () {
        const anteriorBlock = await providerRPC.getBlockHashAndNumber();

        await context.createBlock(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT,
          ),
          {
            finalize: true,
          },
        );

        const stateUpdate = await providerRPC.getStateUpdate(
          anteriorBlock.block_hash,
        );

        // TODO: Add real values
        expect(stateUpdate).to.not.be.undefined;
        assert(
          "block_hash" in stateUpdate,
          "block_hash is not in stateUpdate which means it's still pending",
        );
        expect(stateUpdate.block_hash).to.be.equal(anteriorBlock.block_hash);
        expect(stateUpdate.state_diff).to.deep.equal({
          storage_diffs: [],
          deprecated_declared_classes: [],
          declared_classes: [],
          deployed_contracts: [],
          replaced_classes: [],
          nonces: [],
        });
      });

      it("should throw block not found error", async function () {
        const transaction = providerRPC.getStateUpdate("0x123");
        await expect(transaction)
          .to.eventually.be.rejectedWith("24: Block not found")
          .and.be.an.instanceOf(LibraryError);
      });
    });
  },
  { runNewNode: true },
);
