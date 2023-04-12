// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import "@polkadot/api-base/types/consts";

import type { ApiTypes, AugmentedConst } from "@polkadot/api-base/types";
import type { u128, u16, u32, u64, u8 } from "@polkadot/types-codec";
import type { Codec } from "@polkadot/types-codec/types";
import type {
  FrameSystemLimitsBlockLength,
  FrameSystemLimitsBlockWeights,
  SpVersionRuntimeVersion,
  SpWeightsRuntimeDbWeight,
} from "@polkadot/types/lookup";

export type __AugmentedConst<ApiType extends ApiTypes> =
  AugmentedConst<ApiType>;

declare module "@polkadot/api-base/types/consts" {
  interface AugmentedConsts<ApiType extends ApiTypes> {
    balances: {
      /** The minimum amount required to keep an account open. */
      existentialDeposit: u128 & AugmentedConst<ApiType>;
      /**
       * The maximum number of locks that should exist on an account. Not
       * strictly enforced, but used for weight estimation.
       */
      maxLocks: u32 & AugmentedConst<ApiType>;
      /** The maximum number of named reserves that can exist on an account. */
      maxReserves: u32 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    grandpa: {
      /** Max Authorities in use */
      maxAuthorities: u32 & AugmentedConst<ApiType>;
      /**
       * The maximum number of entries to keep in the set id to session index mapping.
       *
       * Since the `SetIdSession` map is only used for validating equivocations
       * this value should relate to the bonding duration of whatever staking
       * system is being used (if any). If equivocation handling is not enabled
       * then this value can be zero.
       */
      maxSetIdSessionEntries: u64 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    system: {
      /**
       * Maximum number of block number to block hash mappings to keep (oldest
       * pruned first).
       */
      blockHashCount: u32 & AugmentedConst<ApiType>;
      /** The maximum length of a block (in bytes). */
      blockLength: FrameSystemLimitsBlockLength & AugmentedConst<ApiType>;
      /** Block & extrinsics weights: base values and limits. */
      blockWeights: FrameSystemLimitsBlockWeights & AugmentedConst<ApiType>;
      /** The weight of runtime database operations the runtime can invoke. */
      dbWeight: SpWeightsRuntimeDbWeight & AugmentedConst<ApiType>;
      /**
       * The designated SS58 prefix of this chain.
       *
       * This replaces the "ss58Format" property declared in the chain spec.
       * Reason is that the runtime should know about the prefix in order to
       * make use of it as an identifier of the chain.
       */
      ss58Prefix: u16 & AugmentedConst<ApiType>;
      /** Get the chain's current version. */
      version: SpVersionRuntimeVersion & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    timestamp: {
      /**
       * The minimum period between blocks. Beware that this is different to the
       * _expected_ period that the block production apparatus provides. Your
       * chosen consensus system will generally work with this to determine a
       * sensible block time. e.g. For Aura, it will be double this period on
       * default settings.
       */
      minimumPeriod: u64 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    transactionPayment: {
      /**
       * A fee multiplier for `Operational` extrinsic to compute "virtual tip"
       * to boost their `priority`
       *
       * This value is multiplied by the `final_fee` to obtain a "virtual tip"
       * that is later added to a tip component in regular `priority`
       * calculations. It means that a `Normal` transaction can front-run a
       * similarly-sized `Operational` extrinsic (with no tip), by including a
       * tip value greater than the virtual tip.
       *
       * ```rust,ignore
       * // For `Normal`
       * let priority = priority_calc(tip);
       *
       * // For `Operational`
       * let virtual_tip = (inclusion_fee + tip) * OperationalFeeMultiplier;
       * let priority = priority_calc(tip + virtual_tip);
       * ```
       *
       * Note that since we use `final_fee` the multiplier applies also to the
       * regular `tip` sent with the transaction. So, not only does the
       * transaction get a priority bump based on the `inclusion_fee`, but we
       * also amplify the impact of tips applied to `Operational` transactions.
       */
      operationalFeeMultiplier: u8 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
    utility: {
      /** The limit on the number of batched calls. */
      batchedCallsLimit: u32 & AugmentedConst<ApiType>;
      /** Generic const */
      [key: string]: Codec;
    };
  } // AugmentedConsts
} // declare module
