#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

use sp_core::{H256, U256};

sp_api::decl_runtime_apis! {
	pub trait StarknetRuntimeApi {
		fn current_block_number() -> Option<U256>;
		fn current_block_hash() -> Option<H256>;
	}
}
