//! Starknet transaction related functionality.
/// Types related to transactions.
pub mod types;

use alloc::vec;

use blockifier::block_context::BlockContext;
use blockifier::execution::entry_point::CallInfo;
use blockifier::state::cached_state::CachedState;
use blockifier::state::state_api::StateReader;
use blockifier::transaction::errors::InvokeTransactionError;
use blockifier::transaction::objects::{AccountTransactionContext, TransactionExecutionResult};
use blockifier::transaction::transactions::Executable;
use frame_support::BoundedVec;
use sp_core::{H256, U256};
use starknet_api::api_core::{ContractAddress as StarknetContractAddress, EntryPointSelector, Nonce};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{
    Fee, InvokeTransaction, L1HandlerTransaction, TransactionHash, TransactionSignature, TransactionVersion, DeclareTransaction,
};

use self::types::{Event, MaxArraySize, Transaction, TxType};
use crate::execution::{CallEntryPointWrapper, ContractAddressWrapper, ContractClassWrapper};
use crate::starknet_block::block::Block;
use crate::starknet_block::serialize::SerializeBlockContext;

impl Event {
    /// Creates a new instance of an event.
    pub fn new(keys: BoundedVec<H256, MaxArraySize>, data: BoundedVec<H256, MaxArraySize>, from_address: H256) -> Self {
        Self { keys, data, from_address }
    }

    /// Creates an empty event.
    pub fn empty() -> Self {
        Self {
            keys: BoundedVec::try_from(vec![]).unwrap(),
            data: BoundedVec::try_from(vec![]).unwrap(),
            from_address: H256::zero(),
        }
    }
}
impl Default for Event {
    fn default() -> Self {
        let one = H256::from_slice(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
        ]);
        Self {
            keys: BoundedVec::try_from(vec![one, one]).unwrap(),
            data: BoundedVec::try_from(vec![one, one]).unwrap(),
            from_address: one,
        }
    }
}

