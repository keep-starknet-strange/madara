// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import "@polkadot/api-base/types/submittable";

import type {
  ApiTypes,
  AugmentedSubmittable,
  SubmittableExtrinsic,
  SubmittableExtrinsicFunction,
} from "@polkadot/api-base/types";
import type {
  Bytes,
  Compact,
  U8aFixed,
  Vec,
  bool,
  u128,
  u16,
  u32,
  u64,
} from "@polkadot/types-codec";
import type { AnyNumber, IMethod, ITuple } from "@polkadot/types-codec/types";
import type { Call, MultiAddress } from "@polkadot/types/interfaces/runtime";
import type {
  MadaraRuntimeOriginCaller,
  MpStarknetTransactionTypesTransaction,
  SpConsensusGrandpaEquivocationProof,
  SpCoreVoid,
  SpWeightsWeightV2Weight,
} from "@polkadot/types/lookup";

export type __AugmentedSubmittable = AugmentedSubmittable<() => unknown>;
export type __SubmittableExtrinsic<ApiType extends ApiTypes> =
  SubmittableExtrinsic<ApiType>;
export type __SubmittableExtrinsicFunction<ApiType extends ApiTypes> =
  SubmittableExtrinsicFunction<ApiType>;

declare module "@polkadot/api-base/types/submittable" {
  interface AugmentedSubmittables<ApiType extends ApiTypes> {
    balances: {
      /**
       * Exactly as `transfer`, except the origin must be root and the source
       * account may be specified.
       *
       * ## Complexity
       *
       * - Same as transfer, but additional read and write because the source
       *   account is not assumed to be in the overlay.
       */
      forceTransfer: AugmentedSubmittable<
        (
          source:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          dest:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, MultiAddress, Compact<u128>]
      >;
      /**
       * Unreserve some balance from a user by force.
       *
       * Can only be called by ROOT.
       */
      forceUnreserve: AugmentedSubmittable<
        (
          who:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          amount: u128 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, u128]
      >;
      /**
       * Set the balances of a given account.
       *
       * This will alter `FreeBalance` and `ReservedBalance` in storage. it will
       * also alter the total issuance of the system (`TotalIssuance`)
       * appropriately. If the new free or reserved balance is below the
       * existential deposit, it will reset the account nonce
       * (`frame_system::AccountNonce`).
       *
       * The dispatch origin for this call is `root`.
       */
      setBalance: AugmentedSubmittable<
        (
          who:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          newFree: Compact<u128> | AnyNumber | Uint8Array,
          newReserved: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, Compact<u128>, Compact<u128>]
      >;
      /**
       * Transfer some liquid free balance to another account.
       *
       * `transfer` will set the `FreeBalance` of the sender and receiver. If
       * the sender's account is below the existential deposit as a result of
       * the transfer, the account will be reaped.
       *
       * The dispatch origin for this call must be `Signed` by the transactor.
       *
       * ## Complexity
       *
       * - Dependent on arguments but not critical, given proper implementations
       *   for input config types. See related functions below.
       * - It contains a limited number of reads and writes internally and no
       *   complex computation.
       *
       * Related functions:
       *
       * - `ensure_can_withdraw` is always called internally but has a bounded complexity.
       * - Transferring balances to accounts that did not exist before will cause
       *   `T::OnNewAccount::on_new_account` to be called.
       * - Removing enough funds from an account will trigger
       *   `T::DustRemoval::on_unbalanced`.
       * - `transfer_keep_alive` works the same way as `transfer`, but has an
       *   additional check that the transfer will not kill the origin account.
       */
      transfer: AugmentedSubmittable<
        (
          dest:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, Compact<u128>]
      >;
      /**
       * Transfer the entire transferable balance from the caller account.
       *
       * NOTE: This function only attempts to transfer _transferable_ balances.
       * This means that any locked, reserved, or existential deposits (when
       * `keep_alive` is `true`), will not be transferred by this function. To
       * ensure that this function results in a killed account, you might need
       * to prepare the account by removing any reference counters, storage
       * deposits, etc...
       *
       * The dispatch origin of this call must be Signed.
       *
       * - `dest`: The recipient of the transfer.
       * - `keep_alive`: A boolean to determine if the `transfer_all` operation
       *   should send all of the funds the account has, causing the sender
       *   account to be killed (false), or transfer everything except at least
       *   the existential deposit, which will guarantee to keep the sender
       *   account alive (true). ## Complexity
       * - O(1). Just like transfer, but reading the user's transferable balance first.
       */
      transferAll: AugmentedSubmittable<
        (
          dest:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          keepAlive: bool | boolean | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, bool]
      >;
      /**
       * Same as the [`transfer`][`transfer`] call, but with a check that the
       * transfer will not kill the origin account.
       *
       * 99% of the time you want [`transfer`][`transfer`] instead.
       *
       * [`transfer`]: struct.Pallet.html#method.transfer
       */
      transferKeepAlive: AugmentedSubmittable<
        (
          dest:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          value: Compact<u128> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, Compact<u128>]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    grandpa: {
      /**
       * Note that the current authority set of the GRANDPA finality gadget has stalled.
       *
       * This will trigger a forced authority set change at the beginning of the
       * next session, to be enacted `delay` blocks after that. The `delay`
       * should be high enough to safely assume that the block signalling the
       * forced change will not be re-orged e.g. 1000 blocks. The block
       * production rate (which may be slowed down because of finality lagging)
       * should be taken into account when choosing the `delay`. The GRANDPA
       * voters based on the new authority will start voting on top of
       * `best_finalized_block_number` for new finalized blocks.
       * `best_finalized_block_number` should be the highest of the latest
       * finalized block of all validators of the new authority set.
       *
       * Only callable by root.
       */
      noteStalled: AugmentedSubmittable<
        (
          delay: u32 | AnyNumber | Uint8Array,
          bestFinalizedBlockNumber: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u32, u32]
      >;
      /**
       * Report voter equivocation/misbehavior. This method will verify the
       * equivocation proof and validate the given key ownership proof against
       * the extracted offender. If both are valid, the offence will be reported.
       */
      reportEquivocation: AugmentedSubmittable<
        (
          equivocationProof:
            | SpConsensusGrandpaEquivocationProof
            | { setId?: any; equivocation?: any }
            | string
            | Uint8Array,
          keyOwnerProof: SpCoreVoid | null
        ) => SubmittableExtrinsic<ApiType>,
        [SpConsensusGrandpaEquivocationProof, SpCoreVoid]
      >;
      /**
       * Report voter equivocation/misbehavior. This method will verify the
       * equivocation proof and validate the given key ownership proof against
       * the extracted offender. If both are valid, the offence will be reported.
       *
       * This extrinsic must be called unsigned and it is expected that only
       * block authors will call it (validated in `ValidateUnsigned`), as such
       * if the block author is defined it will be defined as the equivocation reporter.
       */
      reportEquivocationUnsigned: AugmentedSubmittable<
        (
          equivocationProof:
            | SpConsensusGrandpaEquivocationProof
            | { setId?: any; equivocation?: any }
            | string
            | Uint8Array,
          keyOwnerProof: SpCoreVoid | null
        ) => SubmittableExtrinsic<ApiType>,
        [SpConsensusGrandpaEquivocationProof, SpCoreVoid]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    starknet: {
      /**
       * Consume a message from L1.
       *
       * # Arguments
       *
       * - `origin` - The origin of the transaction.
       * - `transaction` - The Starknet transaction.
       *
       * # Returns
       *
       * - `DispatchResult` - The result of the transaction.
       *
       * # TODO
       *
       * - Compute weight
       */
      consumeL1Message: AugmentedSubmittable<
        (
          transaction:
            | MpStarknetTransactionTypesTransaction
            | {
                version?: any;
                hash_?: any;
                signature?: any;
                events?: any;
                senderAddress?: any;
                nonce?: any;
                callEntrypoint?: any;
                contractClass?: any;
                contractAddressSalt?: any;
              }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MpStarknetTransactionTypesTransaction]
      >;
      /**
       * The declare transaction is used to introduce new classes into the state
       * of Starknet, enabling other contracts to deploy instances of those
       * classes or using them in a library call. See
       * `https://docs.starknet.io/documentation/architecture_and_concepts/Blocks/transactions/#declare_transaction`.
       *
       * # Arguments
       *
       * - `origin` - The origin of the transaction.
       * - `transaction` - The Starknet transaction.
       *
       * # Returns
       *
       * - `DispatchResult` - The result of the transaction.
       *
       * # TODO
       *
       * - Compute weight
       */
      declare: AugmentedSubmittable<
        (
          transaction:
            | MpStarknetTransactionTypesTransaction
            | {
                version?: any;
                hash_?: any;
                signature?: any;
                events?: any;
                senderAddress?: any;
                nonce?: any;
                callEntrypoint?: any;
                contractClass?: any;
                contractAddressSalt?: any;
              }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MpStarknetTransactionTypesTransaction]
      >;
      /**
       * Since StarkNet v0.10.1 the deploy_account transaction replaces the
       * deploy transaction for deploying account contracts. To use it, you
       * should first pre-fund your would-be account address so that you could
       * pay the transaction fee (see here for more details) . You can then send
       * the deploy_account transaction. See
       * `https://docs.starknet.io/documentation/architecture_and_concepts/Blocks/transactions/#deploy_account_transaction`.
       *
       * # Arguments
       *
       * - `origin` - The origin of the transaction.
       * - `transaction` - The Starknet transaction.
       *
       * # Returns
       *
       * - `DispatchResult` - The result of the transaction.
       *
       * # TODO
       *
       * - Compute weight
       */
      deployAccount: AugmentedSubmittable<
        (
          transaction:
            | MpStarknetTransactionTypesTransaction
            | {
                version?: any;
                hash_?: any;
                signature?: any;
                events?: any;
                senderAddress?: any;
                nonce?: any;
                callEntrypoint?: any;
                contractClass?: any;
                contractAddressSalt?: any;
              }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MpStarknetTransactionTypesTransaction]
      >;
      /**
       * The invoke transaction is the main transaction type used to invoke
       * contract functions in Starknet. See
       * `https://docs.starknet.io/documentation/architecture_and_concepts/Blocks/transactions/#invoke_transaction`.
       *
       * # Arguments
       *
       * - `origin` - The origin of the transaction.
       * - `transaction` - The Starknet transaction.
       *
       * # Returns
       *
       * - `DispatchResult` - The result of the transaction.
       *
       * # TODO
       *
       * - Compute weight
       */
      invoke: AugmentedSubmittable<
        (
          transaction:
            | MpStarknetTransactionTypesTransaction
            | {
                version?: any;
                hash_?: any;
                signature?: any;
                events?: any;
                senderAddress?: any;
                nonce?: any;
                callEntrypoint?: any;
                contractClass?: any;
                contractAddressSalt?: any;
              }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MpStarknetTransactionTypesTransaction]
      >;
      /** Ping the pallet to check if it is alive. */
      ping: AugmentedSubmittable<() => SubmittableExtrinsic<ApiType>, []>;
      /**
       * Set the value of the fee token address.
       *
       * # Arguments
       *
       * - `origin` - The origin of the transaction.
       * - `fee_token_address` - The value of the fee token address.
       *
       * # Returns
       *
       * - `DispatchResult` - The result of the transaction.
       *
       * # TODO
       *
       * - Add some limitations on how often this can be called.
       */
      setFeeTokenAddress: AugmentedSubmittable<
        (
          feeTokenAddress: U8aFixed | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [U8aFixed]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    sudo: {
      /**
       * Authenticates the current sudo key and sets the given AccountId (`new`)
       * as the new sudo key.
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * ## Complexity
       *
       * - O(1).
       */
      setKey: AugmentedSubmittable<
        (
          updated:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress]
      >;
      /**
       * Authenticates the sudo key and dispatches a function call with `Root` origin.
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * ## Complexity
       *
       * - O(1).
       */
      sudo: AugmentedSubmittable<
        (
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Call]
      >;
      /**
       * Authenticates the sudo key and dispatches a function call with `Signed`
       * origin from a given account.
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * ## Complexity
       *
       * - O(1).
       */
      sudoAs: AugmentedSubmittable<
        (
          who:
            | MultiAddress
            | { Id: any }
            | { Index: any }
            | { Raw: any }
            | { Address32: any }
            | { Address20: any }
            | string
            | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MultiAddress, Call]
      >;
      /**
       * Authenticates the sudo key and dispatches a function call with `Root`
       * origin. This function does not check the weight of the call, and
       * instead allows the Sudo user to specify the weight of the call.
       *
       * The dispatch origin for this call must be _Signed_.
       *
       * ## Complexity
       *
       * - O(1).
       */
      sudoUncheckedWeight: AugmentedSubmittable<
        (
          call: Call | IMethod | string | Uint8Array,
          weight:
            | SpWeightsWeightV2Weight
            | { refTime?: any; proofSize?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Call, SpWeightsWeightV2Weight]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    system: {
      /**
       * Kill all storage items with a key that starts with the given prefix.
       *
       * **NOTE:** We rely on the Root origin to provide us the number of
       * subkeys under the prefix we are removing to accurately calculate the
       * weight of this function.
       */
      killPrefix: AugmentedSubmittable<
        (
          prefix: Bytes | string | Uint8Array,
          subkeys: u32 | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Bytes, u32]
      >;
      /** Kill some items from storage. */
      killStorage: AugmentedSubmittable<
        (
          keys: Vec<Bytes> | (Bytes | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<Bytes>]
      >;
      /**
       * Make some on-chain remark.
       *
       * ## Complexity
       *
       * - `O(1)`
       */
      remark: AugmentedSubmittable<
        (remark: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /** Make some on-chain remark and emit event. */
      remarkWithEvent: AugmentedSubmittable<
        (remark: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Set the new runtime code.
       *
       * ## Complexity
       *
       * - `O(C + S)` where `C` length of `code` and `S` complexity of `can_set_code`
       */
      setCode: AugmentedSubmittable<
        (code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /**
       * Set the new runtime code without doing any checks of the given `code`.
       *
       * ## Complexity
       *
       * - `O(C)` where `C` length of `code`
       */
      setCodeWithoutChecks: AugmentedSubmittable<
        (code: Bytes | string | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [Bytes]
      >;
      /** Set the number of pages in the WebAssembly environment's heap. */
      setHeapPages: AugmentedSubmittable<
        (pages: u64 | AnyNumber | Uint8Array) => SubmittableExtrinsic<ApiType>,
        [u64]
      >;
      /** Set some items of storage. */
      setStorage: AugmentedSubmittable<
        (
          items:
            | Vec<ITuple<[Bytes, Bytes]>>
            | [Bytes | string | Uint8Array, Bytes | string | Uint8Array][]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<ITuple<[Bytes, Bytes]>>]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    timestamp: {
      /**
       * Set the current time.
       *
       * This call should be invoked exactly once per block. It will panic at
       * the finalization phase, if this call hasn't been invoked by that time.
       *
       * The timestamp should be greater than the previous one by the amount
       * specified by `MinimumPeriod`.
       *
       * The dispatch origin for this call must be `Inherent`.
       *
       * ## Complexity
       *
       * - `O(1)` (Note that implementations of `OnTimestampSet` must also be `O(1)`)
       * - 1 storage read and 1 storage mutation (codec `O(1)`). (because of
       *   `DidUpdate::take` in `on_finalize`)
       * - 1 event handler `on_timestamp_set`. Must be `O(1)`.
       */
      set: AugmentedSubmittable<
        (
          now: Compact<u64> | AnyNumber | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Compact<u64>]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
    utility: {
      /**
       * Send a call through an indexed pseudonym of the sender.
       *
       * Filter from origin are passed along. The call will be dispatched with
       * an origin which use the same filter as the origin of this call.
       *
       * NOTE: If you need to ensure that any account-based filtering is not
       * honored (i.e. because you expect `proxy` to have been used prior in the
       * call stack and you do not want the call restrictions to apply to any
       * sub-accounts), then use `as_multi_threshold_1` in the Multisig pallet instead.
       *
       * NOTE: Prior to version *12, this was called `as_limited_sub`.
       *
       * The dispatch origin for this call must be _Signed_.
       */
      asDerivative: AugmentedSubmittable<
        (
          index: u16 | AnyNumber | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [u16, Call]
      >;
      /**
       * Send a batch of dispatch calls.
       *
       * May be called from any origin except `None`.
       *
       * - `calls`: The calls to be dispatched from the same origin. The number of
       *   call must not exceed the constant: `batched_calls_limit` (available
       *   in constant metadata).
       *
       * If origin is root then the calls are dispatched without checking origin
       * filter. (This includes bypassing `frame_system::Config::BaseCallFilter`).
       *
       * ## Complexity
       *
       * - O(C) where C is the number of calls to be batched.
       *
       * This will return `Ok` in all circumstances. To determine the success of
       * the batch, an event is deposited. If a call failed and the batch was
       * interrupted, then the `BatchInterrupted` event is deposited, along with
       * the number of successful calls made and the error of the failed call.
       * If all were successful, then the `BatchCompleted` event is deposited.
       */
      batch: AugmentedSubmittable<
        (
          calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<Call>]
      >;
      /**
       * Send a batch of dispatch calls and atomically execute them. The whole
       * transaction will rollback and fail if any of the calls failed.
       *
       * May be called from any origin except `None`.
       *
       * - `calls`: The calls to be dispatched from the same origin. The number of
       *   call must not exceed the constant: `batched_calls_limit` (available
       *   in constant metadata).
       *
       * If origin is root then the calls are dispatched without checking origin
       * filter. (This includes bypassing `frame_system::Config::BaseCallFilter`).
       *
       * ## Complexity
       *
       * - O(C) where C is the number of calls to be batched.
       */
      batchAll: AugmentedSubmittable<
        (
          calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<Call>]
      >;
      /**
       * Dispatches a function call with a provided origin.
       *
       * The dispatch origin for this call must be _Root_.
       *
       * ## Complexity
       *
       * - O(1).
       */
      dispatchAs: AugmentedSubmittable<
        (
          asOrigin:
            | MadaraRuntimeOriginCaller
            | { system: any }
            | { Void: any }
            | string
            | Uint8Array,
          call: Call | IMethod | string | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [MadaraRuntimeOriginCaller, Call]
      >;
      /**
       * Send a batch of dispatch calls. Unlike `batch`, it allows errors and
       * won't interrupt.
       *
       * May be called from any origin except `None`.
       *
       * - `calls`: The calls to be dispatched from the same origin. The number of
       *   call must not exceed the constant: `batched_calls_limit` (available
       *   in constant metadata).
       *
       * If origin is root then the calls are dispatch without checking origin
       * filter. (This includes bypassing `frame_system::Config::BaseCallFilter`).
       *
       * ## Complexity
       *
       * - O(C) where C is the number of calls to be batched.
       */
      forceBatch: AugmentedSubmittable<
        (
          calls: Vec<Call> | (Call | IMethod | string | Uint8Array)[]
        ) => SubmittableExtrinsic<ApiType>,
        [Vec<Call>]
      >;
      /**
       * Dispatch a function call with a specified weight.
       *
       * This function does not check the weight of the call, and instead allows
       * the Root origin to specify the weight of the call.
       *
       * The dispatch origin for this call must be _Root_.
       */
      withWeight: AugmentedSubmittable<
        (
          call: Call | IMethod | string | Uint8Array,
          weight:
            | SpWeightsWeightV2Weight
            | { refTime?: any; proofSize?: any }
            | string
            | Uint8Array
        ) => SubmittableExtrinsic<ApiType>,
        [Call, SpWeightsWeightV2Weight]
      >;
      /** Generic tx */
      [key: string]: SubmittableExtrinsicFunction<ApiType>;
    };
  } // AugmentedSubmittables
} // declare module
