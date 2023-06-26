use alloc::vec::Vec;

use crate::execution::types::{EntryPointTypeWrapper, EntryPointWrapper};

#[cfg(feature = "std")]
mod reexport_std_types {
    use std::collections::HashMap;

    use starknet_core::types::{LegacyContractEntryPoint, LegacyEntryPointsByType};

    use super::*;
    /// Returns a [HashMap<EntryPointTypeWrapper, Vec<EntryPointWrapper>>] from
    /// [LegacyEntryPointsByType]
    pub fn to_hash_map_entrypoints(
        entries: LegacyEntryPointsByType,
    ) -> HashMap<EntryPointTypeWrapper, Vec<EntryPointWrapper>> {
        let mut entry_points_by_type = HashMap::default();

        entry_points_by_type.insert(EntryPointTypeWrapper::Constructor, get_entrypoint_value(entries.constructor));
        entry_points_by_type.insert(EntryPointTypeWrapper::External, get_entrypoint_value(entries.external));
        entry_points_by_type.insert(EntryPointTypeWrapper::L1Handler, get_entrypoint_value(entries.l1_handler));
        entry_points_by_type
    }

    /// Returns a [Vec<EntryPointWrapper>] from a [Vec<LegacyContractEntryPoint>]
    fn get_entrypoint_value(entries: Vec<LegacyContractEntryPoint>) -> Vec<EntryPointWrapper> {
        entries.iter().map(|e| EntryPointWrapper::from(e.clone())).collect::<Vec<_>>()
    }
}

#[cfg(feature = "std")]
pub use reexport_std_types::*;
