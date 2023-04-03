use sp_core::H256;

sp_api::decl_runtime_apis! {
    pub trait StarknetRuntimeApi {
        fn current_block_hash() -> H256;
        fn current_block() -> mp_starknet::starknet_block::block::Block;
    }
}
