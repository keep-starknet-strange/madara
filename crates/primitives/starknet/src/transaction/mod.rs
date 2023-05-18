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
use blockifier::transaction::transactions::Executable;
use frame_support::BoundedVec;
use sp_core::{H256, U256};
use starknet_api::api_core::{ContractAddress as StarknetContractAddress, EntryPointSelector, Nonce};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{
    Calldata, ContractAddressSalt, DeclareTransaction, DeclareTransactionV0V1, DeployAccountTransaction, EventContent,
    Fee, InvokeTransactionV1, L1HandlerTransaction, TransactionHash, TransactionOutput, TransactionReceipt,
    TransactionSignature, TransactionVersion,
};
use starknet_api::{calldata, StarknetApiError};

use self::types::{
    EventError, EventWrapper, MaxArraySize, Transaction, TransactionExecutionErrorWrapper,
    TransactionExecutionInfoWrapper, TransactionExecutionResultWrapper, TransactionReceiptWrapper,
    TransactionValidationErrorWrapper, TransactionValidationResultWrapper, TxType,
};
use crate::block::serialize::SerializeBlockContext;
use crate::block::Block as StarknetBlock;
use crate::execution::types::{CallEntryPointWrapper, ContractAddressWrapper, ContractClassWrapper};
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
    pub fn new(
        keys: BoundedVec<H256, MaxArraySize>,
        data: BoundedVec<H256, MaxArraySize>,
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
    keys: vec::Vec<H256>,
    data: vec::Vec<H256>,
    from_address: Option<StarknetContractAddress>,
}

impl EventBuilder {
    /// Sets the keys of the event.
    ///
    /// # Arguments
    ///
    /// * `keys` - Event keys.
    pub fn with_keys(mut self, keys: vec::Vec<H256>) -> Self {
        self.keys = keys;
        self
    }

    /// Sets the data of the event.
    ///
    /// # Arguments
    ///
    /// * `data` - Event data.
    pub fn with_data(mut self, data: vec::Vec<H256>) -> Self {
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
        self.keys = event_content.keys.iter().map(|k| H256::from_slice(k.0.bytes())).collect::<vec::Vec<H256>>();
        self.data = event_content.data.0.iter().map(|d| H256::from_slice(d.bytes())).collect::<vec::Vec<H256>>();
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
        let one = H256::from_slice(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
        ]);
        Self {
            keys: BoundedVec::try_from(vec![one, one]).unwrap(),
            data: BoundedVec::try_from(vec![one, one]).unwrap(),
            from_address: ContractAddressWrapper::from(one),
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
            transaction_hash: H256::from_slice(self.transaction_hash.0.bytes()),
            actual_fee: U256::from(self.output.actual_fee().0),
            tx_type: match self.output {
                TransactionOutput::Declare(_) => TxType::Declare,
                TransactionOutput::DeployAccount(_) => TxType::DeployAccount,
                TransactionOutput::Invoke(_) => TxType::Invoke,
                TransactionOutput::L1Handler(_) => TxType::L1Handler,
                _ => TxType::Invoke,
            },
            block_hash: U256::from(self.block_hash.0.0),
            block_number: self.block_number.0,
            events: BoundedVec::try_from(_events?).map_err(|_| EventError::TooManyEvents)?,
        })
    }
}

/// Try to convert a `&Transaction` into a `DeployAccountTransaction`.
impl TryInto<DeployAccountTransaction> for &Transaction {
    type Error = StarknetApiError;

    fn try_into(self) -> Result<DeployAccountTransaction, Self::Error> {
        let entrypoint: CallEntryPoint = self.call_entrypoint.clone().try_into()?;

        Ok(DeployAccountTransaction {
            transaction_hash: TransactionHash(StarkFelt::new(self.hash.0)?),
            max_fee: Fee(2),
            version: TransactionVersion(StarkFelt::new(U256::from(self.version).into())?),
            signature: TransactionSignature(
                self.signature.clone().into_inner().iter().map(|x| StarkFelt::new(x.0).unwrap()).collect(),
            ),
            nonce: Nonce(StarkFelt::new(self.nonce.into())?),
            contract_address: StarknetContractAddress::try_from(StarkFelt::new(self.sender_address)?)?,
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
            transaction_hash: TransactionHash(StarkFelt::new(self.hash.0)?),
            version: TransactionVersion(StarkFelt::new(U256::from(self.version).into())?),
            nonce: Nonce(StarkFelt::new(self.nonce.into())?),
            contract_address: StarknetContractAddress::try_from(StarkFelt::new(self.sender_address)?)?,
            calldata: entrypoint.calldata,
            entry_point_selector: EntryPointSelector(StarkHash::new(
                *self.call_entrypoint.entrypoint_selector.unwrap_or_default().as_fixed_bytes(),
            )?),
        })
    }
}

