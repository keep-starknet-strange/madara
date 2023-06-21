// Gas Cost.
// See documentation for more details.
/// Gas per step
pub const STEP_GAS_COST: u64 = 100;
/// An estimation of the initial gas for a transaction to run with. This solution is temporary and
/// this value will become a field of the transaction.
pub const INITIAL_GAS_COST: u64 = 10_u64.pow(8) * STEP_GAS_COST;
