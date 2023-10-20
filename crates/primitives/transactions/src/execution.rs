use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use blockifier::abi::abi_utils::selector_from_name;
use blockifier::abi::constants::{INITIAL_GAS_COST, TRANSACTION_GAS_COST};
use blockifier::block_context::BlockContext;
use blockifier::execution::entry_point::{
    CallEntryPoint, CallInfo, CallType, EntryPointExecutionContext, ExecutionResources,
};
use blockifier::state::state_api::State;
use blockifier::transaction::constants::{
    VALIDATE_DECLARE_ENTRY_POINT_NAME, VALIDATE_DEPLOY_ENTRY_POINT_NAME, VALIDATE_ENTRY_POINT_NAME,
};
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::{
    AccountTransactionContext, ResourcesMapping, TransactionExecutionInfo, TransactionExecutionResult,
};
use blockifier::transaction::transaction_types::TransactionType;
use blockifier::transaction::transaction_utils::{update_remaining_gas, verify_no_calls_to_other_contracts};
use blockifier::transaction::transactions::{
    DeclareTransaction, DeployAccountTransaction, Executable, InvokeTransaction, L1HandlerTransaction,
};
use mp_fee::{calculate_tx_fee, charge_fee, compute_transaction_resources};
use mp_felt::Felt252Wrapper;
use mp_state::{FeeConfig, StateChanges};
use starknet_api::api_core::{ContractAddress, EntryPointSelector, Nonce};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, Fee, TransactionSignature, TransactionVersion};

use super::SIMULATE_TX_VERSION_OFFSET;

const TX_INITIAL_AVAILABLE_GAS: u64 = INITIAL_GAS_COST - TRANSACTION_GAS_COST;

pub struct ValidateExecuteCallInfo {
    pub validate_call_info: Option<CallInfo>,
    pub execute_call_info: Option<CallInfo>,
    pub revert_error: Option<String>,
}

impl ValidateExecuteCallInfo {
    fn new_accepted(validate_call_info: Option<CallInfo>, execute_call_info: Option<CallInfo>) -> Self {
        Self { validate_call_info, execute_call_info, revert_error: None }
    }

    fn new_reverted(validate_call_info: Option<CallInfo>, revert_error: String) -> Self {
        Self { validate_call_info, execute_call_info: None, revert_error: Some(revert_error) }
    }
}

pub trait GetAccountTransactionContext {
    fn get_account_transaction_context(&self, is_query: bool) -> AccountTransactionContext;
}

pub trait SimulateTxVersionOffset {
    fn apply_simulate_tx_version_offset(&self) -> TransactionVersion;
}

impl SimulateTxVersionOffset for TransactionVersion {
    fn apply_simulate_tx_version_offset(&self) -> TransactionVersion {
        Felt252Wrapper(Felt252Wrapper::from(self.0).0 + SIMULATE_TX_VERSION_OFFSET).into()
    }
}

impl GetAccountTransactionContext for DeclareTransaction {
    fn get_account_transaction_context(&self, is_query: bool) -> AccountTransactionContext {
        let mut version = self.tx().version();
        if is_query {
            version = version.apply_simulate_tx_version_offset();
        }

        AccountTransactionContext {
            transaction_hash: self.tx_hash(),
            max_fee: self.tx().max_fee(),
            version,
            signature: self.tx().signature(),
            nonce: self.tx().nonce(),
            sender_address: self.tx().sender_address(),
        }
    }
}

impl GetAccountTransactionContext for DeployAccountTransaction {
    fn get_account_transaction_context(&self, is_query: bool) -> AccountTransactionContext {
        let mut version = self.version();
        if is_query {
            version = version.apply_simulate_tx_version_offset();
        }

        AccountTransactionContext {
            transaction_hash: self.tx_hash,
            max_fee: self.max_fee(),
            version,
            signature: self.signature(),
            nonce: self.nonce(),
            sender_address: self.contract_address,
        }
    }
}

