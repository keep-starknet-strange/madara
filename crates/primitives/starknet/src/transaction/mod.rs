//! Starknet transaction related functionality.
/// Constants related to transactions.
pub mod constants;
/// Types related to transactions.
pub mod types;

use alloc::string::{String, ToString};
use alloc::vec;

use blockifier::block_context::BlockContext;
use blockifier::execution::contract_class::ContractClass;
use blockifier::execution::entry_point::{CallEntryPoint, CallInfo, CallType, ExecutionContext, ExecutionResources};
use blockifier::execution::errors::EntryPointExecutionError;
use blockifier::state::state_api::State;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::AccountTransactionContext;
use blockifier::transaction::transaction_utils::verify_no_calls_to_other_contracts;
use frame_support::BoundedVec;
use sp_core::U256;
use starknet_api::api_core::{ContractAddress as StarknetContractAddress, EntryPointSelector, Nonce};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{
    Calldata, DeployAccountTransaction as StarknetDeployAccountTransaction, EventContent, Fee,
    InvokeTransactionV1 as StarknetInvokeTransactionV1, L1HandlerTransaction as StarknetL1HandlerTransaction,
    TransactionHash, TransactionOutput, TransactionReceipt, TransactionSignature, TransactionVersion,
};
use starknet_api::StarknetApiError;

use self::types::{
    DeclareTransaction, DeclareTransactionV1, DeclareTransactionV2, DeployAccountTransaction, EventError, EventWrapper,
    InvokeTransaction, InvokeTransactionV0, InvokeTransactionV1, L1HandlerTransaction, MaxArraySize, Transaction,
    TransactionError, TransactionExecutionErrorWrapper, TransactionExecutionInfoWrapper,
    TransactionExecutionResultWrapper, TransactionReceiptWrapper, TransactionValidationErrorWrapper,
    TransactionValidationResultWrapper, TxType,
};
use crate::block::serialize::SerializeBlockContext;
use crate::block::Block as StarknetBlock;
use crate::execution::call_entrypoint_wrapper::MaxCalldataSize;
use crate::execution::types::{CallEntryPointWrapper, ContractAddressWrapper, ContractClassWrapper, Felt252Wrapper};
use crate::fees::{self, charge_fee};
use crate::state::StateChanges;
use crate::tests::transaction;

impl EventWrapper {
    /// Creates a new instance of an event.
    ///
    /// # Arguments
    ///
    /// * `keys` - Event keys.
    /// * `data` - Event data.
    /// * `from_address` - Contract Address where the event was emitted from.
    pub fn new(
        keys: BoundedVec<Felt252Wrapper, MaxArraySize>,
        data: BoundedVec<Felt252Wrapper, MaxArraySize>,
        from_address: ContractAddressWrapper,
    ) -> Self {
        Self { keys, data, from_address }
    }

    /// Creates an empty event.
    pub fn empty() -> Self {
        Self {
            keys: BoundedVec::try_from(vec![]).unwrap(),
            data: BoundedVec::try_from(vec![]).unwrap(),
            from_address: ContractAddressWrapper::default(),
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
        })
    }
}

impl Default for EventWrapper {
    fn default() -> Self {
        let one = Felt252Wrapper::one();
        Self {
            keys: BoundedVec::try_from(vec![one, one]).unwrap(),
            data: BoundedVec::try_from(vec![one, one]).unwrap(),
            from_address: one,
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
            block_hash: self.block_hash.0.into(),
            block_number: self.block_number.0,
            events: BoundedVec::try_from(_events?).map_err(|_| EventError::TooManyEvents)?,
        })
    }
}

/// Try to convert a `&Transaction` into a `DeployAccountTransaction`.
impl TryInto<StarknetDeployAccountTransaction> for &Transaction {
    type Error = StarknetApiError;

    fn try_into(self) -> Result<StarknetDeployAccountTransaction, Self::Error> {
        match self {
            Transaction::DeployAccount(tx) => {}
            _ => Err(StarknetApiError::InvalidTransactionType)?,
        }

        Ok(StarknetDeployAccountTransaction {
            transaction_hash: TransactionHash(StarkFelt::new(self..into())?),
            max_fee: Fee(2),
            version: TransactionVersion(StarkFelt::new(U256::from(self.version).into())?),
            signature: TransactionSignature(
                self.signature.clone().into_inner().iter().map(|x| StarkFelt::new((*x).into()).unwrap()).collect(),
            ),
            nonce: Nonce(StarkFelt::new(self.nonce.into())?),
            contract_address: StarknetContractAddress::try_from(StarkFelt::new(self.sender_address.into())?)?,
            class_hash: entrypoint.class_hash.unwrap_or_default(),
            constructor_calldata: entrypoint.calldata,
            contract_address_salt: ContractAddressSalt(StarkFelt::new(
                self.contract_address_salt.unwrap_or_default().into(),
            )?),
        })
    }
}

