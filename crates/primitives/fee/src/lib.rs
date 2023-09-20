//! Starknet fee logic
#![cfg_attr(not(feature = "std"), no_std)]

#[doc(hidden)]
pub extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use blockifier::abi::constants::GAS_USAGE;
use blockifier::block_context::BlockContext;
use blockifier::execution::entry_point::{
    CallEntryPoint, CallInfo, CallType, EntryPointExecutionContext, ExecutionResources,
};
use blockifier::state::cached_state::StateChangesCount;
use blockifier::state::state_api::State;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::{AccountTransactionContext, ResourcesMapping, TransactionExecutionResult};
use blockifier::transaction::transaction_types::TransactionType;
use blockifier::transaction::transaction_utils::{calculate_l1_gas_usage, calculate_tx_resources};
use mp_state::{FeeConfig, StateChanges};
use phf::phf_map;
use starknet_api::api_core::EntryPointSelector;
use starknet_api::calldata;
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, Fee};

/// Initial gas for a transaction
pub const INITIAL_GAS: u64 = u64::MAX;
/// Number of storage updates for the fee transfer tx.
pub const FEE_TRANSFER_N_STORAGE_CHANGES: u8 = 2; // Sender and sequencer balance update.
/// Number of storage updates to actually charge for the fee transfer tx.
pub const FEE_TRANSFER_N_STORAGE_CHANGES_TO_CHARGE: u8 = FEE_TRANSFER_N_STORAGE_CHANGES - 1; // Exclude the sequencer balance update, since it's charged once throughout the batch.

// TODO: add real values here.
// FIXME: https://github.com/keep-starknet-strange/madara/issues/330
static VM_RESOURCE_FEE_COSTS: phf::Map<&'static str, f64> = phf_map! {
    "n_steps" => 1_f64,
    "pedersen_builtin" => 1_f64,
    "range_check_builtin" => 1_f64,
    "ecdsa_builtin" => 1_f64,
    "bitwise_builtin" => 1_f64,
    "poseidon_builtin" => 1_f64,
    "output_builtin" => 1_f64,
    "ec_op_builtin" => 1_f64,
};

/// Gets the transaction resources.
pub fn compute_transaction_resources<S: State + StateChanges>(
    state: &S,
    execute_call_info: &Option<CallInfo>,
    validate_call_info: &Option<CallInfo>,
    execution_resources: &ExecutionResources,
    tx_type: TransactionType,
    l1_handler_payload_size: Option<usize>,
) -> TransactionExecutionResult<ResourcesMapping> {
    let (n_modified_contracts, n_storage_updates, n_class_hash_updates, n_compiled_class_hash_updates) =
        state.count_state_changes();
    let state_changes_count = StateChangesCount {
        n_storage_updates,
        n_class_hash_updates,
        n_compiled_class_hash_updates,
        n_modified_contracts,
    };
    let non_optional_call_infos: Vec<&CallInfo> =
        vec![execute_call_info, validate_call_info].into_iter().flatten().collect();

    let l1_gas_usage = calculate_l1_gas_usage(&non_optional_call_infos, state_changes_count, l1_handler_payload_size)?;
    let actual_resources = calculate_tx_resources(execution_resources, l1_gas_usage, tx_type)?;

    Ok(actual_resources)
}

/// Charges the fees for a specific execution resources.
pub fn charge_fee<S: State + StateChanges + FeeConfig>(
    state: &mut S,
    block_context: &BlockContext,
    account_tx_context: AccountTransactionContext,
    resources: &ResourcesMapping,
) -> TransactionExecutionResult<(Fee, Option<CallInfo>)> {
    if state.is_transaction_fee_disabled() {
        return Ok((Fee(0), None));
    }

    let actual_fee = calculate_tx_fee(resources, block_context)?;

    // even if the user doesn't have enough balance
    // estimate fee shouldn't fail
    if account_tx_context.version.0 >= StarkFelt::try_from("0x100000000000000000000000000000000").unwrap() {
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
        entry_point_selector: EntryPointSelector(
            // The value is hardcoded and it's the encoding of the "transfer" selector so it cannot fail.
            StarkFelt::new([
                0, 131, 175, 211, 244, 202, 237, 198, 238, 191, 68, 36, 111, 229, 78, 56, 201, 94, 49, 121, 165, 236,
                158, 168, 23, 64, 236, 165, 180, 130, 209, 46,
            ])
            .unwrap(),
        ),
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
    let mut context = EntryPointExecutionContext::new(block_context.clone(), account_tx_context, max_steps as usize);

    Ok(fee_transfer_call.execute(state, &mut ExecutionResources::default(), &mut context)?)
}

/// Computes the fees from the execution resources.
pub fn calculate_tx_fee(resources: &ResourcesMapping, block_context: &BlockContext) -> TransactionExecutionResult<Fee> {
    let (l1_gas_usage, vm_resources) = extract_l1_gas_and_vm_usage(resources);
    let l1_gas_by_vm_usage = calculate_l1_gas_by_vm_usage(block_context, &vm_resources)?;
    let total_l1_gas_usage = l1_gas_usage as f64 + l1_gas_by_vm_usage;
    // Ceil is in the std lib so we can't use it sadly.
    let total_l1_gas_usage = if total_l1_gas_usage - total_l1_gas_usage as u128 as f64 > 0.0 {
        total_l1_gas_usage as u128 + 1
    } else {
        total_l1_gas_usage as u128
    };
    Ok(Fee(total_l1_gas_usage * block_context.gas_price))
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

    (l1_gas_usage, ResourcesMapping(vm_resource_usage))
}

/// Calculates the L1 gas consumed when submitting the underlying Cairo program to SHARP.
/// I.e., returns the heaviest Cairo resource weight (in terms of L1 gas), as the size of
/// a proof is determined similarly - by the (normalized) largest segment.
pub fn calculate_l1_gas_by_vm_usage(
    _block_context: &BlockContext,
    vm_resource_usage: &ResourcesMapping,
) -> TransactionExecutionResult<f64> {
    // Check if keys in vm_resource_usage are a subset of keys in VM_RESOURCE_FEE_COSTS
    if vm_resource_usage.0.keys().any(|key| !VM_RESOURCE_FEE_COSTS.contains_key(key.as_str())) {
        return Err(TransactionExecutionError::CairoResourcesNotContainedInFeeCosts);
    };

    // Convert Cairo usage to L1 gas usage.
    let vm_l1_gas_usage: f64 = vm_resource_usage
        .0
        .iter()
        .map(|(key, &value)| VM_RESOURCE_FEE_COSTS.get(key.as_str()).unwrap() * value as f64)
        .fold(f64::NAN, f64::max);

    Ok(vm_l1_gas_usage)
}
