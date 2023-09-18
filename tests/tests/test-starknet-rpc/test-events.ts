import "@keep-starknet-strange/madara-api-augment";
import { expect } from "chai";
import { LibraryError, RpcProvider, validateAndParseAddress } from "starknet";
import { describeDevMadara } from "../../util/setup-dev-tests";
import { cleanHex, rpcTransfer, starknetKeccak, toHex } from "../../util/utils";
import {
  ARGENT_CONTRACT_ADDRESS,
  FEE_TOKEN_ADDRESS,
  MINT_AMOUNT,
  SEQUENCER_ADDRESS,
} from "../constants";
import { InvokeTransaction } from "./types";

// keep "let" over "const" as the nonce is passed by reference
// to abstract the increment
// eslint-disable-next-line prefer-const
let ARGENT_CONTRACT_NONCE = { value: 0 };

describeDevMadara("Starknet RPC - Events Test", (context) => {
  let providerRPC: RpcProvider;

  before(async function () {
    providerRPC = new RpcProvider({
      nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
      retries: 3,
    }); // substrate node
  });

  describe("getEvents", () => {
    it("should fail on invalid continuation token", async function () {
      const filter = {
        from_block: { block_number: 0 },
        to_block: { block_number: 1 },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 1,
        continuation_token: "0xabdel",
        keys: [[]],
      };

      let events = providerRPC.getEvents(filter);
      await expect(events)
        .to.eventually.be.rejectedWith(
          "33: The supplied continuation token is invalid or unknown",
        )
        .and.be.an.instanceOf(LibraryError);

      // Send transactions
      const transactions = [];
      for (let i = 0; i < 5; i++) {
        transactions.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT,
          ),
        );
      }
      await context.createBlock(transactions);
      const block = await providerRPC.getBlockHashAndNumber();
      let filter2 = {
        from_block: { block_number: block.block_number },
        to_block: { block_number: block.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 1,
        continuation_token: "0,100,1",
        keys: [[]],
      };

      events = providerRPC.getEvents(filter2);
      await expect(events)
        .to.eventually.be.rejectedWith(
          "33: The supplied continuation token is invalid or unknown",
        )
        .and.be.an.instanceOf(LibraryError);

      filter2 = {
        from_block: { block_number: block.block_number },
        to_block: { block_number: block.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 1,
        continuation_token: "0,0,100",
        keys: [[]],
      };

      events = providerRPC.getEvents(filter2);
      await expect(events)
        .to.eventually.be.rejectedWith(
          "33: The supplied continuation token is invalid or unknown",
        )
        .and.be.an.instanceOf(LibraryError);
    });

    it("should fail on chunk size too big", async function () {
      const filter = {
        from_block: { block_number: 0 },
        to_block: { block_number: 1 },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 1001,
        keys: [[]],
      };

      const events = providerRPC.getEvents(filter);
      await expect(events)
        .to.eventually.be.rejectedWith("31: Requested page size is too big")
        .and.be.an.instanceOf(LibraryError);
    });

    it("should fail on keys too big", async function () {
      const filter = {
        from_block: { block_number: 0 },
        to_block: { block_number: 1 },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 1,
        keys: Array(101).fill(["0x0"]),
      };

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const events = providerRPC.getEvents(filter);
      await expect(events)
        .to.eventually.be.rejectedWith("34: Too many keys provided in a filter")
        .and.be.an.instanceOf(LibraryError);
    });

    it("returns expected events on correct filter", async function () {
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
      const block_hash_and_number = await providerRPC.getBlockHashAndNumber();
      const filter = {
        from_block: "latest",
        to_block: "latest",
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 10,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const events = await providerRPC.getEvents(filter);

      expect(events.events.length).to.be.equal(2);
      expect(events.continuation_token).to.be.null;
      for (const event of events.events) {
        expect(validateAndParseAddress(event.from_address)).to.be.equal(
          FEE_TOKEN_ADDRESS,
        );
        expect(event.transaction_hash).to.be.equal(tx.transaction_hash);
      }
      // check transfer event
      const transfer_event = events.events[0];
      expect(transfer_event).to.deep.equal({
        transaction_hash: tx.transaction_hash,
        block_hash: block_hash_and_number.block_hash,
        block_number: block_hash_and_number.block_number,
        from_address: cleanHex(FEE_TOKEN_ADDRESS),
        keys: [toHex(starknetKeccak("Transfer"))],
        data: [
          ARGENT_CONTRACT_ADDRESS,
          ARGENT_CONTRACT_ADDRESS,
          MINT_AMOUNT,
          "0x0",
        ].map(cleanHex),
      });
      // check fee transfer event
      const fee_event = events.events[1];
      expect(fee_event).to.deep.equal({
        transaction_hash: tx.transaction_hash,
        block_hash: block_hash_and_number.block_hash,
        block_number: block_hash_and_number.block_number,
        from_address: cleanHex(FEE_TOKEN_ADDRESS),
        keys: [toHex(starknetKeccak("Transfer"))],
        data: [
          ARGENT_CONTRACT_ADDRESS,
          SEQUENCER_ADDRESS,
          "0x1705c", // current fee perceived for the transfer
          "0x0",
        ].map(cleanHex),
      });
    });

    it("returns expected events on correct filter two blocks", async function () {
      // Send transactions
      const transactions = [];
      for (let i = 0; i < 5; i++) {
        transactions.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT,
          ),
        );
      }
      await context.createBlock(transactions);
      const firstBlockCreated = await providerRPC.getBlockHashAndNumber();
      // Second block
      const transactions2 = [];
      for (let i = 0; i < 5; i++) {
        transactions2.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT,
          ),
        );
      }
      await context.createBlock(transactions2);
      const secondBlockCreated = await providerRPC.getBlockHashAndNumber();

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const filter = {
        from_block: { block_number: firstBlockCreated.block_number },
        to_block: { block_number: secondBlockCreated.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 100,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const events = await providerRPC.getEvents(filter);

      expect(events.events.length).to.be.equal(20);
      expect(events.continuation_token).to.be.null;
      for (let i = 0; i < 2; i++) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const tx: InvokeTransaction =
          await providerRPC.getTransactionByBlockIdAndIndex(
            firstBlockCreated.block_hash,
            i,
          );
        expect(
          validateAndParseAddress(events.events[2 * i].from_address),
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events.events[2 * i].transaction_hash).to.be.equal(
          tx.transaction_hash,
        );
        expect(
          validateAndParseAddress(events.events[2 * i + 1].from_address),
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events.events[2 * i + 1].transaction_hash).to.be.equal(
          tx.transaction_hash,
        );
      }
      for (let i = 0; i < 2; i++) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const tx_second_block: InvokeTransaction =
          await providerRPC.getTransactionByBlockIdAndIndex(
            secondBlockCreated.block_hash,
            i,
          );
        expect(
          validateAndParseAddress(events.events[10 + 2 * i].from_address),
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events.events[10 + 2 * i].transaction_hash).to.be.equal(
          tx_second_block.transaction_hash,
        );
        expect(
          validateAndParseAddress(events.events[10 + 2 * i + 1].from_address),
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events.events[10 + 2 * i + 1].transaction_hash).to.be.equal(
          tx_second_block.transaction_hash,
        );
      }
    });

    it("returns expected events on correct filter two blocks pagination", async function () {
      // Send transactions
      const transactions = [];
      for (let i = 0; i < 5; i++) {
        transactions.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT,
          ),
        );
      }
      await context.createBlock(transactions);
      const firstBlockCreated = await providerRPC.getBlockHashAndNumber();
      // Second block
      const transactions2 = [];
      for (let i = 0; i < 5; i++) {
        transactions2.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT,
          ),
        );
      }
      await context.createBlock(transactions2);
      const secondBlockCreated = await providerRPC.getBlockHashAndNumber();

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      let filter = {
        from_block: { block_number: firstBlockCreated.block_number },
        to_block: { block_number: secondBlockCreated.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 7,
        continuation_token: null,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      let { events, continuation_token } = await providerRPC.getEvents(filter);

      expect(events.length).to.be.equal(7);
      // Transaction receipt events ordered as follows:
      // 0 FEE_TOKEN :: Transfer <-- rpc filter stops here
      // 1 ARGENT_ACCOUNT :: Execute
      // 2 FEE_TOKEN :: Transfer (fee charge)
      // 3 + 3 + 3 + 1 = a (visited events)
      // 2 + 2 + 2 + 1 = 7 (filtered events == chunk size)
      expect(continuation_token).to.be.equal("0,a");

      for (let i = 0; i < 3; i++) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const tx: InvokeTransaction =
          await providerRPC.getTransactionByBlockIdAndIndex(
            firstBlockCreated.block_hash,
            i,
          );
        expect(validateAndParseAddress(events[2 * i].from_address)).to.be.equal(
          FEE_TOKEN_ADDRESS,
        );
        expect(events[2 * i].transaction_hash).to.be.equal(tx.transaction_hash);
        expect(
          validateAndParseAddress(events[2 * i + 1].from_address),
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events[2 * i + 1].transaction_hash).to.be.equal(
          tx.transaction_hash,
        );
      }
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const tx3: InvokeTransaction =
        await providerRPC.getTransactionByBlockIdAndIndex(
          firstBlockCreated.block_hash,
          3,
        );
      expect(validateAndParseAddress(events[6].from_address)).to.be.equal(
        FEE_TOKEN_ADDRESS,
      );
      expect(events[6].transaction_hash).to.be.equal(tx3.transaction_hash);

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      filter = {
        from_block: { block_number: firstBlockCreated.block_number },
        to_block: { block_number: secondBlockCreated.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 7,
        continuation_token: continuation_token,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      ({ events, continuation_token } = await providerRPC.getEvents(filter));

      expect(events.length).to.be.equal(7);
      expect(continuation_token).to.be.equal("1,6");

      expect(validateAndParseAddress(events[0].from_address)).to.be.equal(
        FEE_TOKEN_ADDRESS,
      );
      expect(events[0].transaction_hash).to.be.equal(tx3.transaction_hash);

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const tx4: InvokeTransaction =
        await providerRPC.getTransactionByBlockIdAndIndex(
          firstBlockCreated.block_hash,
          4,
        );
      expect(validateAndParseAddress(events[1].from_address)).to.be.equal(
        FEE_TOKEN_ADDRESS,
      );
      expect(events[1].transaction_hash).to.be.equal(tx4.transaction_hash);
      expect(validateAndParseAddress(events[2].from_address)).to.be.equal(
        FEE_TOKEN_ADDRESS,
      );
      expect(events[2].transaction_hash).to.be.equal(tx4.transaction_hash);

      for (let i = 0; i < 2; i++) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const tx: InvokeTransaction =
          await providerRPC.getTransactionByBlockIdAndIndex(
            secondBlockCreated.block_hash,
            i,
          );
        expect(
          validateAndParseAddress(events[2 * i + 3].from_address),
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events[2 * i + 3].transaction_hash).to.be.equal(
          tx.transaction_hash,
        );
        expect(
          validateAndParseAddress(events[2 * i + 4].from_address),
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events[2 * i + 4].transaction_hash).to.be.equal(
          tx.transaction_hash,
        );
      }

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      filter = {
        from_block: { block_number: firstBlockCreated.block_number },
        to_block: { block_number: secondBlockCreated.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 7,
        continuation_token: continuation_token,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      ({ events, continuation_token } = await providerRPC.getEvents(filter));

      expect(events.length).to.be.equal(6);
      expect(continuation_token).to.be.null;

      for (let i = 2; i < 5; i++) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const tx: InvokeTransaction =
          await providerRPC.getTransactionByBlockIdAndIndex(
            secondBlockCreated.block_hash,
            i,
          );
        expect(
          validateAndParseAddress(events[2 * i - 4].from_address),
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events[2 * i - 4].transaction_hash).to.be.equal(
          tx.transaction_hash,
        );
        expect(
          validateAndParseAddress(events[2 * i - 3].from_address),
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events[2 * i - 3].transaction_hash).to.be.equal(
          tx.transaction_hash,
        );
      }
    });

    it("returns expected events on correct filter many blocks pagination", async function () {
      // Send transactions
      const transactions = [];
      for (let i = 0; i < 5; i++) {
        transactions.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT,
          ),
        );
      }
      await context.createBlock(transactions);
      const firstBlockCreated = await providerRPC.getBlockHashAndNumber();

      // 3 blocks without transactions
      const empty_transactions = [];
      await context.createBlock(empty_transactions);
      await context.createBlock(empty_transactions);
      await context.createBlock(empty_transactions);
      // Second block
      const transactions2 = [];
      for (let i = 0; i < 5; i++) {
        transactions2.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT,
          ),
        );
      }
      await context.createBlock(transactions2);
      const fifthBlockCreated = await providerRPC.getBlockHashAndNumber();

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      let filter = {
        from_block: { block_number: firstBlockCreated.block_number },
        to_block: { block_number: fifthBlockCreated.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 10,
        continuation_token: null,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      let { events, continuation_token } = await providerRPC.getEvents(filter);

      expect(events.length).to.be.equal(10);
      expect(continuation_token).to.be.equal("0,f");

      for (let i = 0; i < 5; i++) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const tx: InvokeTransaction =
          await providerRPC.getTransactionByBlockIdAndIndex(
            firstBlockCreated.block_hash,
            i,
          );
        expect(validateAndParseAddress(events[2 * i].from_address)).to.be.equal(
          FEE_TOKEN_ADDRESS,
        );
        expect(events[2 * i].transaction_hash).to.be.equal(tx.transaction_hash);
        expect(
          validateAndParseAddress(events[2 * i + 1].from_address),
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events[2 * i + 1].transaction_hash).to.be.equal(
          tx.transaction_hash,
        );
      }

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      filter = {
        from_block: { block_number: firstBlockCreated.block_number },
        to_block: { block_number: fifthBlockCreated.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 10,
        continuation_token: continuation_token,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      ({ events, continuation_token } = await providerRPC.getEvents(filter));

      expect(events.length).to.be.equal(10);
      expect(continuation_token).to.be.null;

      for (let i = 0; i < 5; i++) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const tx: InvokeTransaction =
          await providerRPC.getTransactionByBlockIdAndIndex(
            fifthBlockCreated.block_hash,
            i,
          );
        expect(validateAndParseAddress(events[2 * i].from_address)).to.be.equal(
          FEE_TOKEN_ADDRESS,
        );
        expect(events[2 * i].transaction_hash).to.be.equal(tx.transaction_hash);
        expect(
          validateAndParseAddress(events[2 * i + 1].from_address),
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events[2 * i + 1].transaction_hash).to.be.equal(
          tx.transaction_hash,
        );
      }
    });

    it("returns expected events on correct filter many empty blocks pagination", async function () {
      // Send transactions
      const transactions = [];
      for (let i = 0; i < 5; i++) {
        transactions.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT,
          ),
        );
      }
      await context.createBlock(transactions);
      const firstBlockCreated = await providerRPC.getBlockHashAndNumber();

      // 4 blocks without transactions
      const empty_transactions = [];
      await context.createBlock(empty_transactions);
      await context.createBlock(empty_transactions);
      await context.createBlock(empty_transactions);
      await context.createBlock(empty_transactions);

      const fifthBlockCreated = await providerRPC.getBlockHashAndNumber();

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      let filter = {
        from_block: { block_number: firstBlockCreated.block_number },
        to_block: { block_number: fifthBlockCreated.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 10,
        continuation_token: null,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      let { events, continuation_token } = await providerRPC.getEvents(filter);

      expect(events.length).to.be.equal(10);
      expect(continuation_token).to.be.equal("0,f");

      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      filter = {
        from_block: { block_number: firstBlockCreated.block_number },
        to_block: { block_number: fifthBlockCreated.block_number },
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 10,
        continuation_token: continuation_token,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      ({ events, continuation_token } = await providerRPC.getEvents(filter));

      expect(events.length).to.be.equal(0);
      expect(continuation_token).to.be.null;
    });

    it("returns expected events on correct filter with chunk size", async function () {
      // Send transactions
      const transactions = [];
      for (let i = 0; i < 5; i++) {
        transactions.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT,
          ),
        );
      }
      await context.createBlock(transactions);

      const filter = {
        from_block: "latest",
        to_block: "latest",
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 4,
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const events = await providerRPC.getEvents(filter);
      expect(events.events.length).to.be.equal(4);
      expect(events.continuation_token).to.be.equal("0,6");
      for (let i = 0; i < 2; i++) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const tx: InvokeTransaction =
          await providerRPC.getTransactionByBlockIdAndIndex("latest", i);
        expect(
          validateAndParseAddress(events.events[2 * i].from_address),
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events.events[2 * i].transaction_hash).to.be.equal(
          tx.transaction_hash,
        );
        expect(
          validateAndParseAddress(events.events[2 * i + 1].from_address),
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events.events[2 * i + 1].transaction_hash).to.be.equal(
          tx.transaction_hash,
        );
      }
    });

    it("returns expected events on correct filter with continuation token", async function () {
      // Send transactions
      const transactions = [];
      for (let i = 0; i < 5; i++) {
        transactions.push(
          rpcTransfer(
            providerRPC,
            ARGENT_CONTRACT_NONCE,
            ARGENT_CONTRACT_ADDRESS,
            MINT_AMOUNT,
          ),
        );
      }
      await context.createBlock(transactions);

      const skip = 3;
      const filter = {
        from_block: "latest",
        to_block: "latest",
        address: FEE_TOKEN_ADDRESS,
        chunk_size: 4,
        continuation_token: `0,${skip * 3}`, // 3 events per transaction
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const events = await providerRPC.getEvents(filter);
      expect(events.events.length).to.be.equal(4);
      expect(events.continuation_token).to.be.null;
      for (let i = 0; i < 2; i++) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        const tx: InvokeTransaction =
          await providerRPC.getTransactionByBlockIdAndIndex("latest", skip + i);
        expect(
          validateAndParseAddress(events.events[2 * i].from_address),
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events.events[2 * i].transaction_hash).to.be.equal(
          tx.transaction_hash,
        );
        expect(
          validateAndParseAddress(events.events[2 * i + 1].from_address),
        ).to.be.equal(FEE_TOKEN_ADDRESS);
        expect(events.events[2 * i + 1].transaction_hash).to.be.equal(
          tx.transaction_hash,
        );
      }
    });

    it("returns expected events on correct filter with keys", async function () {
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
      const block_hash_and_number = await providerRPC.getBlockHashAndNumber();
      const filter = {
        from_block: "latest",
        to_block: "latest",
        chunk_size: 1,
        keys: [[toHex(starknetKeccak("transaction_executed"))]],
      };
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      const events = await providerRPC.getEvents(filter);
      expect(events.events.length).to.be.equal(1);
      // Transaction receipt events ordered as follows:
      // 0 FEE_TOKEN :: Transfer
      // 1 ARGENT_ACCOUNT :: Execute <-- rpc filter stops here
      // 2 FEE_TOKEN :: Transfer (fee charge)
      expect(events.continuation_token).to.be.equal("0,2");
      expect(events.events[0]).to.deep.equal({
        transaction_hash: tx.transaction_hash,
        block_hash: block_hash_and_number.block_hash,
        block_number: block_hash_and_number.block_number,
        from_address: cleanHex(ARGENT_CONTRACT_ADDRESS),
        keys: [toHex(starknetKeccak("transaction_executed"))],
        data: [tx.transaction_hash, "0x2", "0x1", "0x1"].map(cleanHex),
      });
    });
  });
});
