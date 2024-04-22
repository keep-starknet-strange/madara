use std::collections::HashSet;
use std::sync::Arc;

use blockifier::abi::abi_utils::selector_from_name;
use blockifier::context::{BlockContext, TransactionContext};
use blockifier::execution::call_info::{CallInfo, Retdata};
use blockifier::execution::contract_class::ContractClass;
use blockifier::execution::entry_point::{CallEntryPoint, CallType, EntryPointExecutionContext};
use blockifier::fee::actual_cost::{ActualCost, ActualCostBuilder};
use blockifier::fee::fee_checks::{FeeCheckReportFields, PostExecutionReport};
use blockifier::state::cached_state::{CachedState, GlobalContractCache, StateChangesCount};
use blockifier::state::errors::StateError;
use blockifier::state::state_api::{State, StateReader, StateResult};
use blockifier::transaction::account_transaction::{AccountTransaction, ValidateExecuteCallInfo};
use blockifier::transaction::errors::{TransactionExecutionError, TransactionFeeError, TransactionPreValidationError};
use blockifier::transaction::objects::{
    GasVector, HasRelatedFeeType, ResourcesMapping, TransactionExecutionInfo, TransactionExecutionResult,
    TransactionInfo, TransactionInfoCreator,
};
use blockifier::transaction::transaction_types::TransactionType;
use blockifier::transaction::transactions::{
    DeclareTransaction, DeployAccountTransaction, Executable, InvokeTransaction, L1HandlerTransaction,
};
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use indexmap::IndexMap;
use mp_felt::Felt252Wrapper;
use starknet_api::core::{ClassHash, CompiledClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;
use starknet_api::transaction::{Calldata, Fee, ResourceBounds, TransactionVersion};

use super::SIMULATE_TX_VERSION_OFFSET;

/// Contains the execution configuration regarding transaction fee
/// activation, transaction fee charging, nonce validation, transaction
/// validation and the execution mode (query or not).
/// Use [`RuntimeExecutionConfigBuilder`] to build this struct in the runtime.
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// If true, the transaction is a query.
    pub is_query: bool,
    /// If true, the transaction version has the simulation offset.
    pub offset_version: bool,
    /// If true, transaction fee calculation and charging
    /// is disabled for all transactions.
    pub disable_transaction_fee: bool,
    /// If true, fee charging is disabled for all transactions.
    pub disable_fee_charge: bool,
    /// If true, nonce validation is disabled for all transactions.
    pub disable_nonce_validation: bool,
    /// If true, transaction validation is disabled for all transactions.
    pub disable_validation: bool,
}

impl ExecutionConfig {
    pub fn set_offset_version(&mut self, value: bool) {
        self.offset_version = value;
    }
}

pub trait SimulateTxVersionOffset {
    fn apply_simulate_tx_version_offset(&self) -> TransactionVersion;
}

impl SimulateTxVersionOffset for TransactionVersion {
    fn apply_simulate_tx_version_offset(&self) -> TransactionVersion {
        Felt252Wrapper(Felt252Wrapper::from(self.0).0 + SIMULATE_TX_VERSION_OFFSET).into()
    }
}

pub trait GetValidateEntryPointCalldata {
    fn get_validate_entry_point_calldata(&self) -> Calldata;
}

impl GetValidateEntryPointCalldata for DeclareTransaction {
    fn get_validate_entry_point_calldata(&self) -> Calldata {
        Calldata(Arc::new(vec![self.tx().class_hash().0]))
    }
}

impl GetValidateEntryPointCalldata for DeployAccountTransaction {
    fn get_validate_entry_point_calldata(&self) -> Calldata {
        let mut validate_calldata = Vec::with_capacity(self.tx().constructor_calldata().0.len() + 2);
        validate_calldata.push(self.tx.class_hash().0);
        validate_calldata.push(self.tx.contract_address_salt().0);
        validate_calldata.extend_from_slice(&(self.tx.constructor_calldata().0));
        Calldata(validate_calldata.into())
    }
}

impl GetValidateEntryPointCalldata for InvokeTransaction {
    fn get_validate_entry_point_calldata(&self) -> Calldata {
        self.tx.calldata()
    }
}

impl GetValidateEntryPointCalldata for L1HandlerTransaction {
    fn get_validate_entry_point_calldata(&self) -> Calldata {
        self.tx.calldata.clone()
    }
}

