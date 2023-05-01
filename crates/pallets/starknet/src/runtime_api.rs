use mp_starknet::execution::types::{ClassHashWrapper, ContractAddressWrapper, ContractClassWrapper};
use sp_core::{H256, U256};
pub extern crate alloc;
use alloc::vec::Vec;

use sp_runtime::DispatchError;

use crate::types::StarkFeltWrapper;

sp_api::decl_runtime_apis! {
    pub trait StarknetRuntimeApi {
        /// Returns the current block hash.
        fn current_block_hash() -> H256;
        /// Returns the current block.
        fn current_block() -> mp_starknet::block::Block;
        /// Returns a `Call` response.
        fn call(address: ContractAddressWrapper, function_selector: H256, calldata: Vec<U256>) -> Result<Vec<StarkFeltWrapper>, DispatchError>;
        /// Returns the contract class hash at the given address.
        fn contract_class_hash_by_address(address: ContractAddressWrapper) -> Option<ClassHashWrapper>;
        /// Returns the contract class for the given class hash.
        fn contract_class_by_class_hash(class_hash: ClassHashWrapper) -> Option<ContractClassWrapper>;
    }
}
