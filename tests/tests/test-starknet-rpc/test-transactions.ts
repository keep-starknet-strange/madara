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
  CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH,
} from "../constants";
import { InvokeTransaction } from "./types";
import { numberToHex } from "@polkadot/util";

// keep "let" over "const" as the nonce is passed by reference
// to abstract the increment
// eslint-disable-next-line prefer-const
let ARGENT_CONTRACT_NONCE = { value: 0 };
const CAIRO_1_NO_VALIDATE_ACCOUNT = { value: 0 };

describeDevMadara(
  "Starknet RPC - Transactions Test",
  (context) => {
    let providerRPC: RpcProvider;

    before(async function () {
      providerRPC = new RpcProvider({
        nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
        retries: 3,
      }); // substrate node
    });

    describe("getTransactionByBlockIdAndIndex", async () => {
      it("should returns 1 transaction", async function () {
        // Send a transaction
        await context.createBlock(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT,
          ),
        );

        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const tx: InvokeTransaction =
          await providerRPC.getTransactionByBlockIdAndIndex("latest", 0);
        expect(tx).to.not.be.undefined;
        expect(tx.type).to.be.equal("INVOKE");
        expect(tx.sender_address).to.be.equal(toHex(ARGENT_CONTRACT_ADDRESS));
        expect(tx.calldata).to.deep.equal(
          [
            1,
            FEE_TOKEN_ADDRESS,
            hash.getSelectorFromName("transfer"),
            0,
            3,
            3,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT,
            0,
          ].map(toHex),
        );
      });

      it("should throws block not found error", async function () {
        const transaction = providerRPC.getTransactionByBlockIdAndIndex(
          "0x123",
          2,
        );
        await expect(transaction)
          .to.eventually.be.rejectedWith("24: Block not found")
          .and.be.an.instanceOf(LibraryError);
      });

      it("should throws invalid transaction index error", async function () {
        await context.createBlock(undefined, {
          parentHash: undefined,
          finalize: true,
        });
        const latestBlockCreated = await providerRPC.getBlockHashAndNumber();
        const transaction = providerRPC.getTransactionByBlockIdAndIndex(
          latestBlockCreated.block_hash,
          2,
        );
        await expect(transaction)
          .to.eventually.be.rejectedWith(
            "27: Invalid transaction index in a block",
          )
          .and.be.an.instanceOf(LibraryError);
      });
    });

    describe("addInvokeTransaction", async () => {
      it("should invoke successfully", async function () {
        const account = new Account(
          providerRPC,
          ARGENT_CONTRACT_ADDRESS,
          SIGNER_PRIVATE,
        );

        await account.execute(
          {
            contractAddress: FEE_TOKEN_ADDRESS,
            entrypoint: "transfer",
            calldata: ["0xdeadbeef", "0x123", "0x0"],
          },
          undefined,
          {
            nonce: ARGENT_CONTRACT_NONCE.value,
          },
        );
        ARGENT_CONTRACT_NONCE.value += 1;
        await jumpBlocks(context, 1);

        // ERC20_balances(0xdeadbeef).low = 0x4c761778f11aa10fc40190ff3127637fe00dc59bfa557bd4c8beb30a178f016
        const balance = await providerRPC.getStorageAt(
          FEE_TOKEN_ADDRESS,
          "0x04c761778f11aa10fc40190ff3127637fe00dc59bfa557bd4c8beb30a178f016",
          "latest",
        );
        expect(toHex(balance)).to.be.equal("0x123");
      });

      it("should deploy ERC20 via UDC", async function () {
        const account = new Account(
          providerRPC,
          ARGENT_CONTRACT_ADDRESS,
          SIGNER_PRIVATE,
        );

        const calldata = [
          numberToHex(1, 256), // Token Name
          numberToHex(1, 256), // Token Symbol
          numberToHex(18, 256), // Token Decimals
          numberToHex(42, 256), // Initial Supply
          "0x0000000000000000000000000000000000000000000000000000000000000000", // Initial Supply Cont { since u256 }
          "0xdeadbeef", // Recipient
        ];

        const deployedContractAddress = hash.calculateContractAddressFromHash(
          SALT,
          TOKEN_CLASS_HASH,
          calldata,
          0,
        );

        await account.execute(
          {
            contractAddress: UDC_CONTRACT_ADDRESS,
            entrypoint: "deployContract",
            calldata: [TOKEN_CLASS_HASH, SALT, "0x0", "0x6", ...calldata],
          },
          undefined,
          {
            nonce: ARGENT_CONTRACT_NONCE.value,
          },
        );
        ARGENT_CONTRACT_NONCE.value += 1;
        await jumpBlocks(context, 1);

        // ERC20_balances(0xdeadbeef).low = 0x4c761778f11aa10fc40190ff3127637fe00dc59bfa557bd4c8beb30a178f016
        const balance = await providerRPC.getStorageAt(
          deployedContractAddress,
          "0x04c761778f11aa10fc40190ff3127637fe00dc59bfa557bd4c8beb30a178f016",
          "latest",
        );
        expect(toHex(balance)).to.be.equal("0x2a");
      });

      it("should fail on invalid nonce", async function () {
        const invalid_nonce = { value: ARGENT_CONTRACT_NONCE.value + 1 };

        // ERC20_balances(0x1111).low = 0x72943352085ed3fbe3b8ff53a6aef9da8d893ccdab99bd5223d765f1a22735f
        let balance = await providerRPC.getStorageAt(
          FEE_TOKEN_ADDRESS,
          "0x72943352085ed3fbe3b8ff53a6aef9da8d893ccdab99bd5223d765f1a22735f",
          "latest",
        );

        expect(toHex(balance)).to.be.equal("0x0");

        await rpcTransfer(
          providerRPC,
          invalid_nonce,
          TEST_CONTRACT_ADDRESS,
          MINT_AMOUNT,
        ),
          await jumpBlocks(context, 1);

        // ERC20_balances(0x1111).low = 0x72943352085ed3fbe3b8ff53a6aef9da8d893ccdab99bd5223d765f1a22735f
        balance = await providerRPC.getStorageAt(
          FEE_TOKEN_ADDRESS,
          "0x72943352085ed3fbe3b8ff53a6aef9da8d893ccdab99bd5223d765f1a22735f",
          "latest",
        );
        expect(toHex(balance)).to.be.equal("0x0");

        // This transaction is send in order to clear the pending transactions (sending a correct nonce triggers the pending
        // transaction in the pool)
        await rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          TEST_CONTRACT_ADDRESS,
          MINT_AMOUNT,
        ),
          await jumpBlocks(context, 1);

        // ERC20_balances(0x1111).low = 0x72943352085ed3fbe3b8ff53a6aef9da8d893ccdab99bd5223d765f1a22735f
        balance = await providerRPC.getStorageAt(
          FEE_TOKEN_ADDRESS,
          "0x72943352085ed3fbe3b8ff53a6aef9da8d893ccdab99bd5223d765f1a22735f",
          "latest",
        );
        // The balance should be == MINT_AMOUNT * 2
        expect(toHex(balance)).to.be.equal("0x2");
        // Increment the nonce since we sent one transaction which wasn't accounted for
        ARGENT_CONTRACT_NONCE.value += 1;
      });
    });

    describe("addDeployAccountTransaction", async () => {
      it("should deploy successfully", async function () {
        // Compute contract address
        const selector = hash.getSelectorFromName("initialize");
        const calldata = [
          ARGENT_ACCOUNT_CLASS_HASH,
          selector,
          2,
          SIGNER_PUBLIC,
          0,
        ];

        const deployedContractAddress = hash.calculateContractAddressFromHash(
          SALT,
          ARGENT_PROXY_CLASS_HASH,
          calldata,
          0,
        );
        // fund address
        await rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          deployedContractAddress,
          DEPLOY_ACCOUNT_COST,
        );
        await jumpBlocks(context, 1);

        const invocationDetails = {
          nonce: "0x0",
          maxFee: "0x1111111111111111111111",
          version: "0x1",
        };

        const signer = new Signer(SIGNER_PRIVATE);
        const signature = await signer.signDeployAccountTransaction({
          classHash: ARGENT_PROXY_CLASS_HASH,
          contractAddress: deployedContractAddress,
          constructorCalldata: calldata,
          addressSalt: SALT,
          maxFee: invocationDetails.maxFee,
          version: invocationDetails.version,
          chainId: constants.StarknetChainId.SN_GOERLI,
          nonce: invocationDetails.nonce,
        });

        // Deploy account contract
        const txDeployAccount = {
          signature: signature, // signature
          contractAddress: deployedContractAddress, // address of the sender contract
          addressSalt: SALT, // contract address salt
          classHash: ARGENT_PROXY_CLASS_HASH, // class hash of the contract
          constructorCalldata: calldata,
        };

        await providerRPC.deployAccountContract(
          txDeployAccount,
          invocationDetails,
        );
        await createAndFinalizeBlock(context.polkadotApi);

        const accountContractClassHash = await providerRPC.getClassHashAt(
          deployedContractAddress,
        );

        expect(validateAndParseAddress(accountContractClassHash)).to.be.equal(
          ARGENT_PROXY_CLASS_HASH,
        );
      });
    });

    describe("estimateFee", async () => {
      it("should estimate fee", async function () {
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
        };

        const invocation: AccountInvocationItem = {
          type: "INVOKE_FUNCTION",
          ...tx,
          ...txDetails,
        };

        const fee_estimates = await providerRPC.getEstimateFeeBulk(
          [invocation],
          {
            blockIdentifier: "latest",
          },
        );

        expect(fee_estimates[0].overall_fee > 0n).to.be.equal(true);
        expect(fee_estimates[0].gas_consumed > 0n).to.be.equal(true);
      });

      it("should fail estimate fee if version is 1", async function () {
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
          version: 1,
        };

        const invocation: AccountInvocationItem = {
          type: "INVOKE_FUNCTION",
          ...tx,
          ...txDetails,
        };

        await expect(
          providerRPC.getEstimateFeeBulk([invocation], {
            blockIdentifier: "latest",
          }),
        )
          .to.eventually.be.rejectedWith(
            "61: The transaction version is not supported",
          )
          .and.be.an.instanceOf(LibraryError);
      });

      it("should raise if contract does not exist", async function () {
        const tx = {
          contractAddress: ACCOUNT_CONTRACT,
          calldata: [
            "0x000000000000000000000000000000000000000000000000000000000000DEAD",
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

        const fee_estimates = providerRPC.getEstimateFeeBulk([invocation], {
          blockIdentifier: "latest",
        });

        //    TODO: once starknet-js supports estimateFee using array
        //   expect(estimate).to.eventually.be.rejectedWith(
        //     "invalid type: map, expected variant identifier"
        //   );

        expect(fee_estimates)
          .to.eventually.be.rejectedWith("40: Contract error")
          .and.be.an.instanceOf(LibraryError);
      });

      it("should estimate fees for multiple invocations", async function () {
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
        };

        const invocation: AccountInvocationItem = {
          type: "INVOKE_FUNCTION",
          ...tx,
          ...txDetails,
        };

        const fee_estimates = await providerRPC.getEstimateFeeBulk(
          [invocation, invocation],
          {
            blockIdentifier: "latest",
          },
        );

        expect(fee_estimates[0].overall_fee > 0n).to.be.equal(true);
        expect(fee_estimates[0].gas_consumed > 0n).to.be.equal(true);
        expect(fee_estimates[1].overall_fee > 0n).to.be.equal(true);
        expect(fee_estimates[1].gas_consumed > 0n).to.be.equal(true);
      });

      it("should return empty array if no invocations", async function () {
        const fee_estimates = await providerRPC.getEstimateFeeBulk([], {
          blockIdentifier: "latest",
        });

        expect(fee_estimates.length == 0).to.be.equal(true);
      });

      it("should be possible for an account to estimateInvokeFee", async function () {
        const account = new Account(
          providerRPC,
          ARGENT_CONTRACT_ADDRESS,
          SIGNER_PRIVATE,
        );

        const { suggestedMaxFee } = await account.estimateInvokeFee({
          contractAddress: TEST_CONTRACT_ADDRESS,
          entrypoint: "test_storage_var",
          calldata: [],
        });
        expect(suggestedMaxFee > 0n).to.be.equal(true);
      });

      it("should be possible for an account to estimateDeclareFee", async function () {
        const account = new Account(
          providerRPC,
          ARGENT_CONTRACT_ADDRESS,
          SIGNER_PRIVATE,
        );

        const { suggestedMaxFee } = await account.estimateDeclareFee({
          contract: ERC20_CONTRACT,
        });

        expect(suggestedMaxFee > 0n).to.be.equal(true);
      });

      it("should be possible for an account to estimateAccountDeployFee", async function () {
        const account = new Account(
          providerRPC,
          ARGENT_CONTRACT_ADDRESS,
          SIGNER_PRIVATE,
        );

        const { suggestedMaxFee } = await account.estimateAccountDeployFee({
          classHash: CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH,
          constructorCalldata: ["0x123"],
          addressSalt: SALT,
        });

        expect(suggestedMaxFee > 0n).to.be.equal(true);
      });
    });

    describe("addDeclareTransaction", async () => {
      it("should set class at given class hash (legacy)", async function () {
        const account = new Account(
          providerRPC,
          ARGENT_CONTRACT_ADDRESS,
          SIGNER_PRIVATE,
        );
        // computed via: starkli class-hash ./cairo-contracts/build/ERC20.json
        // the above command should be used at project root
        const classHash =
          "0x372ee6669dc86563007245ed7343d5180b96221ce28f44408cff2898038dbd4";
        const res = await account.declare(
          {
            classHash: classHash,
            contract: ERC20_CONTRACT,
          },
          { nonce: ARGENT_CONTRACT_NONCE.value },
        );
        ARGENT_CONTRACT_NONCE.value += 1;
        await jumpBlocks(context, 1);

        const contractClassActual = await providerRPC.getClass(
          classHash,
          "latest",
        );
        expect(contractClassActual.entry_points_by_type).to.deep.equal(
          ERC20_CONTRACT.entry_points_by_type,
        );
        // TODO compare the program as well
        // expect(contractClassActual.program).to.be.equal(
        //   stark.compressProgram(ERC20_CONTRACT.program)
        // );
        expect(res.class_hash).to.be.eq(classHash);
      });

      it("should set class at given class hash and deploy a new contract (cairo 1)", async function () {
        const account = new Account(
          providerRPC,
          CAIRO_1_ACCOUNT_CONTRACT,
          "0x123", // it's the no validate account
        );
        // computed via: starknetjs 5.14.1
        const classHash =
          "0x9cf5ef6166edaa87767d05bbfd54ad02fd110028597343a200e82949ce05cf";
        const res = await account.declare(
          {
            casm: TEST_CAIRO_1_CASM,
            contract: TEST_CAIRO_1_SIERRA,
          },
          {
            nonce: CAIRO_1_NO_VALIDATE_ACCOUNT.value,
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
      });

      it("should fail to declare duplicate class", async function () {
        const account = new Account(
          providerRPC,
          ARGENT_CONTRACT_ADDRESS,
          SIGNER_PRIVATE,
        );

        // computed via: starkli class-hash ./cairo-contracts/build/ERC20.json
        // the above command should be used at project root
        const classHash =
          "0x372ee6669dc86563007245ed7343d5180b96221ce28f44408cff2898038dbd4";

        await expect(
          account.declare(
            {
              classHash: classHash,
              contract: ERC20_CONTRACT,
            },
            {
              nonce: ARGENT_CONTRACT_NONCE.value,
            },
          ),
        ).to.be.rejectedWith("51: Class already declared");
      });
    });

    describe("pendingTransactions", async () => {
      it("should return all the starknet invoke transactions", async function () {
        // create a invoke transaction
        await rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT,
        );

        const txs = await providerRPC.getPendingTransactions();

        expect(txs.length).equals(1);

        expect(txs[0]).to.include({ type: "INVOKE" });
        expect(txs[0]).that.includes.all.keys([
          "transaction_hash",
          "max_fee",
          "version",
          "signature",
          "nonce",
          "type",
          "sender_address",
          "calldata",
        ]);

        await jumpBlocks(context, 10);
      });

      it("should return all starknet declare transactions", async function () {
        const account = new Account(
          providerRPC,
          ARGENT_CONTRACT_ADDRESS,
          SIGNER_PRIVATE,
        );

        // computed via: starkli class-hash ./cairo-contracts/build/ERC721.json
        // the above command should be used at project root
        const classHash =
          "0x077cc28ed3c661419fda16bf120fb81f1f8f28617f5543b05a86d63b0926bbf4";
        await account.declare(
          {
            classHash: classHash,
            contract: ERC721_CONTRACT,
          },
          { nonce: ARGENT_CONTRACT_NONCE.value },
        );

        const txs = await providerRPC.getPendingTransactions();

        expect(txs.length).equals(1);

        expect(txs[0]).to.include({ type: "DECLARE" });
        expect(txs[0]).that.includes.all.keys([
          "sender_address",
          "class_hash",
          "max_fee",
          "nonce",
          "signature",
          "transaction_hash",
          "type",
          "version",
        ]);

        await jumpBlocks(context, 10);
      });

      it("should return all starknet deploy_account transactions", async function () {
        // create a deploy_contract transaction
        const selector = hash.getSelectorFromName("initialize");
        const calldata = [
          ARGENT_ACCOUNT_CLASS_HASH,
          selector,
          2,
          SIGNER_PUBLIC,
          0,
        ];

        const deployedContractAddress = hash.calculateContractAddressFromHash(
          SALT,
          ARGENT_PROXY_CLASS_HASH,
          calldata,
          0,
        );

        const invocationDetails = {
          nonce: "0x0",
          maxFee: "0x1111111111111111111111",
          version: "0x1",
        };

        const signer = new Signer(SIGNER_PRIVATE);
        const signature = await signer.signDeployAccountTransaction({
          classHash: ARGENT_PROXY_CLASS_HASH,
          contractAddress: deployedContractAddress,
          constructorCalldata: calldata,
          addressSalt: SALT,
          maxFee: invocationDetails.maxFee,
          version: invocationDetails.version,
          chainId: constants.StarknetChainId.SN_GOERLI,
          nonce: invocationDetails.nonce,
        });

        // Deploy account contract
        const txDeployAccount = {
          signature: signature, // signature
          contractAddress: deployedContractAddress, // address of the sender contract
          addressSalt: SALT, // contract address salt
          classHash: ARGENT_PROXY_CLASS_HASH, // class hash of the contract
          constructorCalldata: calldata,
        };

        await providerRPC.deployAccountContract(
          txDeployAccount,
          invocationDetails,
        );

        const txs = await providerRPC.getPendingTransactions();

        expect(txs.length).equals(1);
        expect(txs[0]).to.include({ type: "DEPLOY_ACCOUNT" });
        expect(txs[0]).that.includes.all.keys([
          "class_hash",
          "constructor_calldata",
          "contract_address_salt",
          "max_fee",
          "nonce",
          "signature",
          "transaction_hash",
          "type",
          "version",
        ]);

        await jumpBlocks(context, 10);
      });

      it("should return transactions from the ready and future queues", async function () {
        const transactionNonceOffset = 1_000;
        // ready transaction
        await rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT,
        );
        // future transaction
        // add a high number to the nonce to make sure the transaction is added to the future queue
        await rpcTransfer(
          providerRPC,
          { value: ARGENT_CONTRACT_NONCE.value + transactionNonceOffset },
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT,
        );

        // the pendingExtrinsics endpoint returns only the ready transactions
        // (https://github.com/paritytech/substrate/blob/master/client/rpc/src/author/mod.rs#L153)
        const readyExtrinsics =
          await context.polkadotApi.rpc.author.pendingExtrinsics();
        const readyTxs = readyExtrinsics.map((pending) => {
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          const obj: any = pending.toHuman();
          return {
            type: obj.method.method.toUpperCase(),
            nonce: toHex(obj.method.args.transaction.nonce),
          };
        });

        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const txs: InvokeTransaction[] =
          await providerRPC.getPendingTransactions();

        expect(readyExtrinsics.length).to.be.equal(1);
        expect(txs.length).to.be.equal(2);

        expect(readyTxs[0]).to.include({
          type: "INVOKE",
          nonce: toHex(ARGENT_CONTRACT_NONCE.value - 1),
        });
        expect(txs[0]).to.include({
          type: "INVOKE",
          nonce: toHex(ARGENT_CONTRACT_NONCE.value - 1),
        });
        expect(txs[1]).to.include({
          type: "INVOKE",
          nonce: toHex(ARGENT_CONTRACT_NONCE.value + transactionNonceOffset),
        });

        await jumpBlocks(context, 10);
      });
    });

    describe("getTransactionByHash", () => {
      it("should return a transaction", async function () {
        await createAndFinalizeBlock(context.polkadotApi);

        // Send a transaction
        const b = await context.createBlock(
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

        const r = await providerRPC.getTransactionByHash(b.result.hash);
        expect(r).to.not.be.undefined;
      });

      it("should return transaction hash not found", async function () {
        // Send a transaction
        await context.createBlock(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT,
          ),
        );

        const transaction = providerRPC.getTransactionByHash("0x1234");
        await expect(transaction)
          .to.eventually.be.rejectedWith("25: Transaction hash not found")
          .and.be.an.instanceOf(LibraryError);
      });

      it("should return transaction hash not found when a transaction is in the pool", async function () {
        await createAndFinalizeBlock(context.polkadotApi);

        // create a invoke transaction
        const b = await rpcTransfer(
          providerRPC,
          ARGENT_CONTRACT_NONCE,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT,
        );

        const transaction = providerRPC.getTransactionByHash(
          b.transaction_hash,
        );
        await expect(transaction)
          .to.eventually.be.rejectedWith("25: Transaction hash not found")
          .and.be.an.instanceOf(LibraryError);
      });
    });

    describe("getTransactionReceipt", () => {
      it("should return a receipt", async function () {
        await createAndFinalizeBlock(context.polkadotApi);

        // Send a transaction
        const b = await context.createBlock(
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

        const block_hash_and_number = await providerRPC.getBlockHashAndNumber();
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const r: TransactionReceipt = await providerRPC.getTransactionReceipt(
          b.result.hash,
        );
        expect(r).to.not.be.undefined;
        expect(r.block_hash).to.be.equal(block_hash_and_number.block_hash);
        expect(r.block_number).to.be.equal(block_hash_and_number.block_number);
      });

      it("should return transaction hash not found", async function () {
        // Send a transaction
        await context.createBlock(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT,
          ),
        );

        const transaction = providerRPC.getTransactionReceipt("0x1234");
        await expect(transaction)
          .to.eventually.be.rejectedWith("25: Transaction hash not found")
          .and.be.an.instanceOf(LibraryError);
      });
    });
  },
  { runNewNode: true },
);
