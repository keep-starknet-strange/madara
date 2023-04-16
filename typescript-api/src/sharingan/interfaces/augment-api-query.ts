// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import "@polkadot/api-base/types/storage";

import type {
  ApiTypes,
  AugmentedQuery,
  QueryableStorageEntry,
} from "@polkadot/api-base/types";
import type {
  Bytes,
  Option,
  U256,
  U8aFixed,
  Vec,
  bool,
  u128,
  u32,
  u64,
} from "@polkadot/types-codec";
import type { AnyNumber, ITuple } from "@polkadot/types-codec/types";
import type { AccountId32, H256 } from "@polkadot/types/interfaces/runtime";
import type {
  FrameSupportDispatchPerDispatchClassWeight,
  FrameSystemAccountInfo,
  FrameSystemEventRecord,
  FrameSystemLastRuntimeUpgradeInfo,
  FrameSystemPhase,
  MpStarknetBlock,
  MpStarknetExecutionContractClassWrapper,
  MpStarknetTransactionTypesTransaction,
  PalletBalancesAccountData,
  PalletBalancesBalanceLock,
  PalletBalancesReserveData,
  PalletGrandpaStoredPendingChange,
  PalletGrandpaStoredState,
  PalletTransactionPaymentReleases,
  SpConsensusAuraSr25519AppSr25519Public,
  SpRuntimeDigest,
} from "@polkadot/types/lookup";
import type { Observable } from "@polkadot/types/types";

export type __AugmentedQuery<ApiType extends ApiTypes> = AugmentedQuery<
  ApiType,
  () => unknown
>;
export type __QueryableStorageEntry<ApiType extends ApiTypes> =
  QueryableStorageEntry<ApiType>;

