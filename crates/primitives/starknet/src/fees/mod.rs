use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use blockifier::abi::constants::{GAS_USAGE, N_STEPS_RESOURCE};
use blockifier::block_context::BlockContext;
use blockifier::execution::entry_point::{
    CallEntryPoint, CallInfo, CallType, EntryPointExecutionContext, ExecutionResources,
};
use blockifier::fee::gas_usage::calculate_tx_gas_usage;
use blockifier::fee::os_usage::get_additional_os_resources;
use blockifier::state::state_api::State;
use blockifier::transaction::objects::AccountTransactionContext;
use starknet_api::api_core::EntryPointSelector;
use starknet_api::calldata;
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, Fee};

use super::state::StateChanges;
use crate::alloc::string::ToString;
use crate::transaction::types::{TransactionExecutionErrorWrapper, TxType};

/// Number of storage updates for the fee transfer tx.
pub const FEE_TRANSFER_N_STORAGE_CHANGES: u8 = 2; // Sender and sequencer balance update.
/// Number of storage updates to actually charge for the fee transfer tx.
pub const FEE_TRANSFER_N_STORAGE_CHANGES_TO_CHARGE: u8 = FEE_TRANSFER_N_STORAGE_CHANGES - 1; // Exclude the sequencer balance update, since it's charged once throughout the batch.

/// Gets the transaction resources.
///
/// # Arguments
///
/// * `state` - State object to get the state changes.
/// * `execute_call_info` - Call info of the execution of the `__execute__` entrypoint.
/// * `execution_resources` - Resources used by the execution.
/// * `tx_type` - Type of the transaction.
///
/// # Returns
///
/// * [BTreeMap<String, usize>] - Mapping from execution resources to the number of uses.
///
/// # Error
///
/// [TransactionExecutionErrorWrapper] if a step of the execution resources computation fails.
pub fn get_transaction_resources<S: State + StateChanges>(
    state: &mut S,
    execute_call_info: &Option<CallInfo>,
    validate_call_info: &Option<CallInfo>,
    execution_resources: &mut ExecutionResources,
    tx_type: TxType,
) -> Result<BTreeMap<String, usize>, TransactionExecutionErrorWrapper> {
    let (n_modified_contracts, n_modified_keys, n_class_updates) = state.count_state_changes();
    let non_optional_call_infos: Vec<&CallInfo> =
        vec![execute_call_info, validate_call_info].into_iter().flatten().collect();
    let mut l2_to_l1_payloads_length = vec![];
    for call_info in non_optional_call_infos {
        l2_to_l1_payloads_length.extend(
            call_info
                .get_sorted_l2_to_l1_payloads_length()
                .map_err(|err| TransactionExecutionErrorWrapper::UnexpectedHoles(err.to_string()))?,
        );
    }
    let l1_gas_usage = calculate_tx_gas_usage(
        &l2_to_l1_payloads_length,
        n_modified_contracts,
        n_modified_keys + usize::from(FEE_TRANSFER_N_STORAGE_CHANGES_TO_CHARGE),
        None,
        n_class_updates,
    );
    // Add additional Cairo resources needed for the OS to run the transaction.
    let total_vm_usage = &execution_resources.vm_resources
        + &get_additional_os_resources(execution_resources.syscall_counter.clone(), tx_type.into())
            .map_err(|_| TransactionExecutionErrorWrapper::FeeComputationError)?;
    let total_vm_usage = total_vm_usage.filter_unused_builtins();
    let mut tx_resources = BTreeMap::from([
        (GAS_USAGE.to_string(), l1_gas_usage),
        (N_STEPS_RESOURCE.to_string(), total_vm_usage.n_steps + total_vm_usage.n_memory_holes),
    ]);
    tx_resources.extend(total_vm_usage.builtin_instance_counter);
    Ok(tx_resources)
}

/// Charges the fees for a specific execution resources.
///
/// # Arguments
///
/// * `state` - State object to get the state changes.
/// * `block_context` - Block context to get information needed to compute the fees.
/// * `account_tx_context` - Account context.
/// * `resources` - Execution resources.
///
/// # Returns
///
/// * [Fee] - Amount charged for the transaction.
/// * [`Option<CallInfo>`] - Call info of the fee transfer tx.
///
/// # Errors
///
/// [TransactionExecutionErrorWrapper] if any step of the fee transfer computation/transaction
/// fails.
pub fn charge_fee<S: State + StateChanges>(
    state: &mut S,
    block_context: &BlockContext,
    account_tx_context: AccountTransactionContext,
    resources: &BTreeMap<String, usize>,
) -> Result<(Fee, Option<CallInfo>), TransactionExecutionErrorWrapper> {
    let no_fee = Fee::default();
    if account_tx_context.max_fee == no_fee {
        // Fee charging is not enforced in some tests.
        return Ok((no_fee, None));
    }

    let actual_fee = calculate_tx_fee(resources, block_context)
        .map_err(|_| TransactionExecutionErrorWrapper::FeeComputationError)?;
    let fee_transfer_call_info = execute_fee_transfer(state, block_context, account_tx_context, actual_fee)?;

    Ok((actual_fee, Some(fee_transfer_call_info)))
}