pub trait GetCalldataLen {
    fn get_calldata_len(&self) -> usize;
}
impl GetCalldataLen for DeclareTransaction {
    fn get_calldata_len(&self) -> usize {
        0
    }
}
impl GetCalldataLen for DeployAccountTransaction {
    fn get_calldata_len(&self) -> usize {
        self.tx.constructor_calldata().0.len()
    }
}
impl GetCalldataLen for InvokeTransaction {
    fn get_calldata_len(&self) -> usize {
        self.tx.calldata().0.len()
    }
}

pub trait GetTxType {
    fn tx_type() -> TransactionType;
}

impl GetTxType for DeclareTransaction {
    fn tx_type() -> TransactionType {
        TransactionType::Declare
    }
}
impl GetTxType for DeployAccountTransaction {
    fn tx_type() -> TransactionType {
        TransactionType::DeployAccount
    }
}
impl GetTxType for InvokeTransaction {
    fn tx_type() -> TransactionType {
        TransactionType::InvokeFunction
    }
}
impl GetTxType for L1HandlerTransaction {
    fn tx_type() -> TransactionType {
        TransactionType::L1Handler
    }
}

pub trait HandleNonce {
    fn handle_nonce(state: &mut dyn State, tx_info: &TransactionInfo, strict: bool) -> TransactionExecutionResult<()> {
        if tx_info.is_v0() {
            return Ok(());
        }

        let address = tx_info.sender_address();
        let account_nonce = state.get_nonce_at(address)?;
        let incoming_tx_nonce = tx_info.nonce();
        let valid_nonce = if strict { account_nonce == incoming_tx_nonce } else { account_nonce <= incoming_tx_nonce };

        if valid_nonce {
            state.increment_nonce(address)?;
            Ok(())
        } else {
            Err(TransactionPreValidationError::InvalidNonce { address, account_nonce, incoming_tx_nonce }.into())
        }
    }
}

impl HandleNonce for DeclareTransaction {}
impl HandleNonce for DeployAccountTransaction {}
impl HandleNonce for InvokeTransaction {}

pub trait GetActualCostBuilder {
    fn get_actual_cost_builder(&self, tx_context: Arc<TransactionContext>) -> ActualCostBuilder<'_>;
}

impl GetActualCostBuilder for InvokeTransaction {
    fn get_actual_cost_builder(&self, tx_context: Arc<TransactionContext>) -> ActualCostBuilder<'_> {
        ActualCostBuilder::new(tx_context, Self::tx_type(), self.get_calldata_len(), self.tx.signature().0.len())
    }
}
impl GetActualCostBuilder for DeployAccountTransaction {
    fn get_actual_cost_builder(&self, tx_context: Arc<TransactionContext>) -> ActualCostBuilder<'_> {
        ActualCostBuilder::new(tx_context, Self::tx_type(), self.get_calldata_len(), self.tx.signature().0.len())
    }
}
impl GetActualCostBuilder for DeclareTransaction {
    fn get_actual_cost_builder(&self, tx_context: Arc<TransactionContext>) -> ActualCostBuilder<'_> {
        let mut actual_cost_builder =
            ActualCostBuilder::new(tx_context, Self::tx_type(), self.get_calldata_len(), self.tx.signature().0.len());
        actual_cost_builder = actual_cost_builder.with_class_info(self.class_info.clone());

        actual_cost_builder
    }
}

pub trait CheckFeeBounds: GetCalldataLen + GetTxType {
    fn state_changes() -> StateChangesCount;

