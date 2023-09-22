//! Definition of the runtime API for the StarkNet pallet.

// Adding allow unused type parameters to avoid clippy errors
// generated by the `decl_runtime_apis` macro.
// Specifically, the macro generates a trait (`StarknetRuntimeApi`) with unused type parameters.
#![allow(clippy::extra_unused_type_parameters)]

use blockifier::execution::contract_class::ContractClass;
use mp_felt::Felt252Wrapper;
use mp_transactions::{Transaction, TxType, UserTransaction};
use sp_api::BlockT;
pub extern crate alloc;
use alloc::vec::Vec;

use sp_runtime::DispatchError;
use starknet_api::api_core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;
use starknet_api::transaction::{Calldata, Event as StarknetEvent, TransactionHash};

#[derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)]
pub enum StarknetTransactionExecutionError {
    ContractNotFound,
    ClassAlreadyDeclared,
    ClassHashNotFound,
    InvalidContractClass,
    ContractError,
}

sp_api::decl_runtime_apis! {
    pub trait StarknetRuntimeApi {
        /// Returns the nonce associated with the given address in the given block
        fn nonce(contract_address: ContractAddress) -> Nonce;
        /// Returns a storage slot value
        fn get_storage_at(address: ContractAddress, key: StorageKey) -> Result<StarkFelt, DispatchError>;
        /// Returns a `Call` response.
        fn call(address: ContractAddress, function_selector: EntryPointSelector, calldata: Calldata) -> Result<Vec<Felt252Wrapper>, DispatchError>;
        /// Returns the contract class hash at the given address.
        fn contract_class_hash_by_address(address: ContractAddress) -> ClassHash;
        /// Returns the contract class for the given class hash.
        fn contract_class_by_class_hash(class_hash: ClassHash) -> Option<ContractClass>;
        /// Returns the chain id.
        fn chain_id() -> Felt252Wrapper;
        /// Returns fee estimate
        fn estimate_fee(transaction: UserTransaction) -> Result<(u64, u64), DispatchError>;
        /// Filters extrinsic transactions to return only Starknet transactions
        ///
        /// To support runtime upgrades, the client must be unaware of the specific extrinsic
        /// details. To achieve this, the client uses an OpaqueExtrinsic type to represent and
        /// manipulate extrinsics. However, the client cannot decode and filter extrinsics due to
        /// this limitation. The solution is to offload decoding and filtering to the RuntimeApi in
        /// the runtime itself, accomplished through the extrinsic_filter method. This enables the
        /// client to operate seamlessly while abstracting the extrinsic complexity.
        fn extrinsic_filter(xts: Vec<<Block as BlockT>::Extrinsic>) -> Vec<Transaction>;
        fn get_events_for_tx_hash(xts: Vec<<Block as BlockT>::Extrinsic>, chain_id: Felt252Wrapper, tx_hash: Felt252Wrapper) -> Option<(TxType, Vec<StarknetEvent>)>;

        /// Return the list of StarknetEvent evmitted during this block, along with the hash of the starknet transaction they bellong to
        ///
        /// `block_extrinsics` is the list of all the extrinsic executed during this block, it is used in order to match
        fn get_starknet_events_and_their_associated_tx_hash(block_extrinsics: Vec<<Block as BlockT>::Extrinsic>, chain_id: Felt252Wrapper) -> Vec<(Felt252Wrapper, StarknetEvent)>;
        /// Return the outcome of the tx execution
        fn get_tx_execution_outcome(tx_hash: TransactionHash) -> Option<Vec<u8>>;
    }

    pub trait ConvertTransactionRuntimeApi {
        /// Converts the transaction to an UncheckedExtrinsic for submission to the pool.
        fn convert_transaction(transaction: UserTransaction) -> Result<<Block as BlockT>::Extrinsic, DispatchError>;
        /// Converts the DispatchError to an understandable error for the client
        fn convert_error(error: DispatchError) -> StarknetTransactionExecutionError;
    }
}
