use starknet_ff::FieldElement;

use crate::execution::felt252_wrapper::Felt252Wrapper;

// Gas Cost.
// See documentation for more details.
/// Gas per step
pub const STEP_GAS_COST: u64 = 100;
/// An estimation of the initial gas for a transaction to run with. This solution is temporary and
/// this value will become a field of the transaction.
pub const INITIAL_GAS_COST: u64 = 10_u64.pow(8) * STEP_GAS_COST;

// Need to use `from_mont` because this needs to be a constant function call
/// ChainId for Starknet Goerli testnet
pub const SN_GOERLI_CHAIN_ID: Felt252Wrapper = Felt252Wrapper(FieldElement::from_mont([
    3753493103916128178,
    18446744073709548950,
    18446744073709551615,
    398700013197595345,
]));