declare module "@polkadot/api-base/types/storage" {
  interface AugmentedQueries<ApiType extends ApiTypes> {
    aura: {
      /** The current authority set. */
      authorities: AugmentedQuery<
        ApiType,
        () => Observable<Vec<SpConsensusAuraSr25519AppSr25519Public>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The current slot of this block.
       *
       * This will be set in `on_initialize`.
       */
      currentSlot: AugmentedQuery<ApiType, () => Observable<u64>, []> &
        QueryableStorageEntry<ApiType, []>;
      /** Generic query */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    balances: {
      /**
       * The Balances pallet example of storing the balance of an account.
       *
       * # Example
       *
       * ```nocompile
       * impl pallet_balances::Config for Runtime {
       * type AccountStore = StorageMapShim<Self::Account<Runtime>, frame_system::Provider<Runtime>, AccountId, Self::AccountData<Balance>>
       * }
       * ```
       *
       * You can also store the balance of an account in the `System` pallet.
       *
       * # Example
       *
       * ```nocompile
       * impl pallet_balances::Config for Runtime {
       * type AccountStore = System
       * }
       * ```
       *
       * But this comes with tradeoffs, storing account balances in the system
       * pallet stores `frame_system` data alongside the account data contrary
       * to storing account balances in the `Balances` pallet, which uses a
       * `StorageMap` to store balances data only. NOTE: This is only used in
       * the case that this pallet is used to store balances.
       */
      account: AugmentedQuery<
        ApiType,
        (
          arg: AccountId32 | string | Uint8Array
        ) => Observable<PalletBalancesAccountData>,
        [AccountId32]
      > &
        QueryableStorageEntry<ApiType, [AccountId32]>;
      /** The total units of outstanding deactivated balance in the system. */
      inactiveIssuance: AugmentedQuery<ApiType, () => Observable<u128>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Any liquidity locks on some account balances. NOTE: Should only be
       * accessed when setting, changing and freeing a lock.
       */
      locks: AugmentedQuery<
        ApiType,
        (
          arg: AccountId32 | string | Uint8Array
        ) => Observable<Vec<PalletBalancesBalanceLock>>,
        [AccountId32]
      > &
        QueryableStorageEntry<ApiType, [AccountId32]>;
      /** Named reserves on some account balances. */
      reserves: AugmentedQuery<
        ApiType,
        (
          arg: AccountId32 | string | Uint8Array
        ) => Observable<Vec<PalletBalancesReserveData>>,
        [AccountId32]
      > &
        QueryableStorageEntry<ApiType, [AccountId32]>;
      /** The total units issued in the system. */
      totalIssuance: AugmentedQuery<ApiType, () => Observable<u128>, []> &
        QueryableStorageEntry<ApiType, []>;
      /** Generic query */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    grandpa: {
      /**
       * The number of changes (both in terms of keys and underlying economic
       * responsibilities) in the "set" of Grandpa validators from genesis.
       */
      currentSetId: AugmentedQuery<ApiType, () => Observable<u64>, []> &
        QueryableStorageEntry<ApiType, []>;
      /** Next block number where we can force a change. */
      nextForced: AugmentedQuery<ApiType, () => Observable<Option<u32>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /** Pending change: (signaled at, scheduled change). */
      pendingChange: AugmentedQuery<
        ApiType,
        () => Observable<Option<PalletGrandpaStoredPendingChange>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * A mapping from grandpa set ID to the index of the _most recent_ session
       * for which its members were responsible.
       *
       * This is only used for validating equivocation proofs. An equivocation
       * proof must contains a key-ownership proof for a given session,
       * therefore we need a way to tie together sessions and GRANDPA set ids,
       * i.e. we need to validate that a validator was the owner of a given key
       * on a given session, and what the active set ID was during that session.
       *
       * TWOX-NOTE: `SetId` is not under user control.
       */
      setIdSession: AugmentedQuery<
        ApiType,
        (arg: u64 | AnyNumber | Uint8Array) => Observable<Option<u32>>,
        [u64]
      > &
        QueryableStorageEntry<ApiType, [u64]>;
      /** `true` if we are currently stalled. */
      stalled: AugmentedQuery<
        ApiType,
        () => Observable<Option<ITuple<[u32, u32]>>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /** State of the current authority set. */
      state: AugmentedQuery<
        ApiType,
        () => Observable<PalletGrandpaStoredState>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /** Generic query */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    starknet: {
      blockHash: AugmentedQuery<
        ApiType,
        (arg: U256 | AnyNumber | Uint8Array) => Observable<H256>,
        [U256]
      > &
        QueryableStorageEntry<ApiType, [U256]>;
      /** Mapping from Starknet class hash to contract class. */
      contractClasses: AugmentedQuery<
        ApiType,
        (
          arg: U8aFixed | string | Uint8Array
        ) => Observable<MpStarknetExecutionContractClassWrapper>,
        [U8aFixed]
      > &
        QueryableStorageEntry<ApiType, [U8aFixed]>;
      /** Mapping from Starknet contract address to the contract's class hash. */
      contractClassHashes: AugmentedQuery<
        ApiType,
        (arg: U8aFixed | string | Uint8Array) => Observable<U8aFixed>,
        [U8aFixed]
      > &
        QueryableStorageEntry<ApiType, [U8aFixed]>;
      /** The current Starknet block. */
      currentBlock: AugmentedQuery<
        ApiType,
        () => Observable<MpStarknetBlock>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /** The address of the fee token ERC20 contract. */
      feeTokenAddress: AugmentedQuery<ApiType, () => Observable<U8aFixed>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * The last processed Ethereum block number for L1 messages consumption.
       * This is used to avoid re-processing the same Ethereum block multiple
       * times. This is used by the offchain worker.
       *
       * # TODO
       *
       * - Find a more relevant name for this.
       */
      lastKnownEthBlock: AugmentedQuery<
        ApiType,
        () => Observable<Option<u64>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /** Mapping from Starknet contract address to its nonce. */
      nonces: AugmentedQuery<
        ApiType,
        (arg: U8aFixed | string | Uint8Array) => Observable<U256>,
        [U8aFixed]
      > &
        QueryableStorageEntry<ApiType, [U8aFixed]>;
      /** The Starknet pallet storage items. STORAGE Current building block's transactions. */
      pending: AugmentedQuery<
        ApiType,
        () => Observable<Vec<MpStarknetTransactionTypesTransaction>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /** Mapping from Starknet contract storage key to its value. */
      storageView: AugmentedQuery<
        ApiType,
        (
          arg:
            | ITuple<[U8aFixed, H256]>
            | [U8aFixed | string | Uint8Array, H256 | string | Uint8Array]
        ) => Observable<U256>,
        [ITuple<[U8aFixed, H256]>]
      > &
        QueryableStorageEntry<ApiType, [ITuple<[U8aFixed, H256]>]>;
      /** Generic query */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    sudo: {
      /** The `AccountId` of the sudo key. */
      key: AugmentedQuery<ApiType, () => Observable<Option<AccountId32>>, []> &
        QueryableStorageEntry<ApiType, []>;
      /** Generic query */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    system: {
      /** The full account information for a particular account ID. */
      account: AugmentedQuery<
        ApiType,
        (
          arg: AccountId32 | string | Uint8Array
        ) => Observable<FrameSystemAccountInfo>,
        [AccountId32]
      > &
        QueryableStorageEntry<ApiType, [AccountId32]>;
      /** Total length (in bytes) for all extrinsics put together, for the current block. */
      allExtrinsicsLen: AugmentedQuery<
        ApiType,
        () => Observable<Option<u32>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /** Map of block numbers to block hashes. */
      blockHash: AugmentedQuery<
        ApiType,
        (arg: u32 | AnyNumber | Uint8Array) => Observable<H256>,
        [u32]
      > &
        QueryableStorageEntry<ApiType, [u32]>;
      /** The current weight for the block. */
      blockWeight: AugmentedQuery<
        ApiType,
        () => Observable<FrameSupportDispatchPerDispatchClassWeight>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /** Digest of the current block, also part of the block header. */
      digest: AugmentedQuery<ApiType, () => Observable<SpRuntimeDigest>, []> &
        QueryableStorageEntry<ApiType, []>;
      /** The number of events in the `Events<T>` list. */
      eventCount: AugmentedQuery<ApiType, () => Observable<u32>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Events deposited for the current block.
       *
       * NOTE: The item is unbound and should therefore never be read on chain.
       * It could otherwise inflate the PoV size of a block.
       *
       * Events have a large in-memory size. Box the events to not go
       * out-of-memory just in case someone still reads them from within the runtime.
       */
      events: AugmentedQuery<
        ApiType,
        () => Observable<Vec<FrameSystemEventRecord>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * Mapping between a topic (represented by T::Hash) and a vector of
       * indexes of events in the `<Events<T>>` list.
       *
       * All topic vectors have deterministic storage locations depending on the
       * topic. This allows light-clients to leverage the changes trie storage
       * tracking mechanism and in case of changes fetch the list of events of interest.
       *
       * The value has the type `(T::BlockNumber, EventIndex)` because if we
       * used only just the `EventIndex` then in case if the topic has the same
       * contents on the next block no notification will be triggered thus the
       * event might be lost.
       */
      eventTopics: AugmentedQuery<
        ApiType,
        (
          arg: H256 | string | Uint8Array
        ) => Observable<Vec<ITuple<[u32, u32]>>>,
        [H256]
      > &
        QueryableStorageEntry<ApiType, [H256]>;
      /** The execution phase of the block. */
      executionPhase: AugmentedQuery<
        ApiType,
        () => Observable<Option<FrameSystemPhase>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /** Total extrinsics count for the current block. */
      extrinsicCount: AugmentedQuery<
        ApiType,
        () => Observable<Option<u32>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /** Extrinsics data for the current block (maps an extrinsic's index to its data). */
      extrinsicData: AugmentedQuery<
        ApiType,
        (arg: u32 | AnyNumber | Uint8Array) => Observable<Bytes>,
        [u32]
      > &
        QueryableStorageEntry<ApiType, [u32]>;
      /**
       * Stores the `spec_version` and `spec_name` of when the last runtime
       * upgrade happened.
       */
      lastRuntimeUpgrade: AugmentedQuery<
        ApiType,
        () => Observable<Option<FrameSystemLastRuntimeUpgradeInfo>>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /** The current block number being processed. Set by `execute_block`. */
      number: AugmentedQuery<ApiType, () => Observable<u32>, []> &
        QueryableStorageEntry<ApiType, []>;
      /** Hash of the previous block. */
      parentHash: AugmentedQuery<ApiType, () => Observable<H256>, []> &
        QueryableStorageEntry<ApiType, []>;
      /**
       * True if we have upgraded so that AccountInfo contains three types of
       * `RefCount`. False (default) if not.
       */
      upgradedToTripleRefCount: AugmentedQuery<
        ApiType,
        () => Observable<bool>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /**
       * True if we have upgraded so that `type RefCount` is `u32`. False
       * (default) if not.
       */
      upgradedToU32RefCount: AugmentedQuery<
        ApiType,
        () => Observable<bool>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /** Generic query */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    timestamp: {
      /** Did the timestamp get updated in this block? */
      didUpdate: AugmentedQuery<ApiType, () => Observable<bool>, []> &
        QueryableStorageEntry<ApiType, []>;
      /** Current time for the current block. */
      now: AugmentedQuery<ApiType, () => Observable<u64>, []> &
        QueryableStorageEntry<ApiType, []>;
      /** Generic query */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
    transactionPayment: {
      nextFeeMultiplier: AugmentedQuery<ApiType, () => Observable<u128>, []> &
        QueryableStorageEntry<ApiType, []>;
      storageVersion: AugmentedQuery<
        ApiType,
        () => Observable<PalletTransactionPaymentReleases>,
        []
      > &
        QueryableStorageEntry<ApiType, []>;
      /** Generic query */
      [key: string]: QueryableStorageEntry<ApiType>;
    };
  } // AugmentedQueries
} // declare module