impl Transaction {
    /// Creates a new instance of a transaction.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: U256,
        hash: H256,
        signature: BoundedVec<H256, MaxArraySize>,
        events: BoundedVec<Event, MaxArraySize>,
        sender_address: ContractAddressWrapper,
        nonce: U256,
        call_entrypoint: CallEntryPointWrapper,
		contract_class: Option<ContractClassWrapper>,
    ) -> Self {
        Self { version, hash, signature, events, sender_address, nonce, call_entrypoint, contract_class }
    }

    /// Creates a new instance of a transaction without signature.
    pub fn from_tx_hash(hash: H256) -> Self {
        Self { hash, ..Self::default() }
    }

	/// Converts a transaction to a blockifier declare transaction
    ///
    /// # Arguments
    ///
    /// * `self` - The transaction to convert
    ///
    /// # Returns
    ///
    /// * `AccountTransaction` - The converted transaction
    pub fn to_invoke_tx(&self) -> InvokeTransaction {
        InvokeTransaction {
            transaction_hash: TransactionHash(StarkFelt::new(self.hash.0).unwrap()),
            max_fee: Fee(2),
            version: TransactionVersion(StarkFelt::new(self.version.into()).unwrap()),
            signature: TransactionSignature(
                self.signature.clone().into_inner().iter().map(|x| StarkFelt::new(x.0).unwrap()).collect(),
            ),
            nonce: Nonce(StarkFelt::new(self.nonce.into()).unwrap()),
            sender_address: StarknetContractAddress::try_from(StarkFelt::new(self.sender_address).unwrap()).unwrap(),
            calldata: self.call_entrypoint.to_starknet_call_entry_point().calldata,
            entry_point_selector: None,
        }
    }

    /// Converts a transaction to a blockifier invoke transaction
    ///
    /// # Arguments
    ///
    /// * `self` - The transaction to convert
    ///
    /// # Returns
    ///
    /// * `AccountTransaction` - The converted transaction
    pub fn to_declare_tx(&self) -> DeclareTransaction {
        DeclareTransaction {
            transaction_hash: TransactionHash(StarkFelt::new(self.hash.0).unwrap()),
            max_fee: Fee(2),
            version: TransactionVersion(StarkFelt::new(self.version.into()).unwrap()),
            signature: TransactionSignature(
                self.signature.clone().into_inner().iter().map(|x| StarkFelt::new(x.0).unwrap()).collect(),
            ),
            nonce: Nonce(StarkFelt::new(self.nonce.into()).unwrap()),
            sender_address: StarknetContractAddress::try_from(StarkFelt::new(self.sender_address).unwrap()).unwrap(),
            class_hash: self.call_entrypoint.to_starknet_call_entry_point().class_hash.unwrap(),
        }
    }

    /// Converts a transaction to a blockifier l1 handler transaction
    ///
    /// # Arguments
    ///
    /// * `self` - The transaction to convert
    ///
    /// # Returns
    ///
    /// * `L1HandlerTransaction` - The converted transaction
    pub fn to_l1_handler_tx(&self) -> L1HandlerTransaction {
        L1HandlerTransaction {
            transaction_hash: TransactionHash(StarkFelt::new(self.hash.0).unwrap()),
            version: TransactionVersion(StarkFelt::new(self.version.into()).unwrap()),
            nonce: Nonce(StarkFelt::new(self.nonce.into()).unwrap()),
            contract_address: StarknetContractAddress::try_from(StarkFelt::new(self.sender_address).unwrap()).unwrap(),
            calldata: self.call_entrypoint.to_starknet_call_entry_point().calldata,
            entry_point_selector: EntryPointSelector(
                StarkHash::new(*self.call_entrypoint.entrypoint_selector.unwrap().as_fixed_bytes()).unwrap(),
            ),
        }
    }

    /// Executes a transaction
    ///
    /// # Arguments
    ///
    /// * `self` - The transaction to execute
    /// * `state` - The state to execute the transaction on
    /// * `block` - The block to execute the transaction on
    ///
    /// # Returns
    ///
    /// * `TransactionExecutionResult<TransactionExecutionInfo>` - The result of the transaction
    ///   execution
    pub fn execute<S: StateReader>(
        &self,
        state: &mut CachedState<S>,
        block: Block,
        tx_type: TxType,
		contract_class: ContractClassWrapper,
    ) -> TransactionExecutionResult<Option<CallInfo>> {
        let block_context = BlockContext::serialize(block.header);
        match tx_type {
            TxType::InvokeTx =>  {
				let tx = self.to_invoke_tx();
				let account_context = self.get_invoke_transaction_context(&tx);
				// Specifying an entry point selector is not allowed; `__execute__` is called, and
				// the inner selector appears in the calldata.
				if tx.entry_point_selector.is_some() {
					return Err(InvokeTransactionError::SpecifiedEntryPoint)?;
				}

				tx.run_execute(state, &block_context, &account_context, None)
            },
            TxType::L1HandlerTx => {
                let tx = self.to_l1_handler_tx();
                tx.run_execute(state, &block_context, &self.get_l1_handler_transaction_context(&tx), None)
            },
			TxType::DeclareTx => {
				let tx = self.to_declare_tx();
				let account_context = self.get_declare_transaction_context(&tx);
				let contract_class = contract_class.to_starknet_contract_class();

				// Execute.
				tx.run_execute(
					state,
					&block_context,
					&account_context,
					Some(contract_class.into()),
				)
			}
        }

        // TODO: Investigate the use of tx.execute() instead of tx.run_execute()
        // Going one lower level gives us more flexibility like not validating the tx as we could do
        // it before the tx lands in the mempool.
        // However it also means we need to copy/paste internal code from the tx.execute() method.
    }

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
    fn get_invoke_transaction_context(&self, tx: &InvokeTransaction) -> AccountTransactionContext {
        AccountTransactionContext {
            transaction_hash: tx.transaction_hash,
            max_fee: tx.max_fee,
            version: tx.version,
            signature: tx.signature.clone(),
            nonce: tx.nonce,
            sender_address: tx.sender_address,
        }
    }

	fn get_declare_transaction_context(&self, tx: &DeclareTransaction) -> AccountTransactionContext {
		AccountTransactionContext {
			transaction_hash: tx.transaction_hash,
			max_fee: tx.max_fee,
			version: tx.version,
			signature: tx.signature.clone(),
			nonce: tx.nonce,
			sender_address: tx.sender_address,
		}
	}

}

impl Default for Transaction {
    fn default() -> Self {
        let one = H256::from_slice(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
        ]);
        Self {
            version: U256::default(),
            hash: one,
            signature: BoundedVec::try_from(vec![one, one]).unwrap(),
            events: BoundedVec::try_from(vec![Event::default(), Event::default()]).unwrap(),
            nonce: U256::default(),
            sender_address: ContractAddressWrapper::default(),
            call_entrypoint: CallEntryPointWrapper::default(),
			contract_class: None,
        }
    }
}