/// Try to convert a `&Transaction` into a `InvokeTransaction`.
impl TryInto<InvokeTransactionV1> for &Transaction {
    type Error = StarknetApiError;

    fn try_into(self) -> Result<InvokeTransactionV1, Self::Error> {
        let entrypoint: CallEntryPoint = self.call_entrypoint.clone().try_into()?;

        Ok(InvokeTransactionV1 {
            transaction_hash: TransactionHash(StarkFelt::new(self.hash.0)?),
            max_fee: Fee(2),
            signature: TransactionSignature(
                self.signature.clone().into_inner().iter().map(|x| StarkFelt::new(x.0).unwrap()).collect(),
            ),
            nonce: Nonce(StarkFelt::new(self.nonce.into())?),
            sender_address: StarknetContractAddress::try_from(StarkFelt::new(self.sender_address)?)?,
            calldata: entrypoint.calldata,
        })
    }
}

/// Try to convert a `&Transaction` into a `DeclareTransaction`.
impl TryInto<DeclareTransaction> for &Transaction {
    type Error = StarknetApiError;

    fn try_into(self) -> Result<DeclareTransaction, Self::Error> {
        let entrypoint: CallEntryPoint = self.call_entrypoint.clone().try_into()?;

        let tx = DeclareTransactionV0V1 {
            transaction_hash: TransactionHash(StarkFelt::new(self.hash.0)?),
            max_fee: Fee(2),
            signature: TransactionSignature(
                self.signature.clone().into_inner().iter().map(|x| StarkFelt::new(x.0).unwrap()).collect(),
            ),
            nonce: Nonce(StarkFelt::new(self.nonce.into())?),
            sender_address: StarknetContractAddress::try_from(StarkFelt::new(self.sender_address)?)?,
            class_hash: entrypoint.class_hash.unwrap_or_default(),
        };

        Ok(if self.version == 0_u8 {
            DeclareTransaction::V0(tx)
        } else if self.version == 1_u8 {
            DeclareTransaction::V1(tx)
        } else {
            unimplemented!("DeclareTransactionV2 required the compiled class hash. I don't know how to get it");
        })
    }
}

