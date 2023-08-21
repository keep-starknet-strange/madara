use alloc::vec::Vec;

use starknet_api::api_core::Nonce;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::TransactionVersion;
use starknet_api::StarknetApiError;
use starknet_ff::FieldElement;

use crate::execution::felt252_wrapper::Felt252Wrapper;
use crate::execution::types::{EntryPointTypeWrapper, EntryPointWrapper};

const QUERY_VERSION_OFFSET: FieldElement =
    FieldElement::from_mont([18446744073700081665, 17407, 18446744073709551584, 576460752142434320]);

/// Estimate fee adds an additional offset to the transaction version
/// when handling Transaction within Madara, we ignore the offset and use the actual version.
/// However, before sending the transaction to the account, we need to add the offset back for
/// signature verification to work
pub fn calculate_transaction_version(is_query: bool, version: TransactionVersion) -> TransactionVersion {
    if !is_query {
        return version;
    }
    let version = FieldElement::from(version.0) + QUERY_VERSION_OFFSET;
    TransactionVersion(StarkFelt::from(version))
}

/// calls [calculate_transaction_version] after converting version to [TransactionVersion]
pub fn calculate_transaction_version_from_u8(is_query: bool, version: u8) -> TransactionVersion {
    calculate_transaction_version(is_query, TransactionVersion(StarkFelt::from(version)))
}

/// converts [Felt252Wrapper] to [Nonce]
pub fn felt_to_nonce(nonce: Felt252Wrapper) -> Result<Nonce, StarknetApiError> {
    Ok(Nonce(StarkFelt::new(nonce.into())?))
}

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
