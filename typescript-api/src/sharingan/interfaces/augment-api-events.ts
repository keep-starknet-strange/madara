// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import "@polkadot/api-base/types/events";

import type { ApiTypes, AugmentedEvent } from "@polkadot/api-base/types";
import type {
  Null,
  Option,
  Result,
  U8aFixed,
  Vec,
  u128,
  u32,
  u64,
} from "@polkadot/types-codec";
import type { ITuple } from "@polkadot/types-codec/types";
import type { AccountId32, H256 } from "@polkadot/types/interfaces/runtime";
import type {
  FrameSupportDispatchDispatchInfo,
  FrameSupportTokensMiscBalanceStatus,
  MpStarknetTransactionTypesEventWrapper,
  SpConsensusGrandpaAppPublic,
  SpRuntimeDispatchError,
} from "@polkadot/types/lookup";

export type __AugmentedEvent<ApiType extends ApiTypes> =
  AugmentedEvent<ApiType>;

declare module "@polkadot/api-base/types/events" {
  interface AugmentedEvents<ApiType extends ApiTypes> {
    balances: {
      /** A balance was set by root. */
      BalanceSet: AugmentedEvent<
        ApiType,
        [who: AccountId32, free: u128, reserved: u128],
        { who: AccountId32; free: u128; reserved: u128 }
      >;
      /** Some amount was deposited (e.g. for transaction fees). */
      Deposit: AugmentedEvent<
        ApiType,
        [who: AccountId32, amount: u128],
        { who: AccountId32; amount: u128 }
      >;
      /**
       * An account was removed whose balance was non-zero but below
       * ExistentialDeposit, resulting in an outright loss.
       */
      DustLost: AugmentedEvent<
        ApiType,
        [account: AccountId32, amount: u128],
        { account: AccountId32; amount: u128 }
      >;
      /** An account was created with some free balance. */
      Endowed: AugmentedEvent<
        ApiType,
        [account: AccountId32, freeBalance: u128],
        { account: AccountId32; freeBalance: u128 }
      >;
      /** Some balance was reserved (moved from free to reserved). */
      Reserved: AugmentedEvent<
        ApiType,
        [who: AccountId32, amount: u128],
        { who: AccountId32; amount: u128 }
      >;
      /**
       * Some balance was moved from the reserve of the first account to the
       * second account. Final argument indicates the destination balance type.
       */
      ReserveRepatriated: AugmentedEvent<
        ApiType,
        [
          from: AccountId32,
          to: AccountId32,
          amount: u128,
          destinationStatus: FrameSupportTokensMiscBalanceStatus
        ],
        {
          from: AccountId32;
          to: AccountId32;
          amount: u128;
          destinationStatus: FrameSupportTokensMiscBalanceStatus;
        }
      >;
      /** Some amount was removed from the account (e.g. for misbehavior). */
      Slashed: AugmentedEvent<
        ApiType,
        [who: AccountId32, amount: u128],
        { who: AccountId32; amount: u128 }
      >;
      /** Transfer succeeded. */
      Transfer: AugmentedEvent<
        ApiType,
        [from: AccountId32, to: AccountId32, amount: u128],
        { from: AccountId32; to: AccountId32; amount: u128 }
      >;
      /** Some balance was unreserved (moved from reserved to free). */
      Unreserved: AugmentedEvent<
        ApiType,
        [who: AccountId32, amount: u128],
        { who: AccountId32; amount: u128 }
      >;
      /** Some amount was withdrawn from the account (e.g. for transaction fees). */
      Withdraw: AugmentedEvent<
        ApiType,
        [who: AccountId32, amount: u128],
        { who: AccountId32; amount: u128 }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    grandpa: {
      /** New authority set has been applied. */
      NewAuthorities: AugmentedEvent<
        ApiType,
        [authoritySet: Vec<ITuple<[SpConsensusGrandpaAppPublic, u64]>>],
        { authoritySet: Vec<ITuple<[SpConsensusGrandpaAppPublic, u64]>> }
      >;
      /** Current authority set has been paused. */
      Paused: AugmentedEvent<ApiType, []>;
      /** Current authority set has been resumed. */
      Resumed: AugmentedEvent<ApiType, []>;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    starknet: {
      /**
       * Emitted when fee token address is changed. This is emitted by the
       * `set_fee_token_address` extrinsic. [old_fee_token_address,
       * new_fee_token_address]
       */
      FeeTokenAddressChanged: AugmentedEvent<
        ApiType,
        [oldFeeTokenAddress: U8aFixed, newFeeTokenAddress: U8aFixed],
        { oldFeeTokenAddress: U8aFixed; newFeeTokenAddress: U8aFixed }
      >;
      KeepStarknetStrange: AugmentedEvent<ApiType, []>;
      /** Regular Starknet event */
      StarknetEvent: AugmentedEvent<
        ApiType,
        [MpStarknetTransactionTypesEventWrapper]
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    sudo: {
      /** The [sudoer] just switched identity; the old key is supplied if one existed. */
      KeyChanged: AugmentedEvent<
        ApiType,
        [oldSudoer: Option<AccountId32>],
        { oldSudoer: Option<AccountId32> }
      >;
      /** A sudo just took place. [result] */
      Sudid: AugmentedEvent<
        ApiType,
        [sudoResult: Result<Null, SpRuntimeDispatchError>],
        { sudoResult: Result<Null, SpRuntimeDispatchError> }
      >;
      /** A sudo just took place. [result] */
      SudoAsDone: AugmentedEvent<
        ApiType,
        [sudoResult: Result<Null, SpRuntimeDispatchError>],
        { sudoResult: Result<Null, SpRuntimeDispatchError> }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    system: {
      /** `:code` was updated. */
      CodeUpdated: AugmentedEvent<ApiType, []>;
      /** An extrinsic failed. */
      ExtrinsicFailed: AugmentedEvent<
        ApiType,
        [
          dispatchError: SpRuntimeDispatchError,
          dispatchInfo: FrameSupportDispatchDispatchInfo
        ],
        {
          dispatchError: SpRuntimeDispatchError;
          dispatchInfo: FrameSupportDispatchDispatchInfo;
        }
      >;
      /** An extrinsic completed successfully. */
      ExtrinsicSuccess: AugmentedEvent<
        ApiType,
        [dispatchInfo: FrameSupportDispatchDispatchInfo],
        { dispatchInfo: FrameSupportDispatchDispatchInfo }
      >;
      /** An account was reaped. */
      KilledAccount: AugmentedEvent<
        ApiType,
        [account: AccountId32],
        { account: AccountId32 }
      >;
      /** A new account was created. */
      NewAccount: AugmentedEvent<
        ApiType,
        [account: AccountId32],
        { account: AccountId32 }
      >;
      /** On on-chain remark happened. */
      Remarked: AugmentedEvent<
        ApiType,
        [sender: AccountId32, hash_: H256],
        { sender: AccountId32; hash_: H256 }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    transactionPayment: {
      /**
       * A transaction fee `actual_fee`, of which `tip` was added to the minimum
       * inclusion fee, has been paid by `who`.
       */
      TransactionFeePaid: AugmentedEvent<
        ApiType,
        [who: AccountId32, actualFee: u128, tip: u128],
        { who: AccountId32; actualFee: u128; tip: u128 }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
    utility: {
      /** Batch of dispatches completed fully with no error. */
      BatchCompleted: AugmentedEvent<ApiType, []>;
      /** Batch of dispatches completed but has errors. */
      BatchCompletedWithErrors: AugmentedEvent<ApiType, []>;
      /**
       * Batch of dispatches did not complete fully. Index of first failing
       * dispatch given, as well as the error.
       */
      BatchInterrupted: AugmentedEvent<
        ApiType,
        [index: u32, error: SpRuntimeDispatchError],
        { index: u32; error: SpRuntimeDispatchError }
      >;
      /** A call was dispatched. */
      DispatchedAs: AugmentedEvent<
        ApiType,
        [result: Result<Null, SpRuntimeDispatchError>],
        { result: Result<Null, SpRuntimeDispatchError> }
      >;
      /** A single item within a Batch of dispatches has completed with no error. */
      ItemCompleted: AugmentedEvent<ApiType, []>;
      /** A single item within a Batch of dispatches has completed with error. */
      ItemFailed: AugmentedEvent<
        ApiType,
        [error: SpRuntimeDispatchError],
        { error: SpRuntimeDispatchError }
      >;
      /** Generic event */
      [key: string]: AugmentedEvent<ApiType>;
    };
  } // AugmentedEvents
} // declare module
