use sp_core::H256;

sp_api::decl_runtime_apis! {
    pub trait StarknetRuntimeApi {
        fn current_block_hash() -> H256;
        fn block() -> mp_starknet::starknet_block::block::Block;
    }
}
