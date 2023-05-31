//! Definition of the runtime API for the StarkNet pallet.

// Adding allow unused type parameters to avoid clippy errors
// generated by the `decl_runtime_apis` macro.
// Specifically, the macro generates a trait (`StarknetRuntimeApi`) with unused type parameters.
#![allow(clippy::extra_unused_type_parameters)]

use mp_starknet::crypto::hash::Hasher;
use mp_starknet::execution::types::{
    ClassHashWrapper, ContractAddressWrapper, ContractClassWrapper, Felt252Wrapper, StorageKeyWrapper,
};
use mp_starknet::transaction::types::{Transaction, TxType};
use sp_api::BlockT;
pub extern crate alloc;
use alloc::vec::Vec;

use sp_runtime::DispatchError;

use crate::types::NonceWrapper;

sp_api::decl_runtime_apis! {
    pub trait StarknetRuntimeApi {
        /// Returns the current block hash.
        fn current_block_hash() -> Felt252Wrapper;
        /// Returns the current block.
        fn current_block() -> mp_starknet::block::Block;
        /// Returns the nonce associated with the given address in the given block
        fn nonce(contract_address: ContractAddressWrapper) -> NonceWrapper;
        /// Returns a storage slot value
        fn get_storage_at(address: ContractAddressWrapper, key: StorageKeyWrapper) -> Result<Felt252Wrapper, DispatchError>;
        /// Returns a `Call` response.
        fn call(address: ContractAddressWrapper, function_selector: Felt252Wrapper, calldata: Vec<Felt252Wrapper>) -> Result<Vec<Felt252Wrapper>, DispatchError>;
        /// Returns the contract class hash at the given address.
        fn contract_class_hash_by_address(address: ContractAddressWrapper) -> Option<ClassHashWrapper>;
        /// Returns the contract class for the given class hash.
        fn contract_class_by_class_hash(class_hash: ClassHashWrapper) -> Option<ContractClassWrapper>;
        /// Returns the chain id.
        fn chain_id() -> u128;
        /// Returns fee estimate
        fn estimate_fee(transaction: Transaction) -> Result<(u64, u64), DispatchError>;
        /// Returns the hasher used by the runtime.
        fn get_hasher() -> Hasher;
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
