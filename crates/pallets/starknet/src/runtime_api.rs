use sp_core::H256;

use crate::types::StarkFeltWrapper;

sp_api::decl_runtime_apis! {
    pub trait StarknetRuntimeApi {
		/// Returns the current block hash.
        fn current_block_hash() -> H256;
		/// Returns the current block.
        fn current_block() -> mp_starknet::block::Block;
		/// Returns a `Call` response.
		fn call() -> StarkFeltWrapper;
    }
}
