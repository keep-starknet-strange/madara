use sp_core::{H256, U256};

sp_api::decl_runtime_apis! {
    pub trait StarknetRuntimeApi {
        fn current_block_number() -> U256;
        fn current_block_hash() -> H256;
    }
}
