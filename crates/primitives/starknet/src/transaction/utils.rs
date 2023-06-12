use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use frame_support::BoundedVec;
#[cfg(feature = "std")]
use starknet_core::types::{LegacyContractEntryPoint, LegacyEntryPointsByType};

use crate::execution::types::{EntryPointTypeWrapper, EntryPointWrapper, MaxEntryPoints};

/// Returns a btree map of entry point types to entrypoint from deprecated entry point by type
#[cfg(feature = "std")]
pub fn to_btree_map_entrypoints(
    entries: LegacyEntryPointsByType,
) -> BTreeMap<EntryPointTypeWrapper, BoundedVec<EntryPointWrapper, MaxEntryPoints>> {
    let mut entry_points_by_type: BTreeMap<EntryPointTypeWrapper, BoundedVec<EntryPointWrapper, MaxEntryPoints>> =
        BTreeMap::new();

    entry_points_by_type.insert(EntryPointTypeWrapper::Constructor, get_entrypoint_value(entries.constructor));
    entry_points_by_type.insert(EntryPointTypeWrapper::External, get_entrypoint_value(entries.external));
    entry_points_by_type.insert(EntryPointTypeWrapper::L1Handler, get_entrypoint_value(entries.l1_handler));
    entry_points_by_type
}

/// Returns a bounded vector of `EntryPointWrapper` from a vector of LegacyContractEntryPoint
#[cfg(feature = "std")]
fn get_entrypoint_value(entries: Vec<LegacyContractEntryPoint>) -> BoundedVec<EntryPointWrapper, MaxEntryPoints> {
    // We can unwrap safely as we already checked the length of the vectors
    BoundedVec::try_from(entries.iter().map(|e| EntryPointWrapper::from(e.clone())).collect::<Vec<_>>()).unwrap()
}