    fn check_fee_bounds(&self, tx_context: &TransactionContext) -> TransactionExecutionResult<()> {
        let minimal_l1_gas_amount_vector = {
            let block_info = tx_context.block_context.block_info();
            let versioned_constants = tx_context.block_context.versioned_constants();

            let state_changes = Self::state_changes();

            let GasVector { l1_gas: gas_cost, l1_data_gas: blob_gas_cost } =
                blockifier::fee::gas_usage::get_da_gas_cost(&state_changes, block_info.use_kzg_da);

            let data_segment_length = blockifier::fee::gas_usage::get_onchain_data_segment_length(&state_changes);
            let os_steps_for_type =
                versioned_constants.os_resources_for_tx_type(&Self::tx_type(), self.get_calldata_len()).n_steps
                    + versioned_constants.os_kzg_da_resources(data_segment_length).n_steps;

            let resources = ResourcesMapping(IndexMap::from([
                (blockifier::abi::constants::L1_GAS_USAGE.to_string(), gas_cost),
                (blockifier::abi::constants::BLOB_GAS_USAGE.to_string(), blob_gas_cost),
                (blockifier::abi::constants::N_STEPS_RESOURCE.to_string(), os_steps_for_type as u128),
            ]));

            blockifier::fee::fee_utils::calculate_tx_gas_vector(&resources, versioned_constants)?
        };

        // TODO(Aner, 30/01/24): modify once data gas limit is enforced.
        let minimal_l1_gas_amount = blockifier::fee::gas_usage::compute_discounted_gas_from_gas_vector(
            &minimal_l1_gas_amount_vector,
            tx_context,
        );

        let TransactionContext { block_context, tx_info } = tx_context;
        let block_info = block_context.block_info();
        let fee_type = &tx_info.fee_type();

        match tx_info {
            TransactionInfo::Current(context) => {
                let ResourceBounds { max_amount: max_l1_gas_amount, max_price_per_unit: max_l1_gas_price } =
                    context.l1_resource_bounds()?;

                let max_l1_gas_amount_as_u128: u128 = max_l1_gas_amount.into();
                if max_l1_gas_amount_as_u128 < minimal_l1_gas_amount {
                    return Err(TransactionFeeError::MaxL1GasAmountTooLow {
                        max_l1_gas_amount,
                        // TODO(Ori, 1/2/2024): Write an indicative expect message explaining why
                        // the convertion works.
                        minimal_l1_gas_amount: (minimal_l1_gas_amount
                            .try_into()
                            .expect("Failed to convert u128 to u64.")),
                    })?;
                }

                let actual_l1_gas_price = block_info.gas_prices.get_gas_price_by_fee_type(fee_type);
                if max_l1_gas_price < actual_l1_gas_price.into() {
                    return Err(TransactionFeeError::MaxL1GasPriceTooLow {
                        max_l1_gas_price,
                        actual_l1_gas_price: actual_l1_gas_price.into(),
                    })?;
                }
            }
            TransactionInfo::Deprecated(context) => {
                let max_fee = context.max_fee;
                let min_fee = blockifier::fee::fee_utils::get_fee_by_gas_vector(
                    block_info,
                    minimal_l1_gas_amount_vector,
                    fee_type,
                );
                if max_fee < min_fee {
                    return Err(TransactionFeeError::MaxFeeTooLow { min_fee, max_fee })?;
                }
            }
        };

        Ok(())
    }
}

impl CheckFeeBounds for DeclareTransaction {
    fn state_changes() -> StateChangesCount {
        StateChangesCount {
            n_storage_updates: 1,
            n_class_hash_updates: 0,
            n_compiled_class_hash_updates: 0,
            n_modified_contracts: 1,
        }
    }
}

impl CheckFeeBounds for DeployAccountTransaction {
    fn state_changes() -> StateChangesCount {
        StateChangesCount {
            n_storage_updates: 1,
            n_class_hash_updates: 1,
            n_compiled_class_hash_updates: 0,
            n_modified_contracts: 1,
        }
    }
}

impl CheckFeeBounds for InvokeTransaction {
    fn state_changes() -> StateChangesCount {
        StateChangesCount {
            n_storage_updates: 1,
            n_class_hash_updates: 0,
            n_compiled_class_hash_updates: 0,
            n_modified_contracts: 1,
        }
    }
}

pub trait GetValidateEntryPointSelector {
    fn get_validate_entry_point_selector() -> EntryPointSelector;
}
impl GetValidateEntryPointSelector for DeclareTransaction {
    fn get_validate_entry_point_selector() -> EntryPointSelector {
        selector_from_name(blockifier::transaction::constants::VALIDATE_DECLARE_ENTRY_POINT_NAME)
    }
}
impl GetValidateEntryPointSelector for DeployAccountTransaction {
    fn get_validate_entry_point_selector() -> EntryPointSelector {
        selector_from_name(blockifier::transaction::constants::VALIDATE_DEPLOY_ENTRY_POINT_NAME)
    }
}
impl GetValidateEntryPointSelector for InvokeTransaction {
    fn get_validate_entry_point_selector() -> EntryPointSelector {
        selector_from_name(blockifier::transaction::constants::VALIDATE_ENTRY_POINT_NAME)
    }
}
impl GetValidateEntryPointSelector for L1HandlerTransaction {
    fn get_validate_entry_point_selector() -> EntryPointSelector {
        selector_from_name(blockifier::transaction::constants::VALIDATE_ENTRY_POINT_NAME)
    }
}

