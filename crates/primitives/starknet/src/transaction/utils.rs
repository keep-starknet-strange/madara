use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(feature = "std")]
use starknet_core::types::{LegacyContractEntryPoint, LegacyEntryPointsByType};

use crate::execution::types::{EntryPointTypeWrapper, EntryPointV0Wrapper};

#[cfg(feature = "std")]
pub mod reexport_std_types {
    use std::collections::HashMap;

    use starknet_core::types::{LegacyContractEntryPoint, LegacyEntryPointsByType};

    use super::*;
    /// Returns a [HashMap<EntryPointTypeWrapper, Vec<EntryPointV0Wrapper>>] from
    /// [LegacyEntryPointsByType]
    pub fn to_hash_map_entrypoints(
        entries: LegacyEntryPointsByType,
    ) -> HashMap<EntryPointTypeWrapper, Vec<EntryPointV0Wrapper>> {
        let mut entry_points_by_type = HashMap::default();

        entry_points_by_type.insert(EntryPointTypeWrapper::Constructor, get_entrypoint_value(entries.constructor));
        entry_points_by_type.insert(EntryPointTypeWrapper::External, get_entrypoint_value(entries.external));
        entry_points_by_type.insert(EntryPointTypeWrapper::L1Handler, get_entrypoint_value(entries.l1_handler));
        entry_points_by_type
    }

    /// Returns a [Vec<EntryPointV0Wrapper>] from a [Vec<LegacyContractEntryPoint>]
    fn get_entrypoint_value(entries: Vec<LegacyContractEntryPoint>) -> Vec<EntryPointV0Wrapper> {
        entries.iter().map(|e| EntryPointV0Wrapper::from(e.clone())).collect::<Vec<_>>()
    }
}

#[cfg(feature = "std")]
fn get_entrypoint_value(entries: Vec<LegacyContractEntryPoint>) -> Vec<EntryPointV0Wrapper> {
    entries.iter().map(|e| EntryPointV0Wrapper::from(e.clone())).collect::<Vec<_>>()
}
