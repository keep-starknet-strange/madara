import "@keep-starknet-strange/madara-api-augment";

import { expect } from "chai";

import { describeDevMadara } from "../../util/setup-dev-tests";
import { RpcProvider, validateAndParseAddress } from "starknet";

describeDevMadara("Starknet RPC", (context) => {
  let providerRPC: RpcProvider;

  before(async function () {
    this.timeout(100000);
    providerRPC = new RpcProvider({
      nodeUrl: `http://127.0.0.1:${context.rpcPort}/`,
      retries: 3,
    }); // substrate node
  });

  it("getBlockhashAndNumber", async function () {
    let block = await providerRPC.getBlockHashAndNumber();

    console.log(block);

    expect(block).to.not.be.undefined;
  });

  it("getBlockNumber", async function () {
    let blockNumber = await providerRPC.getBlockNumber();

    console.log(blockNumber);

    expect(blockNumber).to.not.be.undefined;
  });

  it("getBlockTransactionCount", async function () {
    let block = await providerRPC.getBlockHashAndNumber();

    let transactionCount = await providerRPC.getTransactionCount(
      block.block_hash
    );

    console.log(transactionCount);

    expect(transactionCount).to.not.be.undefined;
    expect(transactionCount).to.be.equal(0);
  });

  it("call", async function () {
    let block = await providerRPC.getBlockHashAndNumber();

    let block_hash = `0x${block.block_hash.slice(2).padStart(64, "0")}`;

    let call = await providerRPC.callContract(
      {
        contractAddress:
          "0x0000000000000000000000000000000000000000000000000000000000001111", // test contract
        entrypoint: "return_result",
        calldata: ["0x19"],
      },
      block_hash
    );

    expect(call.result).to.contain("0x19");
  });
});
