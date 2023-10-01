import { Keyring, type ApiPromise } from "@polkadot/api";
import { type ApiTypes, type SubmittableExtrinsic } from "@polkadot/api/types";
import { type EventRecord } from "@polkadot/types/interfaces";
import { type RegistryError } from "@polkadot/types/types";
import { type ChildProcess } from "child_process";

import { createAndFinalizeBlock } from "./block";
import { DEBUG_MODE, SPAWNING_TIME } from "./constants";
import {
  startMadaraDevNode,
  startMadaraForkedNode,
  type RuntimeChain,
} from "./dev-node";
import { providePolkadotApi } from "./providers";
import { extractError, type ExtrinsicCreation } from "./substrate-rpc";

import { type KeyringPair } from "@polkadot/keyring/types";
import debugFactory from "debug";
import { InvokeFunctionResponse } from "starknet";

import chaiAsPromised from "chai-as-promised";
import chai from "chai";
import deepEqualInAnyOrder from "deep-equal-in-any-order";
import process from "process";

const debug = debugFactory("test:setup");

export interface BlockCreation {
  parentHash?: string;
  finalize?: boolean;
}

export interface BlockCreationResponse<
  ApiType extends ApiTypes,
  Call extends
    | SubmittableExtrinsic<ApiType>
    | string
    | Array<SubmittableExtrinsic<ApiType> | string>,
> {
  block: {
    duration: number;
    hash: string;
  };
  result: Call extends Array<string | SubmittableExtrinsic<ApiType>>
    ? ExtrinsicCreation[]
    : ExtrinsicCreation;
}

export interface DevTestContext {
  alice: KeyringPair;
  createPolkadotApi: () => Promise<ApiPromise>;

  createBlock: <
    ApiType extends ApiTypes,
    Call extends
      | SubmittableExtrinsic<ApiType>
      | Promise<SubmittableExtrinsic<ApiType>>
      | string
      | Promise<string>
      | Promise<InvokeFunctionResponse>,
    Calls extends Call | Call[],
  >(
    transactions?: Calls,
    options?: BlockCreation,
  ) => Promise<
    BlockCreationResponse<
      ApiType,
      Calls extends Call[]
        ? Array<Awaited<SubmittableExtrinsic<ApiType>>>
        : Awaited<SubmittableExtrinsic<ApiType>>
    >
  >;

  // We also provided singleton providers for simplicity
  polkadotApi: ApiPromise;
  rpcPort: number;
}

interface InternalDevTestContext extends DevTestContext {
  _polkadotApis: ApiPromise[];
}

interface DevMadaraOptions {
  runNewNode?: boolean;
  withWasm?: boolean;
  forkedMode?: boolean;
}