impl GetAccountTransactionContext for InvokeTransaction {
    fn get_account_transaction_context(&self, is_query: bool) -> AccountTransactionContext {
        let mut version = match self.tx {
            starknet_api::transaction::InvokeTransaction::V0(_) => TransactionVersion(StarkFelt::from(0u8)),
            starknet_api::transaction::InvokeTransaction::V1(_) => TransactionVersion(StarkFelt::from(1u8)),
        };
        if is_query {
            version = version.apply_simulate_tx_version_offset();
        }

        let nonce = match &self.tx {
            starknet_api::transaction::InvokeTransaction::V0(_) => Nonce::default(),
            starknet_api::transaction::InvokeTransaction::V1(tx) => tx.nonce,
        };

        let sender_address = match &self.tx {
            starknet_api::transaction::InvokeTransaction::V0(tx) => tx.contract_address,
            starknet_api::transaction::InvokeTransaction::V1(tx) => tx.sender_address,
        };

        AccountTransactionContext {
            transaction_hash: self.tx_hash,
            max_fee: self.max_fee(),
            version,
            signature: self.signature(),
            nonce,
            sender_address,
        }
    }
}

impl GetAccountTransactionContext for L1HandlerTransaction {
    fn get_account_transaction_context(&self, is_query: bool) -> AccountTransactionContext {
        let mut version = self.tx.version;
        if is_query {
            version = version.apply_simulate_tx_version_offset();
        }

        AccountTransactionContext {
            transaction_hash: self.tx_hash,
            max_fee: Fee::default(),
            version,
            signature: TransactionSignature::default(),
            nonce: self.tx.nonce,
            sender_address: self.tx.contract_address,
        }
    }
}

pub trait GetTransactionCalldata {
    fn calldata(&self) -> Calldata;
}

impl GetTransactionCalldata for DeclareTransaction {
    fn calldata(&self) -> Calldata {
        Calldata(Arc::new(vec![self.tx().class_hash().0]))
    }
}

impl GetTransactionCalldata for DeployAccountTransaction {
    fn calldata(&self) -> Calldata {
        let mut validate_calldata = Vec::with_capacity((*self.tx.constructor_calldata.0).len() + 2);
        validate_calldata.push(self.tx.class_hash.0);
        validate_calldata.push(self.tx.contract_address_salt.0);
        validate_calldata.extend_from_slice(&(self.tx.constructor_calldata.0));
        Calldata(validate_calldata.into())
    }
}

impl GetTransactionCalldata for InvokeTransaction {
    fn calldata(&self) -> Calldata {
        self.calldata()
    }
}

