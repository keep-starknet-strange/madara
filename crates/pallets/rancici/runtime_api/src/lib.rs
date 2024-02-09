#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::extra_unused_type_parameters)]

use frame_support::dispatch::DispatchResult;
use starknet_api::api_core::ContractAddress;

sp_api::decl_runtime_apis! {
    pub trait RanciciRuntimeApi {
        /// join the player pool
        fn join_pool(player: ContractAddress, xp: u64) -> DispatchResult;

        /// get player pool
        fn player_pool() -> Vec<(ContractAddress, u64)>;
    }
}
