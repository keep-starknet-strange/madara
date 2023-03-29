// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

import type { BTreeMap, Bytes, Enum, Struct, U256, Vec } from '@polkadot/types-codec';
import type { H256 } from '@polkadot/types/interfaces/runtime';

/** @name ContractClassWrapper */
export interface ContractClassWrapper extends Struct {
  readonly program: Bytes;
  readonly entry_points_by_type: BTreeMap<EntryPointTypeWrapper, Vec<EntryPointWrapper>>;
}

/** @name EntryPointTypeWrapper */
export interface EntryPointTypeWrapper extends Enum {
  readonly isConstructor: boolean;
  readonly isExternal: boolean;
  readonly isL1Handler: boolean;
  readonly type: 'Constructor' | 'External' | 'L1Handler';
}

/** @name EntryPointWrapper */
export interface EntryPointWrapper extends Struct {
  readonly entrypoint_selector: H256;
  readonly entrypoint_offset: U256;
}

export type PHANTOM_STARKNET = 'starknet';
