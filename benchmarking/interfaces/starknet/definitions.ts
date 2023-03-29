/* eslint-disable @typescript-eslint/camelcase */

export default {
  types: {
    ContractClassWrapper: {
      program: "BoundedVec<u8, MaxProgramSize>",
      entry_points_by_type:
        "BTreeMap<EntryPointTypeWrapper, BoundedVec<EntryPointWrapper, MaxEntryPoints>>",
    },
    EntryPointTypeWrapper: {
      _enum: ["Constructor", "External", "L1Handler"],
    },
    EntryPointWrapper: {
      entrypoint_selector: "H256",
      entrypoint_offset: "U256",
    },
  },
};
