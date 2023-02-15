//! Cairo Execution Engine pallet benchmarking.
#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as Starknet;
use frame_benchmarking::{
	account, benchmarks, whitelist_account, whitelisted_caller, BenchmarkError, Vec,
};
use frame_support::{
	assert_noop, assert_ok,
	sp_runtime::{traits::Bounded, BoundedVec},
	sp_std,
	traits::{Currency, EnsureOrigin, Get, OnInitialize, UnfilteredDispatchable},
};
use frame_system::RawOrigin;

benchmarks! {
	ping {
		let s in 0 .. 100;
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller))
	verify {
		// TODO: Add post conditions checks.
		assert_eq!(true, true);
	}

	impl_benchmark_test_suite!(Starknet, crate::mock::new_test_ext(), crate::mock::Test);
}
