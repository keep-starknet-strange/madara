//! Starknet transaction related functionality.
/// Constants related to transactions.
pub mod constants;
/// Types related to transactions.
pub mod types;
/// Functions related to transaction conversions
pub mod utils;

use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;

use blockifier::block_context::BlockContext;
use blockifier::execution::contract_class::ContractClass;
use blockifier::execution::entry_point::{
    CallEntryPoint, CallInfo, CallType, EntryPointExecutionContext, ExecutionResources,
};
use blockifier::execution::errors::EntryPointExecutionError;
use blockifier::state::errors::StateError;
use blockifier::state::state_api::State;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::AccountTransactionContext;
use blockifier::transaction::transaction_utils::{update_remaining_gas, verify_no_calls_to_other_contracts};
use blockifier::transaction::transactions::{
    DeclareTransaction as StarknetDeclareTransaction, Executable,
};
use cairo_vm::felt::Felt252;
use frame_support::BoundedVec;
use sp_core::U256;
use starknet_api::api_core::{ClassHash, ContractAddress as StarknetContractAddress, EntryPointSelector, Nonce};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::{StarkFelt};
use starknet_api::transaction::{
    Calldata, ContractAddressSalt, DeclareTransaction as DeclareTransactionAPI,
    DeclareTransactionV0V1 as DeclareTransactionAPIV0V1, DeployAccountTransaction as DeployAccountTransactionAPI,
    EventContent, Fee, InvokeTransaction as InvokeTransactionAPI, InvokeTransactionV1 as InvokeTransactionAPIV1,
    L1HandlerTransaction, TransactionHash, TransactionOutput, TransactionReceipt, TransactionSignature,
    TransactionVersion,
};
use starknet_api::{calldata, StarknetApiError};

use self::types::{
    DeclareTransaction, DeployAccountTransaction, EventError, EventWrapper, InvokeTransaction, InvokeTransactionV1,
    MaxArraySize, Transaction, TransactionExecutionErrorWrapper,
    TransactionExecutionInfoWrapper, TransactionExecutionResultWrapper, TransactionReceiptWrapper,
    TransactionValidationErrorWrapper, TransactionValidationResultWrapper, TxType,
};
use crate::execution::types::{ContractAddressWrapper, Felt252Wrapper};
use crate::fees::{self, charge_fee};
use crate::state::StateChanges;

impl EventWrapper {
    /// Creates a new instance of an event.
    ///
    /// # Arguments
    ///
    /// * `keys` - Event keys.
    /// * `data` - Event data.
    /// * `from_address` - Contract Address where the event was emitted from.
    /// * `transaction_hash` - Transaction hash where the event was emitted from.
    pub fn new(
        keys: BoundedVec<Felt252Wrapper, MaxArraySize>,
        data: BoundedVec<Felt252Wrapper, MaxArraySize>,
        from_address: ContractAddressWrapper,
        transaction_hash: Felt252Wrapper,
    ) -> Self {
        Self { keys, data, from_address, transaction_hash }
    }

    /// Creates an empty event.
    pub fn empty() -> Self {
        Self {
            keys: BoundedVec::try_from(vec![]).unwrap(),
            data: BoundedVec::try_from(vec![]).unwrap(),
            from_address: ContractAddressWrapper::default(),
            transaction_hash: Felt252Wrapper::default(),
        }
    }

    /// Creates a new instance of an event builder.
    pub fn builder() -> EventBuilder {
        EventBuilder::default()
    }
}

/// Builder pattern for `EventWrapper`.
#[derive(Default)]
pub struct EventBuilder {
    keys: vec::Vec<Felt252Wrapper>,
    data: vec::Vec<Felt252Wrapper>,
    from_address: Option<StarknetContractAddress>,
    transaction_hash: Option<TransactionHash>,
}

impl EventBuilder {
    /// Sets the keys of the event.
    ///
    /// # Arguments
    ///
    /// * `keys` - Event keys.
    pub fn with_keys(mut self, keys: vec::Vec<Felt252Wrapper>) -> Self {
        self.keys = keys;
        self
    }