export function describeDevMadara(
  title: string,
  cb: (context: DevTestContext) => void,
  options: DevMadaraOptions = {
    runNewNode: false,
    forkedMode: false,
  },
  runtime: RuntimeChain = "madara",
) {
  describe(title, function () {
    // Set timeout to 50000 for all tests.
    this.timeout(50000);

    chai.use(deepEqualInAnyOrder);
    chai.use(chaiAsPromised);

    // The context is initialized empty to allow passing a reference
    // and to be filled once the node information is retrieved
    const context: InternalDevTestContext = {} as InternalDevTestContext;

    // The currently running node for this describe
    let madaraProcess: ChildProcess;

    // Making sure the Madara node has started
    before("Starting Madara Test Node", async function () {
      this.timeout(SPAWNING_TIME);

      const init = await getRunningNode(runtime, options);
      madaraProcess = init.runningNode;
      context.rpcPort = init.rpcPort;

      // Context is given prior to this assignment, so doing
      // context = init.context will fail because it replace the variable;

      context._polkadotApis = [];
      madaraProcess = init.runningNode;

      context.createPolkadotApi = async () => {
        const apiPromise = await providePolkadotApi(init.rpcPort);
        // We keep track of the polkadotApis to close them at the end of the test
        context._polkadotApis.push(apiPromise);
        await apiPromise.isReady;
        // Necessary hack to allow polkadotApi to finish its internal metadata loading
        // apiPromise.isReady unfortunately doesn't wait for those properly
        await new Promise((resolve) => {
          setTimeout(resolve, 1000);
        });

        return apiPromise;
      };

      context.polkadotApi = await context.createPolkadotApi();

      const keyringSr25519 = new Keyring({ type: "sr25519" });
      context.alice = keyringSr25519.addFromUri("//Alice");

      context.createBlock = async <
        ApiType extends ApiTypes,
        Call extends
          | SubmittableExtrinsic<ApiType>
          | Promise<SubmittableExtrinsic<ApiType>>
          | string
          | Promise<string>
          | Promise<InvokeFunctionResponse>,
        Calls extends Call | Call[],
      >(
        transactions?: Calls,
        options: BlockCreation = {},
      ) => {
        const results: Array<
          { type: "starknet"; hash: string } | { type: "sub"; hash: string }
        > = [];
        const txs =
          transactions == undefined
            ? []
            : Array.isArray(transactions)
            ? transactions
            : [transactions];

        for await (const call of txs) {
          if (call.transaction_hash) {
            // Temporary solution to get the transaction hash back
            // after awaiting the transaction.
            results.push({
              type: "starknet",
              hash: call.transaction_hash,
            });

            // TODO: update this when we have the rpc endpoint
            // results.push({
            //   type: "eth",
            //   hash: (
            //     await customWeb3Request(
            //       context.web3,
            //       "eth_sendRawTransaction",
            //       [call]
            //     )
            //   ).result,
            // });
          } else if (call.isSigned) {
            const tx = context.polkadotApi.tx(call);
            debug(
              `- Signed: ${tx.method.section}.${tx.method.method}(${tx.args
                .map((d) => d.toHuman())
                .join("; ")}) [ nonce: ${tx.nonce}]`,
            );
            results.push({
              type: "sub",
              hash: (await call.send()).toString(),
            });
          } else {
            const tx = context.polkadotApi.tx(call);
            debug(
              `- Unsigned: ${tx.method.section}.${tx.method.method}(${tx.args
                .map((d) => d.toHuman())
                .join("; ")}) [ nonce: ${tx.nonce}]`,
            );
            results.push({
              type: "sub",
              hash: (await call.send()).toString(),
            });
          }
        }

        const { parentHash, finalize } = options;
        const blockResult = await createAndFinalizeBlock(
          context.polkadotApi,
          parentHash,
          finalize,
        );

        // No need to extract events if no transactions
        if (results.length == 0) {
          return {
            block: blockResult,
            result: null,
          };
        }

        // We retrieve the events for that block
        const allRecords: EventRecord[] = (await (
          await context.polkadotApi.at(blockResult.hash)
        ).query.system // eslint-disable-next-line @typescript-eslint/no-explicit-any
          .events()) as any;
        // We retrieve the block (including the extrinsics)
        const blockData = await context.polkadotApi.rpc.chain.getBlock(
          blockResult.hash,
        );

        const result: ExtrinsicCreation[] = results.map((result) => {
          const extrinsicIndex =
            result.type == "starknet"
              ? allRecords
                  .find(
                    ({ phase, event: { section, method, data } }) =>
                      phase.isApplyExtrinsic &&
                      section == "starknet" &&
                      method == "Executed" &&
                      data[2].toString() == result.hash,
                  )
                  ?.phase?.asApplyExtrinsic?.toNumber()
              : blockData.block.extrinsics.findIndex(
                  (ext) => ext.hash.toHex() == result.hash,
                );
          // We retrieve the events associated with the extrinsic
          const events = allRecords.filter(
            ({ phase }) =>
              phase.isApplyExtrinsic &&
              phase.asApplyExtrinsic.toNumber() === extrinsicIndex,
          );
          const failure = extractError(events);
          return {
            extrinsic:
              extrinsicIndex >= 0
                ? blockData.block.extrinsics[extrinsicIndex]
                : null,
            events,
            error:
              failure &&
              ((failure.isModule &&
                context.polkadotApi.registry.findMetaError(failure.asModule)) ||
                ({ name: failure.toString() } as RegistryError)),
            successful: extrinsicIndex !== undefined && !failure,
            hash: result.hash,
          };
        });

        // Adds extra time to avoid empty transaction when querying it
        if (results.find((r) => r.type == "starknet")) {
          await new Promise((resolve) => setTimeout(resolve, 2));
        }
        return {
          block: blockResult,
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          result: Array.isArray(transactions) ? result : (result[0] as any),
        };
      };

      debug(`Setup ready`);
    });

    after(async function () {
      await Promise.all(
        context._polkadotApis.map(async (p) => {
          await p.disconnect();
        }),
      );

      if (madaraProcess) {
        await new Promise((resolve) => {
          madaraProcess.once("exit", resolve);
          madaraProcess.kill();
          madaraProcess = null;
        });
      }
    });

    cb(context);
  });
}

const getRunningNode = async (
  runtime: RuntimeChain,
  options: DevMadaraOptions,
) => {
  if (options.forkedMode) {
    return await startMadaraForkedNode(9933);
  }

  if (!DEBUG_MODE) {
    if (!options.runNewNode) {
      const p2pPort = parseInt(process.env.P2P_PORT);
      const rpcPort = parseInt(process.env.RPC_PORT);
      return {
        runningNode: null,
        p2pPort,
        rpcPort,
      };
    }

    return await startMadaraDevNode(options.withWasm, runtime);
  }

  return {
    runningNode: null,
    p2pPort: 19931,
    rpcPort: 9933,
  };
};
