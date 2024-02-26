//! Starknet fee logic
#![cfg_attr(not(feature = "std"), no_std)]

#[doc(hidden)]
pub extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::collections::HashMap;

use blockifier::abi::constants::GAS_USAGE;
use blockifier::block_context::BlockContext;
use blockifier::execution::entry_point::{
    CallEntryPoint, CallInfo, CallType, EntryPointExecutionContext, ExecutionResources,
};
use blockifier::state::state_api::State;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::{AccountTransactionContext, ResourcesMapping, TransactionExecutionResult};
use blockifier::transaction::transaction_types::TransactionType;
use blockifier::transaction::transaction_utils::{calculate_l1_gas_usage, calculate_tx_resources};
#[cfg(not(feature = "std"))]
use hashbrown::HashMap;
use mp_state::StateChanges;
use sp_arithmetic::fixed_point::{FixedPointNumber, FixedU128};
use sp_arithmetic::traits::Zero;
use starknet_api::api_core::EntryPointSelector;
use starknet_api::calldata;
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, Fee};
use starknet_core::types::ResourcePrice as CoreResourcePrice;

/// Initial gas for a transaction
pub const INITIAL_GAS: u64 = u64::MAX;
/// Number of storage updates for the fee transfer tx.
pub const FEE_TRANSFER_N_STORAGE_CHANGES: u8 = 2; // Sender and sequencer balance update.
/// Number of storage updates to actually charge for the fee transfer tx.
pub const FEE_TRANSFER_N_STORAGE_CHANGES_TO_CHARGE: u8 = FEE_TRANSFER_N_STORAGE_CHANGES - 1; // Exclude the sequencer balance update, since it's charged once throughout the batch.

pub static VM_RESOURCE_FEE_COSTS: [(&str, FixedU128); 8] = [
    ("n_steps", FixedU128::from_inner(5_000_000_000_000_000)),
    ("pedersen_builtin", FixedU128::from_inner(160_000_000_000_000_000)),
    ("range_check_builtin", FixedU128::from_inner(80_000_000_000_000_000)),
    ("ecdsa_builtin", FixedU128::from_inner(10_240_000_000_000_000_000)),
    ("bitwise_builtin", FixedU128::from_inner(320_000_000_000_000_000)),
    ("poseidon_builtin", FixedU128::from_inner(160_000_000_000_000_000)),
    ("ec_op_builtin", FixedU128::from_inner(5_120_000_000_000_000_000)),
    ("keccak_builtin", FixedU128::from_inner(5_120_000_000_000_000_000)),
];

pub const TRANSFER_SELECTOR_NAME: &str = "Transfer";
pub const TRANSFER_SELECTOR_HASH: [u8; 32] = [
    0, 131, 175, 211, 244, 202, 237, 198, 238, 191, 68, 36, 111, 229, 78, 56, 201, 94, 49, 121, 165, 236, 158, 168, 23,
    64, 236, 165, 180, 130, 209, 46,
]; // starknet_keccak(TRANSFER_SELECTOR_NAME.as_bytes()).to_le_bytes();

#[serde_with::serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResourcePrice {
    /// The price of one unit of the given resource, denominated in fri (10^-18 strk)
    pub price_in_strk: Option<u64>,
    /// The price of one unit of the given resource, denominated in wei
    pub price_in_wei: u128,
}

impl From<ResourcePrice> for CoreResourcePrice {
    fn from(item: ResourcePrice) -> Self {
        // TODO: when we rebase starknet-rs those field type will be FieldElements
        // Get rid of the type conversions
        CoreResourcePrice { price_in_strk: item.price_in_strk, price_in_wei: item.price_in_wei as u64 }
    }
}