    /// Sets the data of the event.
    ///
    /// # Arguments
    ///
    /// * `data` - Event data.
    pub fn with_data(mut self, data: vec::Vec<Felt252Wrapper>) -> Self {
        self.data = data;
        self
    }

    /// Sets the from address of the event.
    ///
    /// # Arguments
    ///
    /// * `from_address` - Contract Address where the event was emitted from.
    pub fn with_from_address(mut self, from_address: StarknetContractAddress) -> Self {
        self.from_address = Some(from_address);
        self
    }

    /// Sets the transaction hash of the event.
    ///
    /// # Arguments
    ///
    /// * `transaction_hash` - Transaction hash where the event was emitted from.
    pub fn with_transaction_hash(mut self, transaction_hash: TransactionHash) -> Self {
        self.transaction_hash = Some(transaction_hash);
        self
    }

    /// Sets keys and data from an event content.
    ///
    /// # Arguments
    ///
    /// * `event_content` - Event content retrieved from the `CallInfo`.
    pub fn with_event_content(mut self, event_content: EventContent) -> Self {
        // TODO: what's the proper why to handle errors in a map? We should return Return<Self,
        // Felt252WrapperError> instead?
        self.keys = event_content.keys.iter().map(|k| k.0.into()).collect::<vec::Vec<Felt252Wrapper>>();
        self.data = event_content.data.0.iter().map(|d| Felt252Wrapper::from(*d)).collect::<vec::Vec<Felt252Wrapper>>();
        self
    }

    /// Builds the event.
    pub fn build(self) -> Result<EventWrapper, EventError> {
        Ok(EventWrapper {
            keys: BoundedVec::try_from(self.keys).map_err(|_| EventError::InvalidKeys)?,
            data: BoundedVec::try_from(self.data).map_err(|_| EventError::InvalidData)?,
            from_address: self
                .from_address
                .unwrap_or_default()
                .0
                .key()
                .bytes()
                .try_into()
                .map_err(|_| EventError::InvalidFromAddress)?,
            transaction_hash: self.transaction_hash.unwrap_or_default().0.into(),
        })
    }
}

impl Default for EventWrapper {
    fn default() -> Self {
        let one = Felt252Wrapper::ONE;
        Self {
            keys: BoundedVec::try_from(vec![one, one]).unwrap(),
            data: BoundedVec::try_from(vec![one, one]).unwrap(),
            from_address: one,
            transaction_hash: Felt252Wrapper::default(),
        }
    }
}

/// Try to convert a `&TransactionReceipt` into a `TransactionReceiptWrapper`.
impl TryInto<TransactionReceiptWrapper> for &TransactionReceipt {
    type Error = EventError;

    fn try_into(self) -> Result<TransactionReceiptWrapper, Self::Error> {
        let _events: Result<vec::Vec<EventWrapper>, EventError> = self
            .output
            .events()
            .iter()
            .map(|e| {
                EventWrapper::builder().with_event_content(e.content.clone()).with_from_address(e.from_address).build()
            })
            .collect();

        Ok(TransactionReceiptWrapper {
            transaction_hash: self.transaction_hash.0.into(),
            actual_fee: U256::from(self.output.actual_fee().0).try_into().expect("Actual fee too large for felt252."),
            tx_type: match self.output {
                TransactionOutput::Declare(_) => TxType::Declare,
                TransactionOutput::DeployAccount(_) => TxType::DeployAccount,
                TransactionOutput::Invoke(_) => TxType::Invoke,
                TransactionOutput::L1Handler(_) => TxType::L1Handler,
                _ => TxType::Invoke,
            },
            events: BoundedVec::try_from(_events?).map_err(|_| EventError::TooManyEvents)?,
        })
    }
}

/// Try to convert a `&Transaction` into a `DeployAccountTransactionAPI`.
impl TryInto<DeployAccountTransactionAPI> for &Transaction {
    type Error = StarknetApiError;