impl Transaction {
    /// Creates a new instance of a transaction.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: u8,
        hash: H256,
        signature: BoundedVec<H256, MaxArraySize>,
        sender_address: ContractAddressWrapper,
        nonce: U256,
        call_entrypoint: CallEntryPointWrapper,
        contract_class: Option<ContractClassWrapper>,
        contract_address_salt: Option<U256>,
        max_fee: U256,
    ) -> Self {
        Self {
            version,
            hash,
            signature,
            sender_address,
            nonce,
            call_entrypoint,
            contract_class,
            contract_address_salt,
            max_fee,
        }
    }

    /// Creates a new instance of a transaction without signature.
    pub fn from_tx_hash(hash: H256) -> Self {
        Self { hash, ..Self::default() }
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
                let declare_tx: DeclareTransaction =
                    self.try_into().map_err(TransactionValidationErrorWrapper::CalldataError)?;
                Ok(calldata![declare_tx.class_hash().0])
            }
            TxType::DeployAccount => {
                let deploy_account_tx: DeployAccountTransaction =
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
                let invoke_tx: InvokeTransactionV1 =
                    self.try_into().map_err(TransactionValidationErrorWrapper::CalldataError)?;
                Ok(Calldata(invoke_tx.calldata.0))
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
                let tx = self.try_into().map_err(TransactionValidationErrorWrapper::CalldataError)?;
                self.get_l1_handler_transaction_context(&tx)
            }
            TxType::DeployAccount => {
                let tx = self.try_into().map_err(TransactionValidationErrorWrapper::CalldataError)?;
                self.get_deploy_account_transaction_context(&tx)
            }
        };

        self.validate_tx(state, execution_resources, block_context, &account_context, tx_type)
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
    ) -> TransactionValidationResultWrapper<Option<CallInfo>> {
        let validate_call = CallEntryPoint {
            entry_point_type: EntryPointType::External,
            entry_point_selector: self.validate_entry_point_selector(tx_type)?,
            calldata: self.validate_entrypoint_calldata(tx_type)?,
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
    pub fn verify_tx_version(&self, tx_type: &TxType) -> TransactionExecutionResultWrapper<()> {
        let version = match StarkFelt::new(U256::from(self.version).into()) {
            Ok(felt) => TransactionVersion(felt),
            Err(err) => {
                return Err(TransactionExecutionErrorWrapper::StarknetApi(err));
            }
        };

        let allowed_versions: vec::Vec<TransactionVersion> = match tx_type {
            TxType::Declare => {
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
        tx_type: TxType,
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
        self.verify_tx_version(&tx_type)?;

        // Going one lower level gives us more flexibility like not validating the tx as we could do
        // it before the tx lands in the mempool.
        // However it also means we need to copy/paste internal code from the tx.execute() method.
        let (execute_call_info, validate_call_info, account_context) = match tx_type {
            TxType::Invoke => {
                let tx: InvokeTransactionV1 = self.try_into().map_err(TransactionExecutionErrorWrapper::StarknetApi)?;
                let account_context = self.get_invoke_transaction_context(&tx);
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
            TxType::L1Handler => {
                let tx = self.try_into().map_err(TransactionExecutionErrorWrapper::StarknetApi)?;
                let account_context = self.get_l1_handler_transaction_context(&tx);
                (
                    tx.run_execute(state, execution_resources, &block_context, &account_context, contract_class)
                        .map_err(TransactionExecutionErrorWrapper::TransactionExecution)?,
                    None,
                    account_context,
                )
            }
            TxType::Declare => {
                let tx = self.try_into().map_err(TransactionExecutionErrorWrapper::StarknetApi)?;
                let account_context = self.get_declare_transaction_context(&tx);

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
                let account_context = self.get_deploy_account_transaction_context(&tx);
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
    fn get_invoke_transaction_context(&self, tx: &InvokeTransactionV1) -> AccountTransactionContext {
        AccountTransactionContext {
            transaction_hash: tx.transaction_hash,
            max_fee: tx.max_fee,
            version: TransactionVersion(StarkFelt::from(1)),
            signature: tx.signature.clone(),
            nonce: tx.nonce,
            sender_address: tx.sender_address,
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
    fn get_deploy_account_transaction_context(&self, tx: &DeployAccountTransaction) -> AccountTransactionContext {
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
    fn get_declare_transaction_context(&self, tx: &DeclareTransaction) -> AccountTransactionContext {
        // TODO: use lib implem once this PR is merged: https://github.com/starkware-libs/starknet-api/pull/49
        let version = match tx {
            DeclareTransaction::V0(_) => TransactionVersion(StarkFelt::from(0)),
            DeclareTransaction::V1(_) => TransactionVersion(StarkFelt::from(1)),
            DeclareTransaction::V2(_) => TransactionVersion(StarkFelt::from(2)),
        };

        AccountTransactionContext {
            transaction_hash: tx.transaction_hash(),
            max_fee: tx.max_fee(),
            version,
            signature: tx.signature(),
            nonce: tx.nonce(),
            sender_address: tx.sender_address(),
        }
    }
}

impl Default for Transaction {
    fn default() -> Self {
        let one = H256::from_slice(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
        ]);
        Self {
            version: 1_u8,
            hash: one,
            signature: BoundedVec::try_from(vec![one, one]).unwrap(),
            nonce: U256::default(),
            sender_address: ContractAddressWrapper::default(),
            call_entrypoint: CallEntryPointWrapper::default(),
            contract_class: None,
            contract_address_salt: None,
            max_fee: U256::from(u128::MAX),
        }
    }
}

impl Default for TransactionReceiptWrapper {
    fn default() -> Self {
        Self {
            transaction_hash: H256::default(),
            actual_fee: U256::default(),
            tx_type: TxType::Invoke,
            block_hash: U256::default(),
            block_number: 0_u64,
            events: BoundedVec::try_from(vec![EventWrapper::default(), EventWrapper::default()]).unwrap(),
        }
    }
}
