//! Cairo Execution Engine pallet benchmarking.
#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as CairoExecutionEngine;
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

fn dummy_sierra_code<T: Config>() -> BoundedVec<u8, T::MaxSierraProgramLength> {
	BoundedVec::truncate_from(sp_std::vec![1, 2, 3])
}

benchmarks! {
	deploy_sierra_program {
		let s in 0 .. 100;
		let caller: T::AccountId = whitelisted_caller();
		let sierra_code = dummy_sierra_code::<T>();
	}: _(RawOrigin::Signed(caller), sierra_code)
	verify {
		// TODO: Add post conditions checks.
		assert_eq!(true, true);
	}

	impl_benchmark_test_suite!(CairoExecutionEngine, crate::mock::new_test_ext(), crate::mock::Test);
}