/// Try to convert a `&Transaction` into a `L1HandlerTransaction`.
impl TryInto<L1HandlerTransaction> for &Transaction {
    type Error = StarknetApiError;

    fn try_into(self) -> Result<L1HandlerTransaction, Self::Error> {
        let entrypoint: CallEntryPoint = self.call_entrypoint.clone().try_into()?;

        Ok(L1HandlerTransaction {
            transaction_hash: TransactionHash(StarkFelt::new(self.hash.into())?),
            version: TransactionVersion(StarkFelt::new(U256::from(self.version).into())?),
            nonce: Nonce(StarkFelt::new(self.nonce.into())?),
            contract_address: StarknetContractAddress::try_from(StarkFelt::new(self.sender_address.into())?)?,
            calldata: entrypoint.calldata,
            entry_point_selector: EntryPointSelector(StarkHash::new(<[u8; 32]>::from(
                self.call_entrypoint.entrypoint_selector.unwrap_or_default(),
            ))?),
        })
    }
}

/// Try to convert a `&Transaction` into a `InvokeTransaction`.
impl TryInto<InvokeTransactionV1> for &Transaction {
    type Error = StarknetApiError;

    fn try_into(self) -> Result<InvokeTransactionV1, Self::Error> {
        let entrypoint: CallEntryPoint = self.call_entrypoint.clone().try_into()?;

        Ok(InvokeTransactionV1 {
            transaction_hash: TransactionHash(StarkFelt::new(self.hash.into())?),
            max_fee: Fee(2),
            signature: TransactionSignature(
                self.signature.clone().into_inner().iter().map(|x| StarkFelt::new((*x).into()).unwrap()).collect(),
            ),
            nonce: Nonce(StarkFelt::new(self.nonce.into())?),
            sender_address: StarknetContractAddress::try_from(StarkFelt::new(self.sender_address.into())?)?,
            calldata: entrypoint.calldata,
        })
    }
}

