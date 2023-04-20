//! Starknet transaction related functionality.
/// Types related to transactions.
pub mod types;

use alloc::vec;

use blockifier::block_context::BlockContext;
use blockifier::execution::contract_class::ContractClass;
use blockifier::execution::entry_point::{CallInfo, ExecutionResources};
use blockifier::state::cached_state::CachedState;
use blockifier::state::state_api::StateReader;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::AccountTransactionContext;
use blockifier::transaction::transactions::Executable;
use frame_support::BoundedVec;
use sp_core::{H256, U256};
use starknet_api::api_core::{ContractAddress as StarknetContractAddress, EntryPointSelector, Nonce};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{
    ContractAddressSalt, DeclareTransaction, DeclareTransactionV0V1, DeployAccountTransaction, EventContent, Fee,
    InvokeTransactionV1, L1HandlerTransaction, TransactionHash, TransactionOutput, TransactionReceipt,
    TransactionSignature, TransactionVersion,
};
use starknet_api::StarknetApiError;

use self::types::{
    EventError, EventWrapper, MaxArraySize, Transaction, TransactionExecutionErrorWrapper,
    TransactionExecutionResultWrapper, TransactionReceiptWrapper, TxType,
};
use crate::block::serialize::SerializeBlockContext;
use crate::block::Block as StarknetBlock;
use crate::execution::{CallEntryPointWrapper, ContractAddressWrapper, ContractClassWrapper};

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

    // TODO: add block hash and block number (#252)
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
                TransactionOutput::Declare(_) => TxType::DeclareTx,
                TransactionOutput::DeployAccount(_) => TxType::DeployAccountTx,
                TransactionOutput::Invoke(_) => TxType::InvokeTx,
                TransactionOutput::L1Handler(_) => TxType::L1HandlerTx,
                _ => TxType::InvokeTx,
            },
            events: BoundedVec::try_from(_events?).map_err(|_| EventError::TooManyEvents)?,
        })
    }
}

/// Try to convert a `&Transaction` into a `DeployAccountTransaction`.
impl TryInto<DeployAccountTransaction> for &Transaction {
    type Error = StarknetApiError;

    fn try_into(self) -> Result<DeployAccountTransaction, Self::Error> {
        Ok(DeployAccountTransaction {
            transaction_hash: TransactionHash(StarkFelt::new(self.hash.0)?),
            max_fee: Fee(2),
            version: TransactionVersion(StarkFelt::new(U256::from(self.version).into())?),
            signature: TransactionSignature(
                self.signature.clone().into_inner().iter().map(|x| StarkFelt::new(x.0).unwrap()).collect(),
            ),
            nonce: Nonce(StarkFelt::new(self.nonce.into())?),
            contract_address: StarknetContractAddress::try_from(StarkFelt::new(self.sender_address)?)?,
            class_hash: self.call_entrypoint.to_starknet_call_entry_point().class_hash.unwrap_or_default(),
            constructor_calldata: self.call_entrypoint.to_starknet_call_entry_point().calldata,
            contract_address_salt: ContractAddressSalt(StarkFelt::new(
                self.contract_address_salt.unwrap_or_default().to_fixed_bytes(),
            )?),
        })
    }
}

/// Try to convert a `&Transaction` into a `L1HandlerTransaction`.
impl TryInto<L1HandlerTransaction> for &Transaction {
    type Error = StarknetApiError;