    fn try_into(self) -> Result<DeployAccountTransactionAPI, Self::Error> {
        match &self {
            Transaction::DeployAccount(deploy_account_tx) => {
                let _stark_felt_vec: Vec<StarkFelt> = deploy_account_tx.constructor_calldata.clone()
            .into_inner()
            .into_iter()
            .map(|felt_wrapper| felt_wrapper.try_into().unwrap()) // Here, we are assuming that the conversion will not fail.
            .collect();

                Ok(DeployAccountTransactionAPI {
                    transaction_hash: TransactionHash(StarkFelt::new(deploy_account_tx.transaction_hash.into())?),
                    max_fee: Fee(2),
                    version: TransactionVersion(StarkFelt::new(U256::from(deploy_account_tx.version).into())?),
                    signature: TransactionSignature(
                        deploy_account_tx
                            .signature
                            .clone()
                            .into_inner()
                            .iter()
                            .map(|x| StarkFelt::new((*x).into()).unwrap())
                            .collect(),
                    ),
                    nonce: Nonce(StarkFelt::new(deploy_account_tx.nonce.into())?),
                    contract_address: StarknetContractAddress(starknet_api::api_core::PatriciaKey(StarkFelt::new(
                        deploy_account_tx.sender_address.into(),
                    )?)),
                    class_hash: ClassHash(deploy_account_tx.class_hash.into()),
                    constructor_calldata: Calldata(std::sync::Arc::new(
                        deploy_account_tx
                            .constructor_calldata
                            .clone()
                            .into_inner()
                            .iter()
                            .map(|x| StarkFelt::new((*x).into()).unwrap())
                            .collect(),
                    )),
                    contract_address_salt: ContractAddressSalt(StarkFelt::new(
                        deploy_account_tx.contract_address_salt.into(),
                    )?),
                })
            }
            _ => Err(StarknetApiError::OutOfRange { string: String::from("InvalidTransactionType") }),
        }
    }
}
// /// Try to convert a `&Transaction` into a `L1HandlerTransaction`.
// impl TryInto<L1HandlerTransaction> for &Transaction {
//     type Error = StarknetApiError;

//     fn try_into(self) -> Result<L1HandlerTransaction, Self::Error> {
//         let entrypoint: CallEntryPoint = self.call_entrypoint.clone().try_into()?;

//         Ok(L1HandlerTransaction {
//             transaction_hash: TransactionHash(StarkFelt::new(self.hash.into())?),
//             version: TransactionVersion(StarkFelt::new(U256::from(self.version).into())?),
//             nonce: Nonce(StarkFelt::new(self.nonce.into())?),
//             contract_address:
// StarknetContractAddress::try_from(StarkFelt::new(self.sender_address.into())?)?,
// calldata: entrypoint.calldata,             entry_point_selector:
// EntryPointSelector(StarkHash::new(<[u8; 32]>::from(
// self.call_entrypoint.entrypoint_selector.unwrap_or_default(),             ))?),
//         })
//     }
// }

/// Try to convert a `&Transaction` into a `InvokeTransactionAPI`.
impl TryInto<InvokeTransactionAPI> for &Transaction {
    type Error = StarknetApiError;

    fn try_into(self) -> Result<InvokeTransactionAPI, Self::Error> {
        match self {
            Transaction::Invoke(invoke_tx) => match invoke_tx {
                InvokeTransaction::V1(invoke_tx_v1) => Ok(InvokeTransactionAPI::V1(InvokeTransactionAPIV1 {
                    transaction_hash: TransactionHash(StarkFelt::new(invoke_tx_v1.transaction_hash.into())?),
                    max_fee: Fee(2),
                    signature: TransactionSignature(
                        invoke_tx_v1
                            .signature
                            .clone()
                            .into_inner()
                            .iter()
                            .map(|x| StarkFelt::new((*x).into()).unwrap())
                            .collect(),
                    ),
                    nonce: Nonce(StarkFelt::new(invoke_tx_v1.nonce.into())?),
                    sender_address: StarknetContractAddress::try_from(StarkFelt::new(
                        invoke_tx_v1.sender_address.into(),
                    )?)?,
                    calldata: Calldata(Arc::new(
                        invoke_tx_v1
                            .calldata
                            .clone()
                            .into_inner()
                            .iter()
                            .map(|x| StarkFelt::new((*x).into()).unwrap())
                            .collect(),
                    )),
                })),
                InvokeTransaction::V0(_) => {
                    Err(StarknetApiError::OutOfRange { string: String::from("InvalidTransactionType") })
                }
            },
            Transaction::Declare(_declare_tx) => {
                Err(StarknetApiError::OutOfRange { string: String::from("InvalidTransactionType") })
            }
            Transaction::DeployAccount(_deploy_account_tx) => {
                Err(StarknetApiError::OutOfRange { string: String::from("InvalidTransactionType") })
            }
        }
    }
}
/// Try to convert a `&Transaction` into a `DeclareTransactionAPI`.
impl TryInto<DeclareTransactionAPI> for &Transaction {
    type Error = StarknetApiError;