/// Gets the transaction resources.
pub fn compute_transaction_resources<S: State + StateChanges>(
    state: &S,
    execute_call_info: &Option<CallInfo>,
    validate_call_info: &Option<CallInfo>,
    execution_resources: &ExecutionResources,
    tx_type: TransactionType,
    l1_handler_payload_size: Option<usize>,
) -> TransactionExecutionResult<ResourcesMapping> {
    let state_changes_count = state.count_state_changes();
    let non_optional_call_infos: Vec<&CallInfo> =
        vec![execute_call_info, validate_call_info].into_iter().flatten().collect();

    let l1_gas_usage = calculate_l1_gas_usage(&non_optional_call_infos, state_changes_count, l1_handler_payload_size)?;
    let actual_resources = calculate_tx_resources(execution_resources, l1_gas_usage, tx_type)?;

    Ok(actual_resources)
}

/// Charges the fees for a specific execution resources.
pub fn charge_fee<S: State + StateChanges>(
    state: &mut S,
    block_context: &BlockContext,
    account_tx_context: AccountTransactionContext,
    resources: &ResourcesMapping,
    disable_transaction_fee: bool,
    disable_fee_charge: bool,
    is_query: bool,
) -> TransactionExecutionResult<(Fee, Option<CallInfo>)> {
    // disable_transaction_fee flag implies that transaction fees have
    // been disabled and so we return 0 as the fees
    if disable_transaction_fee {
        return Ok((Fee(0), None));
    }

    let actual_fee = calculate_tx_fee(resources, block_context)?;

    // Fee charging is skipped in the following cases:
    //  1) if is_query is true, it's an estimate fee transaction, so we don't charge fees
    //  2) The disable_fee_charge flag is set
    // in both cases we return the actual fee.
    if disable_fee_charge || is_query {
        return Ok((actual_fee, None));
    }

    let fee_transfer_call_info = execute_fee_transfer(state, block_context, account_tx_context, actual_fee)?;

    Ok((actual_fee, Some(fee_transfer_call_info)))
}

/// Executes the fee transfer tx
fn execute_fee_transfer(
    state: &mut dyn State,
    block_context: &BlockContext,
    account_tx_context: AccountTransactionContext,
    actual_fee: Fee,
) -> TransactionExecutionResult<CallInfo> {
    let max_fee = account_tx_context.max_fee;
    if actual_fee > max_fee {
        return Err(TransactionExecutionError::FeeTransferError { max_fee, actual_fee });
    }
    // TODO: This is what's done in the blockifier but this should be improved.
    // FIXME: https://github.com/keep-starknet-strange/madara/issues/332
    // The least significant 128 bits of the amount transferred.
    let lsb_amount = StarkFelt::from(actual_fee.0);
    // The most significant 128 bits of the amount transferred.
    let msb_amount = StarkFelt::from(0_u64);
    let storage_address = block_context.fee_token_address;
    let fee_transfer_call = CallEntryPoint {
        class_hash: None,
        code_address: None,
        entry_point_type: EntryPointType::External,
        // The value TRANSFER_SELECTOR_HASH is hardcoded and it's the encoding of the "transfer" selector so it cannot
        // fail.
        entry_point_selector: EntryPointSelector(StarkFelt::new(TRANSFER_SELECTOR_HASH).unwrap()),
        calldata: calldata![
            *block_context.sequencer_address.0.key(), // Recipient.
            lsb_amount,
            msb_amount
        ],
        storage_address,
        caller_address: account_tx_context.sender_address,
        call_type: CallType::Call,
        // The fee-token contract is a Cairo 0 contract, hence the initial gas is irrelevant.
        initial_gas: INITIAL_GAS,
    };

    let max_steps = block_context.invoke_tx_max_n_steps;
    let mut context = EntryPointExecutionContext::new(block_context.clone(), account_tx_context, max_steps);

    Ok(fee_transfer_call.execute(state, &mut ExecutionResources::default(), &mut context)?)
}