#[allow(clippy::too_many_arguments)]
pub trait Validate: GetValidateEntryPointSelector {
    // Implement this to blacklist some transaction versions
    fn validate_tx_version(&self) -> TransactionExecutionResult<()> {
        Ok(())
    }

    fn validate(
        &self,
        state: &mut dyn State,
        tx_context: Arc<TransactionContext>,
        resources: &mut ExecutionResources,
        remaining_gas: &mut u64,
        validate_tx: bool,
        charge_fee: bool,
        strict_nonce_checking: bool,
    ) -> TransactionExecutionResult<Option<CallInfo>>;

    fn perform_pre_validation_stage(
        &self,
        state: &mut dyn State,
        tx_context: Arc<TransactionContext>,
        strict_nonce_checking: bool,
        charge_fee: bool,
    ) -> TransactionExecutionResult<()>;

    fn run_validate_entrypoint(
        &self,
        state: &mut dyn State,
        tx_context: Arc<TransactionContext>,
        resources: &mut ExecutionResources,
        remaining_gas: &mut u64,
        limit_steps_by_resources: bool,
    ) -> TransactionExecutionResult<Option<CallInfo>>;
}

impl<
    T: CheckFeeBounds
        + GetValidateEntryPointCalldata
        + HandleNonce
        + GetValidateEntryPointSelector
        + TransactionInfoCreator,
> Validate for T
{
    fn validate(
        &self,
        state: &mut dyn State,
        tx_context: Arc<TransactionContext>,
        resources: &mut ExecutionResources,
        remaining_gas: &mut u64,
        validate_tx: bool,
        charge_fee: bool,
        strict_nonce_checking: bool,
    ) -> TransactionExecutionResult<Option<CallInfo>> {
        // Check tx version, nonce and fee
        self.perform_pre_validation_stage(state, tx_context.clone(), strict_nonce_checking, charge_fee)?;

        // Run the actual `validate` entrypoint
        if validate_tx {
            self.run_validate_entrypoint(state, tx_context, resources, remaining_gas, charge_fee)
        } else {
            Ok(None)
        }
    }

    fn perform_pre_validation_stage(
        &self,
        state: &mut dyn State,
        tx_context: Arc<TransactionContext>,
        strict_nonce_checking: bool,
        charge_fee: bool,
    ) -> TransactionExecutionResult<()> {
        // Check that version of the Tx is supported by the network
        self.validate_tx_version()?;

        // Check if nonce has a correct value
        Self::handle_nonce(state, &tx_context.tx_info, strict_nonce_checking)?;

        // Check if user has funds to pay the worst case scenario fees
        if charge_fee && tx_context.tx_info.enforce_fee()? {
            self.check_fee_bounds(&tx_context)?;

            blockifier::fee::fee_utils::verify_can_pay_committed_bounds(state, &tx_context)?;
        }

        Ok(())
    }

    fn run_validate_entrypoint(
        &self,
        state: &mut dyn State,
        tx_context: Arc<TransactionContext>,
        resources: &mut ExecutionResources,
        remaining_gas: &mut u64,
        limit_steps_by_resources: bool,
    ) -> TransactionExecutionResult<Option<CallInfo>> {
        // Everything here is a copy paste from blockifier
        // `impl ValidatableTransaction for AccountTransaction`
        // in `crates/blockifier/src/transaction/account_transaction.rs`
        let mut context = EntryPointExecutionContext::new_validate(tx_context, limit_steps_by_resources)?;
        let tx_info = &context.tx_context.tx_info;
        if tx_info.is_v0() {
            return Ok(None);
        }

        let storage_address = tx_info.sender_address();
        let validate_selector = Self::get_validate_entry_point_selector();
        let validate_call = CallEntryPoint {
            entry_point_type: EntryPointType::External,
            entry_point_selector: validate_selector,
            calldata: self.get_validate_entry_point_calldata(),
            class_hash: None,
            code_address: None,
            storage_address,
            caller_address: ContractAddress::default(),
            call_type: CallType::Call,
            initial_gas: *remaining_gas,
        };

        let validate_call_info = validate_call.execute(state, resources, &mut context).map_err(|error| {
            TransactionExecutionError::ValidateTransactionError { error, storage_address, selector: validate_selector }
        })?;

        // Validate return data.
        let class_hash = state.get_class_hash_at(storage_address)?;
        let contract_class = state.get_compiled_contract_class(class_hash)?;
        if let ContractClass::V1(_) = contract_class {
            // The account contract class is a Cairo 1.0 contract; the `validate` entry point should
            // return `VALID`.
            let expected_retdata =
                Retdata(vec![StarkFelt::try_from(blockifier::transaction::constants::VALIDATE_RETDATA)?]);
            if validate_call_info.execution.retdata != expected_retdata {
                return Err(TransactionExecutionError::InvalidValidateReturnData {
                    actual: validate_call_info.execution.retdata,
                });
            }
        }

        blockifier::transaction::transaction_utils::update_remaining_gas(remaining_gas, &validate_call_info);

        Ok(Some(validate_call_info))
    }
}