    fn try_into(self) -> Result<DeclareTransactionAPI, Self::Error> {
        match self {
            Transaction::Declare(declare_tx) => {
                let tx = DeclareTransactionAPIV0V1 {
                    transaction_hash: TransactionHash(StarkFelt::new(declare_tx.transaction_hash().into())?),
                    max_fee: Fee(2),
                    signature: TransactionSignature(
                        declare_tx
                            .signature()
                            .clone()
                            .into_inner()
                            .iter()
                            .map(|x| StarkFelt::new((*x).into()).unwrap())
                            .collect(),
                    ),
                    nonce: Nonce(StarkFelt::new(declare_tx.nonce().into())?),
                    sender_address: StarknetContractAddress::try_from(StarkFelt::new(
                        declare_tx.sender_address().into(),
                    )?)?,
                    class_hash: ClassHash(declare_tx.class_hash().into()),
                };

                Ok(if declare_tx.version() == 0u8 {
                    DeclareTransactionAPI::V0(tx)
                } else if declare_tx.version() == 1u8 {
                    DeclareTransactionAPI::V1(tx)
                } else {
                    unimplemented!(
                        "DeclareTransactionAPIV2 required the compiled class hash. I don't know how to get it"
                    );
                })
            }
            _ => Err(StarknetApiError::OutOfRange { string: String::from("InvalidTransactionType") }),
        }
    }
}

impl Transaction {
    pub fn new_invoke(transaction: InvokeTransaction) -> Self {
        Transaction::Invoke(transaction)
    }

    pub fn new_declare(transaction: DeclareTransaction) -> Self {
        Transaction::Declare(transaction)
    }

    pub fn new_deploy_account(transaction: DeployAccountTransaction) -> Self {
        Transaction::DeployAccount(transaction)
    }

    /// Returns the validate entry point selector.
    pub fn validate_entry_point_selector(
        &self,
        tx_type: &TxType,
    ) -> TransactionValidationResultWrapper<EntryPointSelector> {
        match tx_type {
            TxType::Declare => Ok(*constants::VALIDATE_DECLARE_ENTRY_POINT_SELECTOR),
            TxType::DeployAccount => Ok(*constants::VALIDATE_DEPLOY_ENTRY_POINT_SELECTOR),
            TxType::Invoke => Ok(*constants::VALIDATE_ENTRY_POINT_SELECTOR),
            TxType::L1Handler => Err(EntryPointExecutionError::InvalidExecutionInput {
                input_descriptor: "tx_type".to_string(),
                info: "l1 handler transaction should not be validated".to_string(),
            })
            .map_err(TransactionValidationErrorWrapper::from),
        }
    }

