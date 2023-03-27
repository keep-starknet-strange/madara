#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

// use mp_starknet::starknet_block::block::Block;
use sp_core::H256;

sp_api::decl_runtime_apis! {
	pub trait StarknetRuntimeApi {
		fn current_block_hash() -> Option<H256>;
	}
}
