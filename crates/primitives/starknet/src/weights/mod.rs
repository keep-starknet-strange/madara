use frame_support::weights::constants::WEIGHT_REF_TIME_PER_MILLIS;
use frame_support::weights::Weight;
use sp_runtime::Perbill;

/// `WeightPerStep` is an approximate ratio of the amount of Weight per Cairo Step.
/// u64 works for approximations because Weight is a very small unit compared to steps.
///
/// The formula for calculating the Weight per Step described below was inspired from here:
/// https://github.com/paritytech/frontier/blob/aab7cfc3e038ec65ca4d1b2bc3d68e51aa162514/primitives/evm/src/lib.rs#L221
///
/// Important to keep in mind the following points
/// 1. WEIGHT_MILLIS_PER_BLOCK = execution time of a block => usually 1/3 of block time
/// 2. TXN_RATIO = % of execution time used for evaluating extrinsics i.e. NORMAL_DISPATCH_RATIO
/// 3. STEPS_PER_MILLIS = Steps consumed per millisecond of execution time. Can be obtained by benchmarking
///
/// `STEPS_PER_MILLIS * WEIGHT_MILLIS_PER_BLOCK * TXN_RATIO ~= BLOCK_STEP_LIMIT`
/// `WEIGHT_PER_STEP = WEIGHT_REF_TIME_PER_MILLIS / STEPS_PER_MILLIS
///                 = WEIGHT_REF_TIME_PER_MILLIS / (BLOCK_STEP_LIMIT / TXN_RATIO / WEIGHT_MILLIS_PER_BLOCK)
/// 				= TXN_RATIO * (WEIGHT_REF_TIME_PER_MILLIS * WEIGHT_MILLIS_PER_BLOCK) / BLOCK_STEP_LIMIT`
///
/// For example, given the 2000ms Weight, from which 75% only are used for transactions,
/// the total Starknet execution step limit is `STEPS_PER_MILLIS * 2000 * 75% = BLOCK_STEP_LIMIT`.
pub fn weight_per_step(block_step_limit: u64, txn_ratio: Perbill, weight_millis_per_block: u64) -> u64 {
    let weight_per_block = WEIGHT_REF_TIME_PER_MILLIS.saturating_mul(weight_millis_per_block);
    let weight_per_step = (txn_ratio * weight_per_block).saturating_div(block_step_limit);
    assert!(weight_per_step >= 1, "WeightPerStep must greater than or equal with 1");
    weight_per_step
}

/// A mapping function that converts Starknet steps to Substrate weight
pub trait StepWeightMapping {
    fn steps_to_weight(steps: u32, without_base_weight: bool) -> Weight;
    fn weight_to_gas(weight: Weight) -> u64;
}