    /// Calldata for validation contains transaction fields that cannot be obtained by calling
    /// `get_tx_info()`.
    pub fn validate_entrypoint_calldata(&self, tx_type: &TxType) -> TransactionValidationResultWrapper<Calldata> {
        match tx_type {
            TxType::Declare => {
                let declare_tx: DeclareTransactionAPI =
                    self.try_into().map_err(TransactionValidationErrorWrapper::CalldataError)?;
                Ok(calldata![declare_tx.class_hash().0])
            }
            TxType::DeployAccount => {
                let deploy_account_tx: DeployAccountTransactionAPI =
                    self.try_into().map_err(TransactionValidationErrorWrapper::CalldataError)?;
                let validate_calldata = vec![
                    vec![deploy_account_tx.class_hash.0, deploy_account_tx.contract_address_salt.0],
                    (*deploy_account_tx.constructor_calldata.0).clone(),
                ]
                .concat();
                Ok(Calldata(validate_calldata.into()))
            }
            // Calldata for validation is the same calldata as for the execution itself.
            TxType::Invoke => {
                let invoke_tx: InvokeTransactionAPI =
                    self.try_into().map_err(TransactionValidationErrorWrapper::CalldataError)?;
                Ok(Calldata(invoke_tx.calldata().0))
            }
            TxType::L1Handler => Err(EntryPointExecutionError::InvalidExecutionInput {
                input_descriptor: "tx_type".to_string(),
                info: "l1 handler transaction should not be validated".to_string(),
            })
            .map_err(TransactionValidationErrorWrapper::from),
        }
    }

    /// Validates account transaction
    ///
    /// # Arguments
    ///
    /// * `self` - The transaction to validate.
    /// * `state` - The state to validate the transaction on.
    /// * `execution_resources` - The execution resources to validate the transaction on.
    /// * `block_context` - The block context to validate the transaction on.
    /// * `tx_type` - The type of the transaction to execute.
    pub fn validate_account_tx<S: State>(
        &self,
        state: &mut S,
        execution_resources: &mut ExecutionResources,
        block_context: &BlockContext,
        tx_type: &TxType,
    ) -> TransactionValidationResultWrapper<Option<CallInfo>> {
        let account_context = match tx_type {
            TxType::Invoke => {
                let tx = self.try_into().map_err(TransactionValidationErrorWrapper::CalldataError)?;
                self.get_invoke_transaction_context(&tx)
            }
            TxType::Declare => {
                let tx = self.try_into().map_err(TransactionValidationErrorWrapper::CalldataError)?;
                self.get_declare_transaction_context(&tx)
            }
            TxType::L1Handler => {
                unimplemented!("L1Handler is not implemented for now in the transaction to enum")
            }
            TxType::DeployAccount => {
                let tx = self.try_into().map_err(TransactionValidationErrorWrapper::CalldataError)?;
                self.get_deploy_account_transaction_context(&tx)
            }
        };

        // FIXME 710
        let mut initial_gas = super::constants::INITIAL_GAS_COST.into();

        self.validate_tx(state, execution_resources, block_context, &account_context, tx_type, &mut initial_gas)
    }

    /// Validates a transaction
    ///
    /// # Arguments
    ///
    /// * `self` - The transaction to validate.
    /// * `state` - The state to validate the transaction on.
    /// * `execution_resources` - The execution resources to validate the transaction on.
    /// * `block_context` - The block context to validate the transaction on.
    /// * `account_tx_context` - The account transaction context to validate the transaction on.
    /// * `tx_type` - The type of the transaction to execute.
    pub fn validate_tx<S: State>(
        &self,
        state: &mut S,
        execution_resources: &mut ExecutionResources,
        block_context: &BlockContext,
        account_tx_context: &AccountTransactionContext,
        tx_type: &TxType,
        remaining_gas: &mut Felt252,
    ) -> TransactionValidationResultWrapper<Option<CallInfo>> {
        let mut context = EntryPointExecutionContext::new(
            block_context.clone(),
            account_tx_context.clone(),
            block_context.validate_max_n_steps,
        );
        if context.account_tx_context.is_v0() {
            return Ok(None);
        }

        let validate_call = CallEntryPoint {
            entry_point_type: EntryPointType::External,
            entry_point_selector: self.validate_entry_point_selector(tx_type)?,
            calldata: self.validate_entrypoint_calldata(tx_type)?,
            class_hash: None,
            code_address: None,
            storage_address: account_tx_context.sender_address,
            caller_address: StarknetContractAddress::default(),
            call_type: CallType::Call,
            initial_gas: remaining_gas.clone(),
        };

        let validate_call_info = validate_call
            .execute(state, execution_resources, &mut context)
            .map_err(TransactionValidationErrorWrapper::from)?;
        verify_no_calls_to_other_contracts(&validate_call_info, String::from(constants::VALIDATE_ENTRY_POINT_NAME))
            .map_err(TransactionValidationErrorWrapper::TransactionValidationError)?;

        update_remaining_gas(remaining_gas, &validate_call_info);

        Ok(Some(validate_call_info))
    }

