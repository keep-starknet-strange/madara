//! Traits for chain ids.
#![cfg_attr(not(feature = "std"), no_std)]

use mp_felt::Felt252Wrapper;

/// ChainId for Starknet Goerli testnet
pub const SN_GOERLI_CHAIN_ID: Felt252Wrapper = Felt252Wrapper(starknet_ff::FieldElement::from_mont([
    3753493103916128178,
    18446744073709548950,
    18446744073709551615,
    398700013197595345,
]));

/// ChainId for Starknet Mainnet
pub const SN_MAIN_CHAIN_ID: Felt252Wrapper = Felt252Wrapper(starknet_ff::FieldElement::from_mont([
    17696389056366564951,
    18446744073709551615,
    18446744073709551615,
    502562008147966918,
]));

/// ChainId for a custom Madara chain
pub const MADARA_CHAIN_ID: Felt252Wrapper = Felt252Wrapper(starknet_ff::FieldElement::from_mont([
    18444025906882525153,
    18446744073709551615,
    18446744073709551615,
    530251916243973616,
]));

#[cfg(test)]
mod tests;