/// Computes the fees from the execution resources.
pub fn calculate_tx_fee(resources: &ResourcesMapping, block_context: &BlockContext) -> TransactionExecutionResult<Fee> {
    let (l1_gas_usage, vm_resources) = extract_l1_gas_and_vm_usage(resources);
    let l1_gas_by_vm_usage = calculate_l1_gas_by_vm_usage(block_context, &vm_resources)?;

    let total_l1_gas_usage = FixedU128::checked_from_integer(l1_gas_usage as u128)
        .ok_or(TransactionExecutionError::FixedPointConversion)?
        + l1_gas_by_vm_usage;
    let tx_fee = total_l1_gas_usage
        .ceil()
        .checked_mul_int(block_context.gas_price)
        .ok_or(TransactionExecutionError::FixedPointConversion)?;

    Ok(Fee(tx_fee))
}

/// Computes the fees for l1 gas usage and the vm usage from the execution resources.
///
/// # Arguments
///
/// * `resources` - Execution resources to compute the fees from.
///
/// # Returns
///
/// [usize] - l1 gas usage.
/// [BTreeMap<String, usize>] - vm resources usage.
pub fn extract_l1_gas_and_vm_usage(resources: &ResourcesMapping) -> (usize, ResourcesMapping) {
    let mut vm_resource_usage = resources.0.clone();
    let l1_gas_usage =
        vm_resource_usage.remove(GAS_USAGE).expect("`ResourcesMapping` does not have the key `l1_gas_usage`.");

    (l1_gas_usage as usize, ResourcesMapping(vm_resource_usage))
}

/// Calculates the L1 gas consumed when submitting the underlying Cairo program to SHARP.
/// I.e., returns the heaviest Cairo resource weight (in terms of L1 gas), as the size of
/// a proof is determined similarly - by the (normalized) largest segment.
pub fn calculate_l1_gas_by_vm_usage(
    _block_context: &BlockContext,
    vm_resource_usage: &ResourcesMapping,
) -> TransactionExecutionResult<FixedU128> {
    let vm_resource_fee_costs: HashMap<&str, FixedU128> = HashMap::from(VM_RESOURCE_FEE_COSTS);
    // Check if keys in vm_resource_usage are a subset of keys in VM_RESOURCE_FEE_COSTS
    if vm_resource_usage.0.keys().any(|key| !vm_resource_fee_costs.contains_key(key.as_str())) {
        return Err(TransactionExecutionError::CairoResourcesNotContainedInFeeCosts);
    };

    // Convert Cairo usage to L1 gas usage.
    let vm_l1_gas_usage = vm_resource_usage
        .0
        .iter()
        .map(|(key, &value)| {
            let value = <FixedU128 as FixedPointNumber>::checked_from_integer(value as u128)
                .ok_or(TransactionExecutionError::FixedPointConversion);

            value.map(|v| vm_resource_fee_costs.get(key.as_str()).unwrap().mul(v))
        })
        .try_fold(FixedU128::zero(), |accum, res| res.map(|v| v.max(accum)))?;

    Ok(vm_l1_gas_usage)
}

#[cfg(test)]
mod vm_resource_fee_costs {
    use super::{FixedU128, HashMap, VM_RESOURCE_FEE_COSTS};

    #[test]
    fn check_values_as_floats() {
        let hm = HashMap::from(VM_RESOURCE_FEE_COSTS);

        assert_eq!(hm.get("n_steps"), Some(FixedU128::from_float(0.005)).as_ref());
        assert_eq!(hm.get("pedersen_builtin"), Some(FixedU128::from_float(0.16)).as_ref());
        assert_eq!(hm.get("range_check_builtin"), Some(FixedU128::from_float(0.08)).as_ref());
        assert_eq!(hm.get("ecdsa_builtin"), Some(FixedU128::from_float(10.24)).as_ref());
        assert_eq!(hm.get("bitwise_builtin"), Some(FixedU128::from_float(0.32)).as_ref());
        assert_eq!(hm.get("poseidon_builtin"), Some(FixedU128::from_float(0.16)).as_ref());
        assert_eq!(hm.get("ec_op_builtin"), Some(FixedU128::from_float(5.12)).as_ref());
        assert_eq!(hm.get("keccak_builtin"), Some(FixedU128::from_float(5.12)).as_ref());
    }
}