/// Executes the fee transfer tx
fn execute_fee_transfer(
    state: &mut dyn State,
    block_context: &BlockContext,
    account_tx_context: AccountTransactionContext,
    actual_fee: Fee,
) -> Result<CallInfo, TransactionExecutionErrorWrapper> {
    // TODO: use real value.
    // FIXME: https://github.com/keep-starknet-strange/madara/issues/331
    let max_fee = Fee(u128::MAX);
    if actual_fee > max_fee {
        return Err(TransactionExecutionErrorWrapper::FeeTransferError { max_fee, actual_fee });
    }
    // TODO: This is what's done in the blockifier but this should be improved.
    // FIXME: https://github.com/keep-starknet-strange/madara/issues/332
    // The least significant 128 bits of the amount transferred.
    let lsb_amount = StarkFelt::from(actual_fee.0 as u64);
    // The most significant 128 bits of the amount transferred.
    let msb_amount = StarkFelt::from(0_u64);

    // The fee-token contract is a Cairo 0 contract, hence the initial gas is irrelevant.
    let initial_gas = super::constants::INITIAL_GAS_COST.into();

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
        initial_gas,
    };

    let max_steps = block_context.invoke_tx_max_n_steps;
    let mut context = EntryPointExecutionContext::new(block_context.clone(), account_tx_context, max_steps);

    fee_transfer_call
        .execute(state, &mut ExecutionResources::default(), &mut context)
        .map_err(TransactionExecutionErrorWrapper::EntrypointExecution)
}

/// Computes the fees from the execution resources.
///
/// # Arguments
///
/// * `resources` - Execution resources to compute the fees from.
/// * `block_context` - Block context to get information needed to compute the fees.
///
/// # Returns
///
/// [Fee] - the fees computed for the transaction.
///
/// # Error
///
/// [TransactionExecutionErrorWrapper] - if the computation of the l1 gas usage fails, returns an
/// error.
pub fn calculate_tx_fee(
    resources: &BTreeMap<String, usize>,
    block_context: &BlockContext,
) -> Result<Fee, TransactionExecutionErrorWrapper> {
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
pub fn extract_l1_gas_and_vm_usage(resources: &BTreeMap<String, usize>) -> (usize, BTreeMap<String, usize>) {
    let mut vm_resource_usage = resources.clone();
    let l1_gas_usage =
        vm_resource_usage.remove(GAS_USAGE).expect("`ResourcesMapping` does not have the key `l1_gas_usage`.");

    (l1_gas_usage, vm_resource_usage)
}

/// Calculates the L1 gas consumed when submitting the underlying Cairo program to SHARP.
/// I.e., returns the heaviest Cairo resource weight (in terms of L1 gas), as the size of
/// a proof is determined similarly - by the (normalized) largest segment.
pub fn calculate_l1_gas_by_vm_usage(
    _block_context: &BlockContext,
    vm_resource_usage: &BTreeMap<String, usize>,
) -> Result<f64, TransactionExecutionErrorWrapper> {
    // TODO: add real values here.
    // FIXME: https://github.com/keep-starknet-strange/madara/issues/330
    let vm_resource_fee_costs = BTreeMap::from([
        (String::from("n_steps"), 1_f64),
        (String::from("pedersen_builtin"), 1_f64),
        (String::from("range_check_builtin"), 1_f64),
        (String::from("ecdsa_builtin"), 1_f64),
        (String::from("bitwise_builtin"), 1_f64),
        (String::from("poseidon_builtin"), 1_f64),
        (String::from("output_builtin"), 1_f64),
        (String::from("ec_op_builtin"), 1_f64),
    ]);
    let vm_resource_names = BTreeSet::<&String>::from_iter(vm_resource_usage.keys());

    if !vm_resource_names.is_subset(&BTreeSet::from_iter(vm_resource_fee_costs.keys())) {
        return Err(TransactionExecutionErrorWrapper::FailedToComputeL1GasUsage);
    };

    // Convert Cairo usage to L1 gas usage.
    let vm_l1_gas_usage = vm_resource_fee_costs
        .iter()
        .map(|(key, resource_val)| (*resource_val) * vm_resource_usage.get(key).cloned().unwrap_or_default() as f64)
        .fold(f64::NAN, f64::max);

    Ok(vm_l1_gas_usage)
}