    fn try_into(self) -> Result<L1HandlerTransaction, Self::Error> {
        Ok(L1HandlerTransaction {
            transaction_hash: TransactionHash(StarkFelt::new(self.hash.0)?),
            version: TransactionVersion(StarkFelt::new(U256::from(self.version).into())?),
            nonce: Nonce(StarkFelt::new(self.nonce.into())?),
            contract_address: StarknetContractAddress::try_from(StarkFelt::new(self.sender_address)?)?,
            calldata: self.call_entrypoint.to_starknet_call_entry_point().calldata,
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
        Ok(InvokeTransactionV1 {
            transaction_hash: TransactionHash(StarkFelt::new(self.hash.0)?),
            max_fee: Fee(2),
            signature: TransactionSignature(
                self.signature.clone().into_inner().iter().map(|x| StarkFelt::new(x.0).unwrap()).collect(),
            ),
            nonce: Nonce(StarkFelt::new(self.nonce.into())?),
            sender_address: StarknetContractAddress::try_from(StarkFelt::new(self.sender_address)?)?,
            calldata: self.call_entrypoint.to_starknet_call_entry_point().calldata,
        })
    }
}

/// Try to convert a `&Transaction` into a `DeclareTransaction`.
impl TryInto<DeclareTransaction> for &Transaction {
    type Error = StarknetApiError;

    fn try_into(self) -> Result<DeclareTransaction, Self::Error> {
        let tx = DeclareTransactionV0V1 {
            transaction_hash: TransactionHash(StarkFelt::new(self.hash.0)?),
            max_fee: Fee(2),
            signature: TransactionSignature(
                self.signature.clone().into_inner().iter().map(|x| StarkFelt::new(x.0).unwrap()).collect(),
            ),
            nonce: Nonce(StarkFelt::new(self.nonce.into())?),
            sender_address: StarknetContractAddress::try_from(StarkFelt::new(self.sender_address)?)?,
            class_hash: self.call_entrypoint.to_starknet_call_entry_point().class_hash.unwrap_or_default(),
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
        contract_address_salt: Option<H256>,
    ) -> Self {
        Self { version, hash, signature, sender_address, nonce, call_entrypoint, contract_class, contract_address_salt }
    }

    /// Creates a new instance of a transaction without signature.
    pub fn from_tx_hash(hash: H256) -> Self {
        Self { hash, ..Self::default() }
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
            TxType::DeclareTx => {
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
    pub fn execute<S: StateReader>(
        &self,
        state: &mut CachedState<S>,
        block: StarknetBlock,
        tx_type: TxType,
        contract_class: Option<ContractClass>,
        fee_token_address: ContractAddressWrapper,
    ) -> TransactionExecutionResultWrapper<Option<CallInfo>> {
        // Create the block context.
        let block_context = BlockContext::try_serialize(block.header().clone(), fee_token_address)
            .map_err(|_| TransactionExecutionErrorWrapper::BlockContextSerializationError)?;
        // Initialize the execution resources.
        let execution_resources = &mut ExecutionResources::default();

        // Verify the transaction version.
        self.verify_tx_version(&tx_type)?;

        match tx_type {
            TxType::InvokeTx => {
                let tx: InvokeTransactionV1 = self.try_into().map_err(TransactionExecutionErrorWrapper::StarknetApi)?;
                let account_context = self.get_invoke_transaction_context(&tx);

                tx.run_execute(state, execution_resources, &block_context, &account_context, contract_class)
                    .map_err(TransactionExecutionErrorWrapper::TransactionExecution)
            }
            TxType::L1HandlerTx => {
                let tx = self.try_into().map_err(TransactionExecutionErrorWrapper::StarknetApi)?;
                let account_context = self.get_l1_handler_transaction_context(&tx);
                tx.run_execute(state, execution_resources, &block_context, &account_context, contract_class)
                    .map_err(TransactionExecutionErrorWrapper::TransactionExecution)
            }
            TxType::DeclareTx => {
                let tx = self.try_into().map_err(TransactionExecutionErrorWrapper::StarknetApi)?;
                let account_context = self.get_declare_transaction_context(&tx);
                // Execute.
                tx.run_execute(state, execution_resources, &block_context, &account_context, contract_class)
                    .map_err(TransactionExecutionErrorWrapper::TransactionExecution)
            }
            TxType::DeployAccountTx => {
                let tx = self.try_into().map_err(TransactionExecutionErrorWrapper::StarknetApi)?;
                let account_context = self.get_deploy_account_transaction_context(&tx);

                // Execute.
                tx.run_execute(state, execution_resources, &block_context, &account_context, contract_class)
                    .map_err(TransactionExecutionErrorWrapper::TransactionExecution)
            }
        }

        // TODO: Investigate the use of tx.execute() instead of tx.run_execute()
        // Going one lower level gives us more flexibility like not validating the tx as we could do
        // it before the tx lands in the mempool.
        // However it also means we need to copy/paste internal code from the tx.execute() method.
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
        }
    }
}

impl Default for TransactionReceiptWrapper {
    fn default() -> Self {
        Self {
            transaction_hash: H256::default(),
            actual_fee: U256::default(),
            tx_type: TxType::InvokeTx,
            events: BoundedVec::try_from(vec![EventWrapper::default(), EventWrapper::default()]).unwrap(),
        }
    }
}
