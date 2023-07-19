//! Definition of the runtime API for the StarkNet pallet.

// Adding allow unused type parameters to avoid clippy errors
// generated by the `decl_runtime_apis` macro.
// Specifically, the macro generates a trait (`StarknetRuntimeApi`) with unused type parameters.
#![allow(clippy::extra_unused_type_parameters)]

use blockifier::execution::contract_class::ContractClass;
use mp_starknet::crypto::hash::Hasher;
use mp_starknet::execution::types::{ClassHashWrapper, ContractAddressWrapper, Felt252Wrapper, StorageKeyWrapper};
use mp_starknet::transaction::types::{EventWrapper, Transaction, TransactionExecutionInfoWrapper, TxType};
use sp_api::BlockT;
pub extern crate alloc;
use alloc::vec::Vec;

use sp_runtime::DispatchError;

use crate::types::{NonceWrapper, StateCommitments};
use crate::StateTrie;

sp_api::decl_runtime_apis! {
    pub trait StarknetRuntimeApi {
        /// Returns the nonce associated with the given address in the given block
        fn nonce(contract_address: ContractAddressWrapper) -> NonceWrapper;
        /// Returns the events associated with the given block
        fn events() -> Vec<EventWrapper>;
        /// Returns a storage slot value
        fn get_storage_at(address: ContractAddressWrapper, key: StorageKeyWrapper) -> Result<Felt252Wrapper, DispatchError>;
        /// Returns a `Call` response.
        fn call(address: ContractAddressWrapper, function_selector: Felt252Wrapper, calldata: Vec<Felt252Wrapper>) -> Result<Vec<Felt252Wrapper>, DispatchError>;
        /// Returns the contract class hash at the given address.
        fn contract_class_hash_by_address(address: ContractAddressWrapper) -> Option<ClassHashWrapper>;
        /// Returns the contract class for the given class hash.
        fn contract_class_by_class_hash(class_hash: ClassHashWrapper) -> Option<ContractClass>;
        /// Returns the contract root for the given address
        fn contract_state_root_by_address(address: ContractAddressWrapper) -> Option<Felt252Wrapper>;
        /// Returns the contract state trie for the given address
        fn contract_state_trie_by_address(address: ContractAddressWrapper) -> Option<StateTrie>;
        /// Returns the chain id.
        fn chain_id() -> Felt252Wrapper;
        /// Returns fee estimate
        fn estimate_fee(transaction: Transaction) -> Result<(u64, u64), DispatchError>;
        /// Execute transactions without applying changes to the state (NOTE: initial state is at the end of the specified substrate block)
        fn simulate_transactions(transactions: Vec<Transaction>, skip_validate: bool, skip_fee_charge: bool) -> Result<Vec<TransactionExecutionInfoWrapper>, DispatchError>;
        /// Returns the hasher used by the runtime.
        fn get_hasher() -> Hasher;
        /// Returns state commitments
        fn get_state_commitments() -> StateCommitments;
        /// Filters extrinsic transactions to return only Starknet transactions
        ///
        /// To support runtime upgrades, the client must be unaware of the specific extrinsic
        /// details. To achieve this, the client uses an OpaqueExtrinsic type to represent and
        /// manipulate extrinsics. However, the client cannot decode and filter extrinsics due to
        /// this limitation. The solution is to offload decoding and filtering to the RuntimeApi in
        /// the runtime itself, accomplished through the extrinsic_filter method. This enables the
        /// client to operate seamlessly while abstracting the extrinsic complexity.
        fn extrinsic_filter(xts: Vec<<Block as BlockT>::Extrinsic>) -> Vec<Transaction>;
    }

    pub trait ConvertTransactionRuntimeApi {
        /// Converts the transaction to an UncheckedExtrinsic for submission to the pool.
        fn convert_transaction(transaction: Transaction, tx_type: TxType) -> Result<<Block as BlockT>::Extrinsic, DispatchError>;
    }
}