    /// Verifies if a transaction has the correct version
    ///
    /// # Arguments
    ///
    /// * `self` - The transaction to execute
    /// * `tx_type` - The type of the transaction to execute
    ///
    /// # Returns
    ///
    /// * `TransactionExecutionResultWrapper<()>` - The result of the transaction version validation
    pub fn verify_tx_version(&self, tx_type: &TxType) -> TransactionExecutionResultWrapper<()> {
        let version = match StarkFelt::new(U256::from(self.get_version()).into()) {
            Ok(felt) => TransactionVersion(felt),
            Err(err) => {
                return Err(TransactionExecutionErrorWrapper::StarknetApi(err));
            }
        };

        let allowed_versions: vec::Vec<TransactionVersion> = match tx_type {
            TxType::Declare => {
                // Support old versions in order to allow bootstrapping of a new system.
                vec![
                    TransactionVersion(StarkFelt::from(0_u8)),
                    TransactionVersion(StarkFelt::from(1_u8)),
                    TransactionVersion(StarkFelt::from(2_u8)),
                ]
            }
            TxType::Invoke => {
                vec![TransactionVersion(StarkFelt::from(0_u8)), TransactionVersion(StarkFelt::from(1_u8))]
            }
            _ => vec![TransactionVersion(StarkFelt::from(1_u8))],
        };
        if allowed_versions.contains(&version) {
            Ok(())
        } else {
            Err(TransactionExecutionErrorWrapper::TransactionExecution(TransactionExecutionError::InvalidVersion {
                version,
                allowed_versions,
            }))
        }
    }