// Drop the cached state
// Write nothing to the actual storage
pub fn abort_transactional_state<S: State>(_transactional_state: CachedState<MutRefState<'_, S>>) {}

// TODO:
// This should be done in a substrate storage transaction to avoid some write failing at the end,
// leaving the storage in a half baked state.
// This should not happen if we use blockifier state adapter as its internal impl does not fail, but
// still we should respect the signature of those traits method.
// This probably will involve the creation of a `TransactionalBlockifierStateAdapter`.
pub fn commit_transactional_state<S: State + SetArbitraryNonce>(
    transactional_state: CachedState<MutRefState<'_, S>>,
) -> StateResult<()> {
    let storage_changes = transactional_state.get_actual_state_changes()?;

    // Because the nonce update is done inside `handle_nonce`, which is directly apply on the real
    // storage, this seems to always be empty...
    for (contract_address, nonce) in storage_changes.nonce_updates {
        transactional_state.state.0.set_nonce_at(contract_address, nonce)?;
    }

    for (class_hash, compiled_class_hash) in storage_changes.compiled_class_hash_updates {
        transactional_state.state.0.set_compiled_class_hash(class_hash, compiled_class_hash)?;
    }

    for (contract_address, class_hash) in storage_changes.class_hash_updates {
        transactional_state.state.0.set_class_hash_at(contract_address, class_hash)?;
    }

    for (storage_enty, value) in storage_changes.storage_updates {
        transactional_state.state.0.set_storage_at(storage_enty.0, storage_enty.1, value)?;
    }

    for (class_hash, contract_class) in transactional_state.class_hash_to_class.take() {
        transactional_state.state.0.set_contract_class(class_hash, contract_class)?;
    }

    Ok(())
}

pub trait SetArbitraryNonce: State {
    fn set_nonce_at(&mut self, contract_address: ContractAddress, nonce: Nonce) -> StateResult<()>;
}

impl<'a, S: State + SetArbitraryNonce> SetArbitraryNonce for MutRefState<'a, S> {
    fn set_nonce_at(&mut self, contract_address: ContractAddress, nonce: Nonce) -> StateResult<()> {
        self.0.set_nonce_at(contract_address, nonce)
    }
}

impl<S: State + SetArbitraryNonce> SetArbitraryNonce for CachedState<S> {
    fn set_nonce_at(&mut self, contract_address: ContractAddress, nonce: Nonce) -> StateResult<()> {
        let mut current_nonce = self.get_nonce_at(contract_address)?;
        if current_nonce > nonce {
            // Not the good error type, who cares?
            Err(StateError::StateReadError("Impossible to decrease a nonce".to_string()))?;
        }

        // This is super dumb, but `increment_nonce` is the only method exposed by `CachedState` to interact
        // with nonce. The alternative is to make CachedState.cache field public in our fork,
        // probably better honestly
        while current_nonce != nonce {
            self.increment_nonce(contract_address)?;
            current_nonce = self.get_nonce_at(contract_address)?;
        }

        Ok(())
    }
}

pub fn run_non_revertible_transaction<T, S>(
    transaction: &T,
    state: &mut S,
    block_context: &BlockContext,
    validate: bool,
    charge_fee: bool,
) -> TransactionExecutionResult<TransactionExecutionInfo>
where
    S: State,
    T: GetTxType + Executable<S> + Validate + GetActualCostBuilder + TransactionInfoCreator,
{
    let mut resources = ExecutionResources::default();
    let mut remaining_gas = block_context.versioned_constants().tx_initial_gas();
    let tx_context = Arc::new(block_context.to_tx_context(transaction));

    let validate_call_info: Option<CallInfo>;
    let execute_call_info: Option<CallInfo>;
    let strinct_nonce_checking = true;
    if matches!(T::tx_type(), TransactionType::DeployAccount) {
        // Handle `DeployAccount` transactions separately, due to different order of things.
        // Also, the execution context required form the `DeployAccount` execute phase is
        // validation context.
        let mut execution_context = EntryPointExecutionContext::new_validate(tx_context.clone(), charge_fee)?;
        execute_call_info =
            transaction.run_execute(state, &mut resources, &mut execution_context, &mut remaining_gas)?;
        validate_call_info = transaction.validate(
            state,
            tx_context.clone(),
            &mut resources,
            &mut remaining_gas,
            validate,
            charge_fee,
            strinct_nonce_checking,
        )?;
    } else {
        let mut execution_context = EntryPointExecutionContext::new_invoke(tx_context.clone(), charge_fee)?;
        validate_call_info = transaction.validate(
            state,
            tx_context.clone(),
            &mut resources,
            &mut remaining_gas,
            validate,
            charge_fee,
            strinct_nonce_checking,
        )?;
        execute_call_info =
            transaction.run_execute(state, &mut resources, &mut execution_context, &mut remaining_gas)?;
    }

    let (actual_cost, bouncer_resources) = transaction
        .get_actual_cost_builder(tx_context.clone())
        .with_validate_call_info(&validate_call_info)
        .with_execute_call_info(&execute_call_info)
        .build(&resources)?;

    let post_execution_report = PostExecutionReport::new(state, &tx_context, &actual_cost, charge_fee)?;
    let validate_execute_call_info = match post_execution_report.error() {
        Some(error) => Err(TransactionExecutionError::from(error)),
        None => Ok(ValidateExecuteCallInfo::new_accepted(
            validate_call_info,
            execute_call_info,
            actual_cost,
            bouncer_resources,
        )),
    }?;

    let fee_transfer_call_info = AccountTransaction::handle_fee(
        state,
        tx_context,
        validate_execute_call_info.final_cost.actual_fee,
        charge_fee,
    )?;

    let tx_execution_info = TransactionExecutionInfo {
        validate_call_info: validate_execute_call_info.validate_call_info,
        execute_call_info: validate_execute_call_info.execute_call_info,
        fee_transfer_call_info,
        actual_fee: validate_execute_call_info.final_cost.actual_fee,
        da_gas: validate_execute_call_info.final_cost.da_gas,
        actual_resources: validate_execute_call_info.final_cost.actual_resources,
        revert_error: validate_execute_call_info.revert_error,
        bouncer_resources: validate_execute_call_info.bouncer_resources,
    };

    Ok(tx_execution_info)
}

pub fn run_revertible_transaction<T, S>(
    transaction: &T,
    state: &mut S,
    block_context: &BlockContext,
    validate: bool,
    charge_fee: bool,
) -> TransactionExecutionResult<TransactionExecutionInfo>
where
    for<'a> T: Executable<CachedState<MutRefState<'a, S>>>
        + Validate
        + GetActualCostBuilder
        + GetTxType
        + GetCalldataLen
        + TransactionInfoCreator,
    S: State + SetArbitraryNonce,
{
    let mut resources = ExecutionResources::default();
    let mut remaining_gas = block_context.versioned_constants().tx_initial_gas();
    let tx_context = Arc::new(block_context.to_tx_context(transaction));

    let validate_call_info = transaction.validate(
        state,
        tx_context.clone(),
        &mut resources,
        &mut remaining_gas,
        validate,
        charge_fee,
        validate,
    )?;

    let mut execution_context = EntryPointExecutionContext::new_invoke(tx_context.clone(), charge_fee)?;

    let n_allotted_execution_steps = execution_context.subtract_validation_and_overhead_steps(
        &validate_call_info,
        &T::tx_type(),
        transaction.get_calldata_len(),
    );

    // Save the state changes resulting from running `validate_tx`, to be used later for
    // resource and fee calculation.
    let actual_cost_builder_with_validation_changes =
        transaction.get_actual_cost_builder(tx_context.clone()).with_validate_call_info(&validate_call_info);

    let validate_execute_call_info = {
        // Create copies of state and resources for the execution.
        // Both will be rolled back if the execution is reverted or committed upon success.
        let mut execution_resources = resources.clone();
        let mut transactional_state = CachedState::new(MutRefState::new(state), GlobalContractCache::new(1));

        let execution_result = transaction.run_execute(
            &mut transactional_state,
            &mut execution_resources,
            &mut execution_context,
            &mut remaining_gas,
        );

        // Pre-compute cost in case of revert.
        let execution_steps_consumed = n_allotted_execution_steps - execution_context.n_remaining_steps();
        let (revert_cost, bouncer_revert_resources) = actual_cost_builder_with_validation_changes
            .clone()
            .with_reverted_steps(execution_steps_consumed)
            .build(&resources)?;

        match execution_result {
            Ok(execute_call_info) => {
                // When execution succeeded, calculate the actual required fee before committing the
                // transactional state. If max_fee is insufficient, revert the `run_execute` part.
                let (actual_cost, bouncer_resources) = actual_cost_builder_with_validation_changes
                    .with_execute_call_info(&execute_call_info)
                    // Fee is determined by the sum of `validate` and `execute` state changes.
                    // Since `execute_state_changes` are not yet committed, we merge them manually
                    // with `validate_state_changes` to count correctly.
                    .try_add_state_changes(&mut transactional_state)?
                    .build(&execution_resources)?;

                // Post-execution checks.
                let post_execution_report =
                    PostExecutionReport::new(&transactional_state, &tx_context, &actual_cost, charge_fee)?;
                match post_execution_report.error() {
                    Some(post_execution_error) => {
                        // Post-execution check failed. Revert the execution, compute the final fee
                        // to charge and recompute resources used (to be consistent with other
                        // revert case, compute resources by adding consumed execution steps to
                        // validation resources).
                        abort_transactional_state(transactional_state);
                        TransactionExecutionResult::Ok(ValidateExecuteCallInfo::new_reverted(
                            validate_call_info,
                            post_execution_error.to_string(),
                            ActualCost { actual_fee: post_execution_report.recommended_fee(), ..revert_cost },
                            bouncer_revert_resources,
                        ))
                    }
                    None => {
                        // Post-execution check passed, commit the execution.
                        commit_transactional_state(transactional_state)?;
                        Ok(ValidateExecuteCallInfo::new_accepted(
                            validate_call_info,
                            execute_call_info,
                            actual_cost,
                            bouncer_resources,
                        ))
                    }
                }
            }
            Err(execution_error) => {
                // Error during execution. Revert, even if the error is sequencer-related.
                abort_transactional_state(transactional_state);
                let post_execution_report = PostExecutionReport::new(state, &tx_context, &revert_cost, charge_fee)?;
                TransactionExecutionResult::Ok(ValidateExecuteCallInfo::new_reverted(
                    validate_call_info,
                    execution_error.to_string(),
                    ActualCost { actual_fee: post_execution_report.recommended_fee(), ..revert_cost },
                    bouncer_revert_resources,
                ))
            }
        }
    }?;

    let fee_transfer_call_info = AccountTransaction::handle_fee(
        state,
        tx_context,
        validate_execute_call_info.final_cost.actual_fee,
        charge_fee,
    )?;

    let tx_execution_info = TransactionExecutionInfo {
        validate_call_info: validate_execute_call_info.validate_call_info,
        execute_call_info: validate_execute_call_info.execute_call_info,
        fee_transfer_call_info,
        actual_fee: validate_execute_call_info.final_cost.actual_fee,
        da_gas: validate_execute_call_info.final_cost.da_gas,
        actual_resources: validate_execute_call_info.final_cost.actual_resources,
        revert_error: validate_execute_call_info.revert_error,
        bouncer_resources: validate_execute_call_info.bouncer_resources,
    };

    Ok(tx_execution_info)
}

pub fn execute_l1_handler_transaction<S: State>(
    transaction: &L1HandlerTransaction,
    state: &mut S,
    block_context: &BlockContext,
) -> TransactionExecutionResult<TransactionExecutionInfo> {
    let mut execution_resources = ExecutionResources::default();
    let mut remaining_gas = block_context.versioned_constants().tx_initial_gas();
    let tx_context = Arc::new(block_context.to_tx_context(transaction));

    let mut context = EntryPointExecutionContext::new_invoke(tx_context.clone(), true)?;

    let execute_call_info =
        transaction.run_execute(state, &mut execution_resources, &mut context, &mut remaining_gas)?;
    let l1_handler_payload_size = transaction.payload_size();

    let (ActualCost { actual_fee, da_gas, actual_resources }, _bouncer_resources) =
        ActualCost::builder_for_l1_handler(tx_context, l1_handler_payload_size)
            .with_execute_call_info(&execute_call_info)
            .build(&execution_resources)?;

    let paid_fee = transaction.paid_fee_on_l1;
    // For now, assert only that any amount of fee was paid.
    // The error message still indicates the required fee.
    if paid_fee == Fee(0) {
        Err(TransactionFeeError::InsufficientL1Fee { paid_fee, actual_fee })?;
    }

    Ok(TransactionExecutionInfo {
        validate_call_info: None,
        execute_call_info,
        fee_transfer_call_info: None,
        actual_fee: Fee::default(),
        da_gas,
        actual_resources: actual_resources.clone(),
        revert_error: None,
        bouncer_resources: actual_resources,
    })
}

/// Wraps a mutable reference to a `State` object, exposing its API.
/// Used to pass ownership to a `CachedState`.
pub struct MutRefState<'a, S: State + ?Sized>(&'a mut S);