impl GetTransactionCalldata for L1HandlerTransaction {
    fn calldata(&self) -> Calldata {
        self.tx.calldata.clone()
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

pub trait Validate: GetAccountTransactionContext + GetTransactionCalldata {
    const VALIDATE_TX_ENTRY_POINT_NAME: &'static str;

    fn validate_entry_point_selector(&self) -> EntryPointSelector {
        selector_from_name(Self::VALIDATE_TX_ENTRY_POINT_NAME)
    }

    fn validate_tx(
        &self,
        state: &mut dyn State,
        block_context: &BlockContext,
        resources: &mut ExecutionResources,
        remaining_gas: &mut u64,
        is_query: bool,
    ) -> TransactionExecutionResult<Option<CallInfo>> {
        let account_tx_context = self.get_account_transaction_context(is_query);
        let mut context = EntryPointExecutionContext::new(
            block_context.clone(),
            account_tx_context,
            block_context.invoke_tx_max_n_steps,
        );

        self.validate_tx_inner(state, resources, remaining_gas, &mut context, self.calldata())
    }

    fn validate_tx_inner(
        &self,
        state: &mut dyn State,
        resources: &mut ExecutionResources,
        remaining_gas: &mut u64,
        entry_point_execution_context: &mut EntryPointExecutionContext,
        calldata: Calldata,
    ) -> TransactionExecutionResult<Option<CallInfo>> {
        if entry_point_execution_context.account_tx_context.is_v0() {
            return Ok(None);
        }

        let storage_address = entry_point_execution_context.account_tx_context.sender_address;
        let validate_call = CallEntryPoint {
            entry_point_type: EntryPointType::External,
            entry_point_selector: self.validate_entry_point_selector(),
            calldata,
            class_hash: None,
            code_address: None,
            storage_address,
            caller_address: ContractAddress::default(),
            call_type: CallType::Call,
            initial_gas: *remaining_gas,
        };

        let validate_call_info = validate_call
            .execute(state, resources, entry_point_execution_context)
            .map_err(TransactionExecutionError::ValidateTransactionError)?;
        verify_no_calls_to_other_contracts(&validate_call_info, String::from(VALIDATE_ENTRY_POINT_NAME))?;
        update_remaining_gas(remaining_gas, &validate_call_info);

        Ok(Some(validate_call_info))
    }
}

pub trait Execute: Sized + GetAccountTransactionContext + GetTransactionCalldata + GetTxType {
    fn execute_inner<S: State + StateChanges + FeeConfig>(
        &self,
        state: &mut S,
        block_context: &BlockContext,
        resources: &mut ExecutionResources,
        remaining_gas: &mut u64,
        account_tx_context: &AccountTransactionContext,
    ) -> TransactionExecutionResult<ValidateExecuteCallInfo>;

    fn handle_nonce(
        account_tx_context: &AccountTransactionContext,
        state: &mut dyn State,
    ) -> TransactionExecutionResult<()> {
        if account_tx_context.version == TransactionVersion(StarkFelt::from(0_u8)) {
            return Ok(());
        }

        let address = account_tx_context.sender_address;
        let current_nonce = state.get_nonce_at(address)?;
        if current_nonce != account_tx_context.nonce {
            return Err(TransactionExecutionError::InvalidNonce {
                address,
                expected_nonce: current_nonce,
                actual_nonce: account_tx_context.nonce,
            });
        }

        // Increment nonce.
        state.increment_nonce(address)?;

        Ok(())
    }

    /// Handles nonce and checks that the account's balance covers max fee.
    fn handle_nonce_and_check_fee_balance(
        state: &mut dyn State,
        block_context: &BlockContext,
        account_tx_context: &AccountTransactionContext,
        disable_nonce_validation: bool,
    ) -> TransactionExecutionResult<()> {
        // Handle nonce.

        if !disable_nonce_validation {
            Self::handle_nonce(account_tx_context, state)?;
        }

        // Check fee balance.
        if account_tx_context.max_fee != Fee(0) {
            let (balance_low, balance_high) =
                state.get_fee_token_balance(block_context, &account_tx_context.sender_address)?;

            if balance_high <= StarkFelt::from(0_u8) && balance_low < StarkFelt::from(account_tx_context.max_fee.0) {
                return Err(TransactionExecutionError::MaxFeeExceedsBalance {
                    max_fee: account_tx_context.max_fee,
                    balance_low,
                    balance_high,
                });
            }
        }

        Ok(())
    }

    fn execute<S: State + StateChanges + FeeConfig>(
        &self,
        state: &mut S,
        block_context: &BlockContext,
        is_query: bool,
        disable_nonce_validation: bool,
    ) -> TransactionExecutionResult<TransactionExecutionInfo> {
        let mut execution_resources = ExecutionResources::default();
        let mut remaining_gas = TX_INITIAL_AVAILABLE_GAS;

        let account_tx_context = self.get_account_transaction_context(is_query);

        // Nonce and fee check should be done before running user code.
        Self::handle_nonce_and_check_fee_balance(state, block_context, &account_tx_context, disable_nonce_validation)?;

        // execute
        let ValidateExecuteCallInfo { validate_call_info, execute_call_info, revert_error } = self.execute_inner(
            state,
            block_context,
            &mut execution_resources,
            &mut remaining_gas,
            &account_tx_context,
        )?;

        let (actual_fee, fee_transfer_call_info, actual_resources) = self.handle_fee(
            state,
            &execute_call_info,
            &validate_call_info,
            &mut execution_resources,
            block_context,
            account_tx_context,
        )?;

        let tx_execution_info = TransactionExecutionInfo {
            validate_call_info,
            execute_call_info,
            fee_transfer_call_info,
            actual_fee,
            actual_resources,
            revert_error,
        };

        Ok(tx_execution_info)
    }

    fn handle_fee<S: State + StateChanges + FeeConfig>(
        &self,
        state: &mut S,
        execute_call_info: &Option<CallInfo>,
        validate_call_info: &Option<CallInfo>,
        execution_resources: &mut ExecutionResources,
        block_context: &BlockContext,
        account_tx_context: AccountTransactionContext,
    ) -> TransactionExecutionResult<(Fee, Option<CallInfo>, ResourcesMapping)> {
        let actual_resources = compute_transaction_resources(
            state,
            execute_call_info,
            validate_call_info,
            execution_resources,
            Self::tx_type(),
            None,
        )?;

        let (actual_fee, fee_transfer_call_info) =
            charge_fee(state, block_context, account_tx_context, &actual_resources)?;

        Ok((actual_fee, fee_transfer_call_info, actual_resources))
    }
}

impl Validate for InvokeTransaction {
    const VALIDATE_TX_ENTRY_POINT_NAME: &'static str = VALIDATE_ENTRY_POINT_NAME;
}

impl Execute for InvokeTransaction {
    fn execute_inner<S: State + StateChanges + FeeConfig>(
        &self,
        state: &mut S,
        block_context: &BlockContext,
        resources: &mut ExecutionResources,
        remaining_gas: &mut u64,
        account_tx_context: &AccountTransactionContext,
    ) -> TransactionExecutionResult<ValidateExecuteCallInfo> {
        let mut context = EntryPointExecutionContext::new(
            block_context.clone(),
            account_tx_context.clone(),
            block_context.invoke_tx_max_n_steps,
        );

        let validate_call_info = self.validate_tx_inner(
            state,
            resources,
            remaining_gas,
            &mut context,
            GetTransactionCalldata::calldata(self),
        )?;
        let validate_execute_call_info = match self.tx {
            // V0 tx cannot revert, we cannot charge the failling ones
            starknet_api::transaction::InvokeTransaction::V0(_) => {
                let execute_call_info = self.run_execute(state, resources, &mut context, remaining_gas)?;
                ValidateExecuteCallInfo::new_accepted(validate_call_info, execute_call_info)
            }
            starknet_api::transaction::InvokeTransaction::V1(_) => {
                match self.run_execute(state, resources, &mut context, remaining_gas) {
                    Ok(execute_call_info) => {
                        ValidateExecuteCallInfo::new_accepted(validate_call_info, execute_call_info)
                    }
                    Err(_) => ValidateExecuteCallInfo::new_reverted(validate_call_info, context.error_trace()),
                }
            }
        };

        Ok(validate_execute_call_info)
    }
}

impl Validate for DeclareTransaction {
    const VALIDATE_TX_ENTRY_POINT_NAME: &'static str = VALIDATE_DECLARE_ENTRY_POINT_NAME;
}

impl Execute for DeclareTransaction {
    fn execute_inner<S: State + StateChanges + FeeConfig>(
        &self,
        state: &mut S,
        block_context: &BlockContext,
        resources: &mut ExecutionResources,
        remaining_gas: &mut u64,
        account_tx_context: &AccountTransactionContext,
    ) -> TransactionExecutionResult<ValidateExecuteCallInfo> {
        let mut context = EntryPointExecutionContext::new(
            block_context.clone(),
            account_tx_context.clone(),
            block_context.invoke_tx_max_n_steps,
        );

        let validate_call_info =
            self.validate_tx_inner(state, resources, remaining_gas, &mut context, self.calldata())?;
        let validate_execute_call_info = match self.tx() {
            // V0 tx cannot revert, we cannot charge the failling ones
            starknet_api::transaction::DeclareTransaction::V0(_) => {
                let execute_call_info = self.run_execute(state, resources, &mut context, remaining_gas)?;
                ValidateExecuteCallInfo::new_accepted(validate_call_info, execute_call_info)
            }
            starknet_api::transaction::DeclareTransaction::V1(_)
            | starknet_api::transaction::DeclareTransaction::V2(_) => {
                match self.run_execute(state, resources, &mut context, remaining_gas) {
                    Ok(execute_call_info) => {
                        ValidateExecuteCallInfo::new_accepted(validate_call_info, execute_call_info)
                    }
                    Err(_) => ValidateExecuteCallInfo::new_reverted(validate_call_info, context.error_trace()),
                }
            }
        };

        Ok(validate_execute_call_info)
    }
}

impl Validate for DeployAccountTransaction {
    const VALIDATE_TX_ENTRY_POINT_NAME: &'static str = VALIDATE_DEPLOY_ENTRY_POINT_NAME;
}

impl Execute for DeployAccountTransaction {
    fn execute_inner<S: State + StateChanges + FeeConfig>(
        &self,
        state: &mut S,
        block_context: &BlockContext,
        resources: &mut ExecutionResources,
        remaining_gas: &mut u64,
        account_tx_context: &AccountTransactionContext,
    ) -> TransactionExecutionResult<ValidateExecuteCallInfo> {
        let mut context = EntryPointExecutionContext::new(
            block_context.clone(),
            account_tx_context.clone(),
            block_context.invoke_tx_max_n_steps,
        );

        // In order to be verified the tx must first be executed
        // so that the `constructor` method can initialize the account state
        let execute_call_info = self.run_execute(state, resources, &mut context, remaining_gas)?;
        let validate_call_info =
            self.validate_tx_inner(state, resources, remaining_gas, &mut context, self.calldata())?;

        Ok(ValidateExecuteCallInfo::new_accepted(validate_call_info, execute_call_info))
    }
}

impl Execute for L1HandlerTransaction {
    fn execute_inner<S: State + StateChanges + FeeConfig>(
        &self,
        state: &mut S,
        block_context: &BlockContext,
        resources: &mut ExecutionResources,
        remaining_gas: &mut u64,
        account_tx_context: &AccountTransactionContext,
    ) -> TransactionExecutionResult<ValidateExecuteCallInfo> {
        let mut context = EntryPointExecutionContext::new(
            block_context.clone(),
            account_tx_context.clone(),
            block_context.invoke_tx_max_n_steps,
        );

        let execute_call_info = self.run_execute(state, resources, &mut context, remaining_gas)?;

        Ok(ValidateExecuteCallInfo::new_accepted(None, execute_call_info))
    }

    // No fee are charged for L1HandlerTransaction
    fn handle_fee<S: State + StateChanges + FeeConfig>(
        &self,
        state: &mut S,
        execute_call_info: &Option<CallInfo>,
        validate_call_info: &Option<CallInfo>,
        execution_resources: &mut ExecutionResources,
        block_context: &BlockContext,
        _account_tx_context: AccountTransactionContext,
    ) -> TransactionExecutionResult<(Fee, Option<CallInfo>, ResourcesMapping)> {
        // The calldata includes the "from" field, which is not a part of the payload.
        let l1_handler_payload_size = self.calldata().0.len() - 1;

        let actual_resources = compute_transaction_resources(
            state,
            execute_call_info,
            validate_call_info,
            execution_resources,
            Self::tx_type(),
            Some(l1_handler_payload_size),
        )?;

        let actual_fee = calculate_tx_fee(&actual_resources, block_context)?;

        let paid_fee = self.paid_fee_on_l1;
        // For now, assert only that any amount of fee was paid.
        // The error message still indicates the required fee.
        if paid_fee == Fee(0) {
            return Err(TransactionExecutionError::InsufficientL1Fee { paid_fee, actual_fee });
        }

        Ok((Fee::default(), None, actual_resources))
    }
}

#[cfg(test)]
mod simulate_tx_offset {
    use blockifier::execution::contract_class::ContractClass;
    use starknet_ff::FieldElement;

    use super::*;

    #[test]
    fn offset_is_correct() {
        assert_eq!(
            SIMULATE_TX_VERSION_OFFSET,
            FieldElement::from_hex_be("0x100000000000000000000000000000000").unwrap()
        );
    }

    #[test]
    fn l1_handler_transaction_correctly_applies_simulate_tx_version_offset() {
        let l1_handler_tx = L1HandlerTransaction {
            tx: Default::default(),
            paid_fee_on_l1: Default::default(),
            tx_hash: Default::default(),
        };

        let original_version = l1_handler_tx.tx.version;
        let queried_version = l1_handler_tx.get_account_transaction_context(true).version;

        assert_eq!(
            queried_version,
            Felt252Wrapper(Felt252Wrapper::from(original_version.0).0 + SIMULATE_TX_VERSION_OFFSET).into()
        );

        let non_queried_version = l1_handler_tx.get_account_transaction_context(false).version;
        assert_eq!(non_queried_version, original_version);
    }

    #[test]
    fn deploy_account_transaction_correctly_applies_simulate_tx_version_offset() {
        let deploy_account_tx = DeployAccountTransaction {
            tx: Default::default(),
            tx_hash: Default::default(),
            contract_address: Default::default(),
        };

        let original_version = deploy_account_tx.tx.version;

        let queried_version = deploy_account_tx.get_account_transaction_context(true).version;
        assert_eq!(
            queried_version,
            Felt252Wrapper(Felt252Wrapper::from(original_version.0).0 + SIMULATE_TX_VERSION_OFFSET).into()
        );

        let non_queried_version = deploy_account_tx.get_account_transaction_context(false).version;
        assert_eq!(non_queried_version, original_version);
    }

    #[test]
    fn declare_transaction_correctly_applies_simulate_tx_version_offset() {
        let declare_tx_v0 = DeclareTransaction::new(
            starknet_api::transaction::DeclareTransaction::V0(Default::default()),
            Default::default(),
            ContractClass::V0(Default::default()),
        )
        .unwrap();

        // gen TxVersion from v0 manually
        let original_version_v0 = TransactionVersion(StarkFelt::from(0u8));

        let queried_version = declare_tx_v0.get_account_transaction_context(true).version;
        assert_eq!(
            queried_version,
            Felt252Wrapper(Felt252Wrapper::from(original_version_v0.0).0 + SIMULATE_TX_VERSION_OFFSET).into()
        );

        let non_queried_version = declare_tx_v0.get_account_transaction_context(false).version;
        assert_eq!(non_queried_version, original_version_v0);
    }

    #[test]
    fn invoke_transaction_correctly_applies_simulate_tx_version_offset() {
        let invoke_tx = InvokeTransaction {
            tx: starknet_api::transaction::InvokeTransaction::V0(Default::default()),
            tx_hash: Default::default(),
        };

        // gen TxVersion from v0 manually
        let original_version_v0 = TransactionVersion(StarkFelt::from(0u8));

        let queried_version = invoke_tx.get_account_transaction_context(true).version;
        assert_eq!(
            queried_version,
            Felt252Wrapper(Felt252Wrapper::from(original_version_v0.0).0 + SIMULATE_TX_VERSION_OFFSET).into()
        );

        let non_queried_version = invoke_tx.get_account_transaction_context(false).version;
        assert_eq!(non_queried_version, original_version_v0);
    }
}