    /// Executes a transaction
    ///
    /// # Arguments
    ///
    /// * `self` - The transaction to execute.
    /// * `state` - The state to execute the transaction on.
    /// * `block` - The block to execute the transaction on.
    /// * `tx_type` - The type of the transaction to execute.
    /// * `contract_class` - The contract class to execute the transaction on.
    /// * `fee_token_address` - The fee token address.
    ///
    /// # Returns
    ///
    /// * `TransactionExecutionResult<TransactionExecutionInfo>` - The result of the transaction
    ///   execution
    pub fn execute<S: State + StateChanges>(
        &self,
        state: &mut S,
        block_context: &BlockContext,
        tx_type: TxType,
        contract_class: Option<ContractClass>,
    ) -> TransactionExecutionResultWrapper<TransactionExecutionInfoWrapper> {
        // Initialize the execution resources.
        let execution_resources = &mut ExecutionResources::default();

        // Verify the transaction version.
        self.verify_tx_version(&tx_type)?;

        // FIXME 710
        let mut initial_gas: Felt252 = super::constants::INITIAL_GAS_COST.into();

        // Going one lower level gives us more flexibility like not validating the tx as we could do
        // it before the tx lands in the mempool.
        // However it also means we need to copy/paste internal code from the tx.execute() method.
        let (execute_call_info, validate_call_info, account_context) = match tx_type {
            TxType::Invoke => {
                let tx: InvokeTransactionAPI =
                    self.try_into().map_err(TransactionExecutionErrorWrapper::StarknetApi)?;
                let account_context = self.get_invoke_transaction_context(&tx);

                // Create the context.
                let mut context = EntryPointExecutionContext::new(
                    block_context.clone(),
                    account_context.clone(),
                    block_context.invoke_tx_max_n_steps,
                );

                // Update nonce
                Self::handle_nonce(state, &account_context)?;

                // Validate.
                let validate_call_info = self.validate_tx(
                    state,
                    execution_resources,
                    block_context,
                    &account_context,
                    &tx_type,
                    &mut initial_gas,
                )?;

                // Execute.
                (
                    tx.run_execute(state, execution_resources, &mut context, &mut initial_gas)
                        .map_err(TransactionExecutionErrorWrapper::TransactionExecution)?,
                    validate_call_info,
                    account_context,
                )
            }
            TxType::L1Handler => {
                unimplemented!("L1Handler is not implemented for now in the transaction to enum")
            }
            TxType::Declare => {
                let tx = self.try_into().map_err(TransactionExecutionErrorWrapper::StarknetApi)?;
                let account_context = self.get_declare_transaction_context(&tx);
                let contract_class =
                    contract_class.ok_or_else(|| StateError::UndeclaredClassHash(ClassHash::default()))?;
                let tx = StarknetDeclareTransaction::new(tx, contract_class)?;

                // Create the context.
                let mut context = EntryPointExecutionContext::new(
                    block_context.clone(),
                    account_context.clone(),
                    block_context.invoke_tx_max_n_steps,
                );

                // Update nonce
                Self::handle_nonce(state, &account_context)?;

                // Validate.
                let validate_call_info = self.validate_tx(
                    state,
                    execution_resources,
                    block_context,
                    &account_context,
                    &tx_type,
                    &mut initial_gas,
                )?;

                // Execute.
                (
                    tx.run_execute(state, execution_resources, &mut context, &mut initial_gas)
                        .map_err(TransactionExecutionErrorWrapper::TransactionExecution)?,
                    validate_call_info,
                    account_context,
                )
            }
            TxType::DeployAccount => {
                let tx = self.try_into().map_err(TransactionExecutionErrorWrapper::StarknetApi)?;
                let account_context = self.get_deploy_account_transaction_context(&tx);

                // Create the context.
                let mut context = EntryPointExecutionContext::new(
                    block_context.clone(),
                    account_context.clone(),
                    block_context.invoke_tx_max_n_steps,
                );

                // Update nonce
                Self::handle_nonce(state, &account_context)?;

                // Execute.
                let transaction_execution = tx
                    .run_execute(state, execution_resources, &mut context, &mut initial_gas)
                    .map_err(TransactionExecutionErrorWrapper::TransactionExecution)?;

                (
                    transaction_execution,
                    self.validate_tx(
                        state,
                        execution_resources,
                        block_context,
                        &account_context,
                        &tx_type,
                        &mut initial_gas,
                    )?,
                    account_context,
                )
            }
        };
        let tx_resources = fees::get_transaction_resources(
            state,
            &execute_call_info,
            &validate_call_info,
            execution_resources,
            tx_type,
        )?;
        let (actual_fee, fee_transfer_call_info) = charge_fee(state, block_context, account_context, &tx_resources)?;
        Ok(TransactionExecutionInfoWrapper {
            validate_call_info,
            execute_call_info,
            fee_transfer_call_info,
            actual_fee,
            actual_resources: tx_resources,
        })
    }

    /// Handles the nonce of a transaction
    ///
    /// # Arguments
    ///
    /// * `self` - The transaction to handle the nonce for
    /// * `state` - The state to handle the nonce on
    /// * `account_tx_context` - The transaction context for the account
    ///
    /// # Returns
    ///
    /// * `TransactionExecutionResult<()>` - The result of the nonce handling
    pub fn handle_nonce(
        state: &mut dyn State,
        account_tx_context: &AccountTransactionContext,
    ) -> TransactionExecutionResultWrapper<()> {
        if account_tx_context.version == TransactionVersion(StarkFelt::from(0_u8)) {
            return Ok(());
        }

        let address = account_tx_context.sender_address;
        let current_nonce = state.get_nonce_at(address).map_err(TransactionExecutionErrorWrapper::StateError)?;
        if current_nonce != account_tx_context.nonce {
            return Err(TransactionExecutionErrorWrapper::TransactionExecution(
                TransactionExecutionError::InvalidNonce {
                    address,
                    expected_nonce: current_nonce,
                    actual_nonce: account_tx_context.nonce,
                },
            ));
        }

        // Increment nonce.
        state.increment_nonce(address).map_err(TransactionExecutionErrorWrapper::StateError)?;

        Ok(())
    }