impl<'a, S: State + ?Sized> MutRefState<'a, S> {
    pub fn new(state: &'a mut S) -> Self {
        Self(state)
    }
}

/// Proxies inner object to expose `State` functionality.
impl<'a, S: State + ?Sized> StateReader for MutRefState<'a, S> {
    fn get_storage_at(&self, contract_address: ContractAddress, key: StorageKey) -> StateResult<StarkFelt> {
        self.0.get_storage_at(contract_address, key)
    }

    fn get_nonce_at(&self, contract_address: ContractAddress) -> StateResult<Nonce> {
        self.0.get_nonce_at(contract_address)
    }

    fn get_class_hash_at(&self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        self.0.get_class_hash_at(contract_address)
    }

    fn get_compiled_contract_class(&self, class_hash: ClassHash) -> StateResult<ContractClass> {
        self.0.get_compiled_contract_class(class_hash)
    }

    fn get_compiled_class_hash(&self, class_hash: ClassHash) -> StateResult<CompiledClassHash> {
        self.0.get_compiled_class_hash(class_hash)
    }
}

impl<'a, S: State + ?Sized> State for MutRefState<'a, S> {
    fn set_storage_at(
        &mut self,
        contract_address: ContractAddress,
        key: StorageKey,
        value: StarkFelt,
    ) -> StateResult<()> {
        self.0.set_storage_at(contract_address, key, value)
    }

    fn increment_nonce(&mut self, contract_address: ContractAddress) -> StateResult<()> {
        self.0.increment_nonce(contract_address)
    }

    fn set_class_hash_at(&mut self, contract_address: ContractAddress, class_hash: ClassHash) -> StateResult<()> {
        self.0.set_class_hash_at(contract_address, class_hash)
    }

    fn set_contract_class(&mut self, class_hash: ClassHash, contract_class: ContractClass) -> StateResult<()> {
        self.0.set_contract_class(class_hash, contract_class)
    }

    fn set_compiled_class_hash(
        &mut self,
        class_hash: ClassHash,
        compiled_class_hash: CompiledClassHash,
    ) -> StateResult<()> {
        self.0.set_compiled_class_hash(class_hash, compiled_class_hash)
    }

    fn add_visited_pcs(&mut self, class_hash: ClassHash, pcs: &HashSet<usize>) {
        self.0.add_visited_pcs(class_hash, pcs)
    }
}