impl Transaction {
    /// Creates a new instance of a transaction.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tx_type: TxType,
        version: u64,
        hash: Felt252Wrapper,
        signature: BoundedVec<Felt252Wrapper, MaxArraySize>,
        sender_address: ContractAddressWrapper,
        nonce: Felt252Wrapper,
        entrypoint: Option<Felt252Wrapper>,
        calldata: BoundedVec<Felt252Wrapper, MaxCalldataSize>,
        contract_class: Option<ContractClassWrapper>,
        contract_class_hash: Option<Felt252Wrapper>,
        compiled_contract_class_hash: Option<Felt252Wrapper>,
        contract_address_salt: Option<Felt252Wrapper>,
        max_fee: Felt252Wrapper,
    ) -> Result<Self, TransactionError> {
        match tx_type {
            TxType::Invoke => match version {
                _ if version == 0 => Ok(Transaction::Invoke(types::InvokeTransaction::V0(InvokeTransactionV0 {
                    transaction_hash: hash,
                    max_fee,
                    signature,
                    contract_address: sender_address,
                    nonce,
                    entry_point_selector: entrypoint.ok_or_else(|| TransactionError::MissingInput)?,
                    calldata,
                }))),
                _ if version == 1 => Ok(Transaction::Invoke(types::InvokeTransaction::V1(InvokeTransactionV1 {
                    transaction_hash: hash,
                    max_fee,
                    signature,
                    sender_address,
                    nonce,
                    calldata,
                }))),
                _ => Err(TransactionError::InvalidVersion),
            },
            TxType::L1Handler => Ok(Transaction::L1Handler(types::L1HandlerTransaction {
                transaction_hash: hash,
                version,
                nonce: u64::try_from(nonce.0).map_err(|_| TransactionError::InvalidData)?,
                contract_address: sender_address,
                entry_point_selector: entrypoint.ok_or_else(|| TransactionError::MissingInput)?,
                calldata,
            })),
            TxType::Declare => match version {
                _ if version == 1 => Ok(Transaction::Declare(types::DeclareTransaction::V1(DeclareTransactionV1 {
                    transaction_hash: hash,
                    max_fee,
                    signature,
                    sender_address,
                    nonce,
                    contract_class: contract_class.ok_or_else(|| TransactionError::MissingInput)?,
                    class_hash: contract_class_hash.ok_or_else(|| TransactionError::MissingInput)?,
                }))),
                _ if version == 2 => Ok(Transaction::Declare(types::DeclareTransaction::V2(DeclareTransactionV2 {
                    transaction_hash: hash,
                    max_fee,
                    signature,
                    nonce,
                    sender_address,
                    class_hash: contract_class_hash.ok_or_else(|| TransactionError::MissingInput)?,
                    compiled_class_hash: compiled_contract_class_hash.ok_or_else(|| TransactionError::MissingInput)?,
                    contract_class: contract_class.ok_or_else(|| TransactionError::MissingInput)?,
                }))),
                _ => Err(TransactionError::InvalidVersion),
            },
            TxType::DeployAccount => Ok(Transaction::DeployAccount(types::DeployAccountTransaction {
                transaction_hash: hash,
                max_fee,
                signature,
                nonce,
                sender_address,
                contract_address_salt: contract_address_salt.ok_or_else(|| TransactionError::MissingInput)?,
                class_hash: contract_class_hash.ok_or_else(|| TransactionError::MissingInput)?,
                constructor_calldata: calldata,
            })),
        }
    }

    /// Returns the validate entry point selector.
    pub fn validate_entry_point_selector(&self) -> TransactionValidationResultWrapper<EntryPointSelector> {
        match self {
            Transaction::Declare(_) => Ok(*constants::VALIDATE_DECLARE_ENTRY_POINT_SELECTOR),
            Transaction::DeployAccount(_) => Ok(*constants::VALIDATE_DEPLOY_ENTRY_POINT_SELECTOR),
            Transaction::Invoke(_) => Ok(*constants::VALIDATE_ENTRY_POINT_SELECTOR),
            Transaction::L1Handler(_) => Err(EntryPointExecutionError::InvalidExecutionInput {
                input_descriptor: "tx_type".to_string(),
                info: "l1 handler transaction should not be validated".to_string(),
            })
            .map_err(TransactionValidationErrorWrapper::from),
        }
    }

    /// Calldata for validation contains transaction fields that cannot be obtained by calling
    /// `get_tx_info()`.
    pub fn validate_entrypoint_calldata(&self) -> TransactionValidationResultWrapper<Calldata> {
        match self {
            Transaction::Declare(tx) => {
                let declare_calldata = vec![tx.class_hash()];
                Ok(Calldata(
                    declare_calldata.into_iter().map(Into::<StarkFelt>::into).collect::<Vec<StarkFelt>>().into(),
                ))
            }
            Transaction::DeployAccount(tx) => {
                let validate_calldata =
                    vec![vec![tx.class_hash, tx.contract_address_salt], (tx.constructor_calldata).into()].concat();
                Ok(Calldata(
                    validate_calldata.into_iter().map(Into::<StarkFelt>::into).collect::<Vec<StarkFelt>>().into(),
                ))
            }
            // Calldata for validation is the same calldata as for the execution itself.
            Transaction::Invoke(tx) => Ok(Calldata(
                tx.calldata().to_vec().into_iter().map(Into::<StarkFelt>::into).collect::<Vec<StarkFelt>>().into(),
            )),
            Transaction::L1Handler(_) => Err(EntryPointExecutionError::InvalidExecutionInput {
                input_descriptor: "tx_type".to_string(),
                info: "l1 handler transaction should not be validated".to_string(),
            })
            .map_err(TransactionValidationErrorWrapper::from),
        }
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
    ) -> TransactionValidationResultWrapper<Option<CallInfo>> {
        let validate_call = CallEntryPoint {
            entry_point_type: EntryPointType::External,
            entry_point_selector: self.validate_entry_point_selector()?,
            calldata: self.validate_entrypoint_calldata()?,
            class_hash: None,
            code_address: None,
            storage_address: account_tx_context.sender_address,
            caller_address: StarknetContractAddress::default(),
            call_type: CallType::Call,
        };
        let mut execution_context = ExecutionContext::default();

        let validate_call_info = validate_call
            .execute(state, execution_resources, &mut execution_context, block_context, account_tx_context)
            .map_err(TransactionValidationErrorWrapper::from)?;
        verify_no_calls_to_other_contracts(&validate_call_info, String::from(constants::VALIDATE_ENTRY_POINT_NAME))
            .map_err(TransactionValidationErrorWrapper::TransactionValidationError)?;

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
    pub fn verify_tx_version(&self) -> TransactionExecutionResultWrapper<()> {
        let version = match StarkFelt::new(U256::from(self.get_transaction_version()).into()) {
            Ok(felt) => TransactionVersion(felt),
            Err(err) => {
                return Err(TransactionExecutionErrorWrapper::StarknetApi(err));
            }
        };

        let allowed_versions: vec::Vec<TransactionVersion> = match self {
            Transaction::Declare(_) => {
                // Support old versions in order to allow bootstrapping of a new system.
                vec![TransactionVersion(StarkFelt::from(0)), TransactionVersion(StarkFelt::from(1))]
            }
            _ => vec![TransactionVersion(StarkFelt::from(1))],
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
        block: StarknetBlock,
        contract_class: Option<ContractClass>,
        fee_token_address: ContractAddressWrapper,
    ) -> TransactionExecutionResultWrapper<TransactionExecutionInfoWrapper> {
        // Create the block context.
        // TODO: don't do that.
        // FIXME: https://github.com/keep-starknet-strange/madara/issues/330
        let block_context = BlockContext::try_serialize(block.header().clone(), fee_token_address)
            .map_err(|_| TransactionExecutionErrorWrapper::BlockContextSerializationError)?;

        // Initialize the execution resources.
        let execution_resources = &mut ExecutionResources::default();

        // Verify the transaction version.
        self.verify_tx_version()?;

        // Going one lower level gives us more flexibility like not validating the tx as we could do
        // it before the tx lands in the mempool.
        // However it also means we need to copy/paste internal code from the tx.execute() method.
        let account_context = self.get_transaction_context()?;
        let (execute_call_info, validate_call_info, account_context) = match self {
            Transaction::Invoke(_) => {
                let tx: InvokeTransactionV1 = self.try_into().map_err(TransactionExecutionErrorWrapper::StarknetApi)?;
                // Update nonce
                self.handle_nonce(state, &account_context)?;

                // Validate.
                let validate_call_info =
                    self.validate_tx(state, execution_resources, &block_context, &account_context)?;

                // Execute.
                (
                    tx.run_execute(state, execution_resources, &block_context, &account_context, contract_class)
                        .map_err(TransactionExecutionErrorWrapper::TransactionExecution)?,
                    validate_call_info,
                    account_context,
                )
            }
            TxType::L1Handler => {
                let tx = self.try_into().map_err(TransactionExecutionErrorWrapper::StarknetApi)?;
                (
                    tx.run_execute(state, execution_resources, &block_context, &account_context, contract_class)
                        .map_err(TransactionExecutionErrorWrapper::TransactionExecution)?,
                    None,
                    account_context,
                )
            }
            TxType::Declare => {
                let tx = self.try_into().map_err(TransactionExecutionErrorWrapper::StarknetApi)?;

                // Update nonce
                self.handle_nonce(state, &account_context)?;

                // Validate.
                let validate_call_info =
                    self.validate_tx(state, execution_resources, &block_context, &account_context, &tx_type)?;

                // Execute.
                (
                    tx.run_execute(state, execution_resources, &block_context, &account_context, contract_class)
                        .map_err(TransactionExecutionErrorWrapper::TransactionExecution)?,
                    validate_call_info,
                    account_context,
                )
            }
            TxType::DeployAccount => {
                let tx = self.try_into().map_err(TransactionExecutionErrorWrapper::StarknetApi)?;
                // Update nonce
                self.handle_nonce(state, &account_context)?;

                // Execute.
                let transaction_execution = tx
                    .run_execute(state, execution_resources, &block_context, &account_context, contract_class)
                    .map_err(TransactionExecutionErrorWrapper::TransactionExecution)?;

                (
                    transaction_execution,
                    self.validate_tx(state, execution_resources, &block_context, &account_context, &tx_type)?,
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
        let (actual_fee, fee_transfer_call_info) = charge_fee(state, &block_context, &account_context, &tx_resources)?;
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
        &self,
        state: &mut dyn State,
        account_tx_context: &AccountTransactionContext,
    ) -> TransactionExecutionResultWrapper<()> {
        if account_tx_context.version == TransactionVersion(StarkFelt::from(0)) {
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

    /// Get the transaction version
    ///
    /// # Arguments
    ///
    /// * `self` - The transaction to get the context for
    ///
    /// # Returns
    ///
    /// * `AccountTransactionContext` - The context of the transaction
    fn get_transaction_version(&self) -> Felt252Wrapper {
        match self {
            Transaction::Invoke(tx) => tx.version(),
            Transaction::L1Handler(tx) => Felt252Wrapper::from(tx.version),
            Transaction::Declare(tx) => tx.version(),
            Transaction::DeployAccount(tx) => Felt252Wrapper::one(),
        }
    }

    /// Get the transaction context
    ///
    /// # Arguments
    ///
    /// * `self` - The transaction to get the context for
    ///
    /// # Returns
    ///
    /// * `AccountTransactionContext` - The context of the transaction
    fn get_transaction_context(&self) -> Result<AccountTransactionContext, StarknetApiError> {
        match self {
            Transaction::Invoke(tx) => Transaction::get_invoke_transaction_context(tx),
            Transaction::L1Handler(tx) => Transaction::get_l1_handler_transaction_context(tx),
            Transaction::Declare(tx) => Transaction::get_declare_transaction_context(tx),
            Transaction::DeployAccount(tx) => Transaction::get_deploy_account_transaction_context(tx),
        }
    }

    /// Get the transaction context for a l1 handler transaction
    ///
    /// # Arguments
    ///
    /// * `self` - The transaction to get the context for
    ///
    /// # Returns
    ///
    /// * `AccountTransactionContext` - The context of the transaction
    fn get_l1_handler_transaction_context(
        tx: &L1HandlerTransaction,
    ) -> Result<AccountTransactionContext, StarknetApiError> {
        Ok(AccountTransactionContext {
            transaction_hash: TransactionHash(tx.transaction_hash.into()),
            max_fee: Fee::default(),
            version: TransactionVersion(tx.version.into()),
            signature: TransactionSignature::default(),
            nonce: Nonce(Into::<StarkFelt>::into(tx.nonce)),
            sender_address: StarknetContractAddress(Into::<StarkFelt>::into(tx.contract_address).try_into()?),
        })
    }

    /// Get the transaction context for an invoke transaction
    ///
    /// # Arguments
    ///
    /// * `tx` - The invoke transaction to get the context for
    ///
    /// # Returns
    ///
    /// * `AccountTransactionContext` - The context of the transaction
    fn get_invoke_transaction_context(tx: &InvokeTransaction) -> Result<AccountTransactionContext, StarknetApiError> {
        Ok(AccountTransactionContext {
            transaction_hash: TransactionHash(tx.transaction_hash().into()),
            max_fee: Fee(tx.max_fee().try_into()?),
            version: TransactionVersion(StarkFelt::from(tx.version())),
            signature: TransactionSignature(tx.signature().to_vec().into_iter().map(Into::<StarkFelt>::into).collect()),
            nonce: Nonce(tx.nonce().into()),
            sender_address: StarknetContractAddress(Into::<StarkFelt>::into(tx.sender_address()).try_into()?),
        })
    }

    /// Get the transaction context for a deploy account transaction
    ///
    /// # Arguments
    ///
    /// * `tx` - The deploy transaction to get the context for
    ///
    /// # Returns
    ///
    /// * `AccountTransactionContext` - The context of the transaction
    fn get_deploy_account_transaction_context(
        tx: &DeployAccountTransaction,
    ) -> Result<AccountTransactionContext, StarknetApiError> {
        Ok(AccountTransactionContext {
            transaction_hash: TransactionHash(tx.transaction_hash.into()),
            max_fee: Fee(tx.max_fee.try_into()?),
            version: TransactionVersion(tx.version().into()),
            signature: TransactionSignature(tx.signature.to_vec().into_iter().map(Into::<StarkFelt>::into).collect()),
            nonce: Nonce(tx.nonce.into()),
            sender_address: StarknetContractAddress(Into::<StarkFelt>::into(tx.sender_address).try_into()?),
        })
    }

    /// Get the transaction context for a declare transaction
    ///
    /// # Arguments
    ///
    /// * `tx` - The declare transaction to get the context for
    ///
    /// # Returns
    ///
    /// * `AccountTransactionContext` - The context of the transaction
    fn get_declare_transaction_context(tx: &DeclareTransaction) -> Result<AccountTransactionContext, StarknetApiError> {
        Ok(AccountTransactionContext {
            transaction_hash: TransactionHash(tx.transaction_hash().into()),
            max_fee: Fee(tx.max_fee().try_into()?),
            version: TransactionVersion(tx.version().into()),
            signature: TransactionSignature(tx.signature().to_vec().into_iter().map(Into::<StarkFelt>::into).collect()),
            nonce: Nonce(tx.nonce().into()),
            sender_address: StarknetContractAddress(Into::<StarkFelt>::into(tx.sender_address()).try_into()?),
        })
    }
}

impl Default for TransactionReceiptWrapper {
    fn default() -> Self {
        Self {
            transaction_hash: Felt252Wrapper::default(),
            actual_fee: Felt252Wrapper::default(),
            tx_type: TxType::Invoke,
            block_hash: Felt252Wrapper::default(),
            block_number: 0_u64,
            events: BoundedVec::try_from(vec![EventWrapper::default(), EventWrapper::default()]).unwrap(),
        }
    }
}
