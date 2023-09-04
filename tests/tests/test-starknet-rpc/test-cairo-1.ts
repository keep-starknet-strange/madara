import "@keep-starknet-strange/madara-api-augment";
import { expect } from "chai";
import { Account, RpcProvider, hash } from "starknet";
import { jumpBlocks } from "../../util/block";
import { describeDevMadara } from "../../util/setup-dev-tests";
import { rpcTransfer, toHex } from "../../util/utils";
import {
  SALT,
  SIGNER_PRIVATE,
  CAIRO_1_ACCOUNT_CONTRACT,
  ERC20_CAIRO_1_CASM,
  ERC20_CAIRO_1_SIERRA,
  CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH,
} from "../constants";

// keep "let" over "const" as the nonce is passed by reference
// to abstract the increment
// eslint-disable-next-line prefer-const
let ARGENT_CONTRACT_NONCE = { value: 0 };
const CAIRO_1_NO_VALIDATE_ACCOUNT = { value: 0 };

describeDevMadara(
  "Starknet RPC - Cairo 1 Test",
  (context) => {
    let providerRPC: RpcProvider;

    before(async function () {
      providerRPC = new RpcProvider({
        nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
        retries: 3,
      }); // substrate node
    });

    describe("Cairo 1 full flow", async () => {
      it("should deploy a Cairo 1 account", async () => {
        const CONSTRUCTOR_CALLDATA = ["0x123"];
        const accountAddress = hash.calculateContractAddressFromHash(
          SALT,
          CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH,
          CONSTRUCTOR_CALLDATA,
          0,
        );

        const account = new Account(
          providerRPC,
          accountAddress,
          SIGNER_PRIVATE,
          "1",
        );

        // transfer native token to allow deployment
        await rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          accountAddress,
          "0xfffffffffffffffffffffffff",
        );
        await jumpBlocks(context, 1);

        // deploy the account
        await account.deploySelf(
          {
            classHash: CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH,
            addressSalt: SALT,
            constructorCalldata: CONSTRUCTOR_CALLDATA,
          },
          { maxFee: "12345678" },
        );
        await jumpBlocks(context, 1);

        expect(await providerRPC.getClassHashAt(accountAddress)).to.be.equal(
          CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH,
        );
      });

      it("should declare and deploy erc20 contract then transfer some tokens", async () => {
        const account = new Account(
          providerRPC,
          CAIRO_1_ACCOUNT_CONTRACT,
          SIGNER_PRIVATE, // it's the no validate account
          "1",
        );
        // computed via: starknetjs 5.14.1
        const classHash =
          "0x4596fa4856bbf13f3448a376d607f8852148b0e6be4b958cde2ca8471a72ede";
        const res = await account.declare(
          {
            casm: ERC20_CAIRO_1_CASM,
            contract: ERC20_CAIRO_1_SIERRA,
          },
          {
            nonce: CAIRO_1_NO_VALIDATE_ACCOUNT.value,
            version: 1,
          },
        );
        CAIRO_1_NO_VALIDATE_ACCOUNT.value += 1;
        await jumpBlocks(context, 1);

        const contractClassActual = await providerRPC.getClass(
          classHash,
          "latest",
        );
        // TODO: (Apoorv) make these checks better once we to_rpc_contract_class is fixed #775 and #790
        expect(contractClassActual).to.have.property("entry_points_by_type");
        expect(contractClassActual).to.have.property("sierra_program");
        expect(contractClassActual).to.have.property("contract_class_version");
        expect(contractClassActual).to.have.property("abi");
        expect(res.class_hash).to.be.eq(classHash);

        const deployRes = await account.deploy(
          {
            classHash,
            constructorCalldata: [
              1, // Token Name
              1, // Token Symbol
              1, // Token Decimals
              "0xffffffffffffffffffffffffffffffff", // Initial Supply
              "0xffffffffffffffffffffffffffffffff", // Initial Supply Cont { since u256 }
              CAIRO_1_ACCOUNT_CONTRACT, // Recipient
            ],
          },
          {
            nonce: CAIRO_1_NO_VALIDATE_ACCOUNT.value,
            version: 1,
            maxFee: "12345678",
          },
        );
        CAIRO_1_NO_VALIDATE_ACCOUNT.value += 1;
        await jumpBlocks(context, 1);
        //  hex(get_storage_var_address("balances", 0x4))
        const balance = await providerRPC.getStorageAt(
          deployRes.contract_address[0],
          "0x617243ac31335377b9d26d1a6b02f47b419ad593e1ae67660dd27ec77635906",
          "latest",
        );
        expect(toHex(balance)).to.be.equal(
          "0xffffffffffffffffffffffffffffffff",
        );

        await account.execute(
          [
            {
              contractAddress: deployRes.contract_address[0],
              entrypoint: "transfer",
              calldata: [
                1, // recipient
                "0xffffffffffffffffffffffffffffffff", // amount low
                0, // amount high
              ],
            },
          ],
          undefined,
          {
            nonce: CAIRO_1_NO_VALIDATE_ACCOUNT.value,
            version: 1,
            maxFee: "12345678",
          },
        );
        await jumpBlocks(context, 1);

        const balanceSender = await providerRPC.getStorageAt(
          deployRes.contract_address[0],
          //  hex(get_storage_var_address("balances", 0x4))
          "0x617243ac31335377b9d26d1a6b02f47b419ad593e1ae67660dd27ec77635906",
          "latest",
        );
        expect(toHex(balanceSender)).to.be.equal("0x0");
        const balanceRecipient = await providerRPC.getStorageAt(
          deployRes.contract_address[0],
          // hex(get_storage_var_address("balances", 0x1))
          "0x753d37842b9cfa00ee311ab2564951681d89ee4d5596e84e74030de35018c8a",
          "latest",
        );
        expect(toHex(balanceRecipient)).to.be.equal(
          "0xffffffffffffffffffffffffffffffff",
        );
      });
    });
  },
  { runNewNode: true },
);
