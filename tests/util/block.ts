import "@keep-starknet-strange/madara-api-augment/sharingan";
import { type ApiPromise } from "@polkadot/api";
import { type Option, type u128, type u32 } from "@polkadot/types";
import { type RuntimeDispatchInfo } from "@polkadot/types/interfaces";
import type { RuntimeDispatchInfoV1 } from "@polkadot/types/interfaces/payment";

import { type DevTestContext } from "./setup-dev-tests";

import type { TxWithEvent } from "@polkadot/api-derive/types";
import type { ITuple } from "@polkadot/types-codec/types";
import type {
  AccountId20,
  Block,
} from "@polkadot/types/interfaces/runtime/types";
import Bottleneck from "bottleneck";
import debugFactory from "debug";
const debug = debugFactory("test:blocks");
export async function createAndFinalizeBlock(
  api: ApiPromise,
  parentHash?: string,
  finalize = true,
): Promise<{
  duration: number;
  hash: string;
}> {
  const startTime: number = Date.now();
  const block = parentHash
    ? await api.rpc.engine.createBlock(true, finalize, parentHash)
    : await api.rpc.engine.createBlock(true, finalize);

  return {
    duration: Date.now() - startTime,
    hash: block.toJSON().hash as string, // toString doesn't work for block hashes
  };
}

export interface TxWithEventAndFee extends TxWithEvent {
  fee: RuntimeDispatchInfo | RuntimeDispatchInfoV1;
}

export interface BlockDetails {
  block: Block;
  txWithEvents: TxWithEventAndFee[];
}

export interface BlockRangeOption {
  from: number;
  to: number;
  concurrency?: number;
}

export async function jumpBlocks(context: DevTestContext, blockCount: number) {
  while (blockCount > 0) {
    (await context.createBlock()).block.hash.toString();
    blockCount--;
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const getBlockTime = (signedBlock: any) =>
  signedBlock.block.extrinsics
    .find((item) => item.method.section == "timestamp")
    .method.args[0].toNumber();

const fetchBlockTime = async (api: ApiPromise, blockNum: number) => {
  const hash = await api.rpc.chain.getBlockHash(blockNum);
  const block = await api.rpc.chain.getBlock(hash);
  return getBlockTime(block);
};

export const fetchHistoricBlockNum = async (
  api: ApiPromise,
  blockNumber: number,
  targetTime: number,
) => {
  if (blockNumber <= 1) {
    return 1;
  }
  const time = await fetchBlockTime(api, blockNumber);

  if (time <= targetTime) {
    return blockNumber;
  }

  return fetchHistoricBlockNum(
    api,
    blockNumber - Math.ceil((time - targetTime) / 30_000),
    targetTime,
  );
};

export const getBlockArray = async (
  api: ApiPromise,
  timePeriod: number,
  limiter?: Bottleneck,
) => {
  /**
  @brief Returns an sequential array of block numbers from a given period of time in the past
  @param api Connected ApiPromise to perform queries on
  @param timePeriod Moment in the past to search until
  @param limiter Bottleneck rate limiter to throttle requests
  */

  if (limiter == null) {
    limiter = new Bottleneck({ maxConcurrent: 10, minTime: 100 });
  }
  const finalizedHead = await limiter.schedule(
    async () => await api.rpc.chain.getFinalizedHead(),
  );
  const signedBlock = await limiter.schedule(
    async () => await api.rpc.chain.getBlock(finalizedHead),
  );

  const lastBlockNumber = signedBlock.block.header.number.toNumber();
  const lastBlockTime = getBlockTime(signedBlock);

  const firstBlockTime = lastBlockTime - timePeriod;
  debug(`Searching for the block at: ${new Date(firstBlockTime)}`);
  const firstBlockNumber = (await limiter.wrap(fetchHistoricBlockNum)(
    api,
    lastBlockNumber,
    firstBlockTime,
  )) as number;

  const length = lastBlockNumber - firstBlockNumber;
  return Array.from({ length }, (_, i) => firstBlockNumber + i);
};

export function extractPreimageDeposit(
  request:
    | Option<ITuple<[AccountId20, u128]>>
    | {
        readonly deposit: ITuple<[AccountId20, u128]>;
        readonly len: u32;
      }
    | {
        readonly deposit: Option<ITuple<[AccountId20, u128]>>;
        readonly count: u32;
        readonly len: Option<u32>;
      },
) {
  const deposit = "deposit" in request ? request.deposit : request;
  if ("isSome" in deposit) {
    return {
      accountId: deposit.unwrap()[0].toHex(),
      amount: deposit.unwrap()[1],
    };
  }
  return {
    accountId: deposit[0].toHex(),
    amount: deposit[1],
  };
}
