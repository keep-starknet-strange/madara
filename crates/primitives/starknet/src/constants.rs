use starknet_ff::FieldElement;

use crate::execution::felt252_wrapper::Felt252Wrapper;

/// Initial gas for a transaction
pub const INITIAL_GAS: u64 = u64::MAX;

// Need to use `from_mont` because this needs to be a constant function call
/// ChainId for Starknet Goerli testnet
pub const SN_GOERLI_CHAIN_ID: Felt252Wrapper = Felt252Wrapper(FieldElement::from_mont([
    3753493103916128178,
    18446744073709548950,
    18446744073709551615,
    398700013197595345,
]));