    /// Get the transaction context for a l1 handler transaction
    ///
    /// # Arguments
    ///
    /// * `self` - The transaction to get the context for
    /// * `tx` - The l1 handler transaction to get the context for
    ///
    /// # Returns
    ///
    /// * `AccountTransactionContext` - The context of the transaction
    fn get_l1_handler_transaction_context(&self, tx: &L1HandlerTransaction) -> AccountTransactionContext {
        AccountTransactionContext {
            transaction_hash: tx.transaction_hash,
            max_fee: Fee::default(),
            version: tx.version,
            signature: TransactionSignature::default(),
            nonce: tx.nonce,
            sender_address: tx.contract_address,
        }
    }

    /// Get the transaction context for an invoke transaction
    ///
    /// # Arguments
    ///
    /// * `self` - The transaction to get the context for
    /// * `tx` - The invoke transaction to get the context for
    ///
    /// # Returns
    ///
    /// * `AccountTransactionContext` - The context of the transaction
    fn get_invoke_transaction_context(&self, tx: &InvokeTransactionAPI) -> AccountTransactionContext {
        AccountTransactionContext {
            transaction_hash: tx.transaction_hash(),
            max_fee: tx.max_fee(),
            version: TransactionVersion(StarkFelt::from(1_u8)),
            signature: tx.signature(),
            nonce: tx.nonce(),
            sender_address: tx.sender_address(),
        }
    }

    /// Get the transaction context for a deploy account transaction
    ///
    /// # Arguments
    ///
    /// * `self` - The transaction to get the context for
    /// * `tx` - The deploy transaction to get the context for
    ///
    /// # Returns
    ///
    /// * `AccountTransactionContext` - The context of the transaction
    fn get_deploy_account_transaction_context(&self, tx: &DeployAccountTransactionAPI) -> AccountTransactionContext {
        AccountTransactionContext {
            transaction_hash: tx.transaction_hash,
            max_fee: tx.max_fee,
            version: tx.version,
            signature: tx.signature.clone(),
            nonce: tx.nonce,
            sender_address: tx.contract_address,
        }
    }

    /// Get the transaction context for a declare transaction
    ///
    /// # Arguments
    ///
    /// * `self` - The transaction to get the context for
    /// * `tx` - The declare transaction to get the context for
    ///
    /// # Returns
    ///
    /// * `AccountTransactionContext` - The context of the transaction
    fn get_declare_transaction_context(&self, tx: &DeclareTransactionAPI) -> AccountTransactionContext {
        AccountTransactionContext {
            transaction_hash: tx.transaction_hash(),
            max_fee: tx.max_fee(),
            version: tx.version(),
            signature: tx.signature(),
            nonce: tx.nonce(),
            sender_address: tx.sender_address(),
        }
    }
}

impl Default for Transaction {
    fn default() -> Self {
        let one = Felt252Wrapper::ONE;
        Transaction::Invoke(InvokeTransaction::V1(InvokeTransactionV1 {
            transaction_hash: one,
            max_fee: Felt252Wrapper::from(u128::MAX),
            signature: BoundedVec::try_from(vec![one, one]).unwrap(),
            nonce: Felt252Wrapper::default(),
            sender_address: Felt252Wrapper::default(),
            calldata: BoundedVec::try_from(vec![one, one]).unwrap(),
        }))
    }
}

impl Default for TransactionReceiptWrapper {
    fn default() -> Self {
        Self {
            transaction_hash: Felt252Wrapper::default(),
            actual_fee: Felt252Wrapper::default(),
            tx_type: TxType::Invoke,
            events: BoundedVec::try_from(vec![EventWrapper::default(), EventWrapper::default()]).unwrap(),
        }
    }
}
