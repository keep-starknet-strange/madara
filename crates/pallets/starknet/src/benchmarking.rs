//! Starknet pallet benchmarking.
#![cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::{account, benchmarks, whitelist_account, whitelisted_caller, BenchmarkError, Vec};
use frame_support::sp_runtime::traits::Bounded;
use frame_support::sp_runtime::BoundedVec;
use frame_support::traits::{Currency, EnsureOrigin, Get, OnInitialize, UnfilteredDispatchable};
use frame_support::{assert_noop, assert_ok, sp_std};
use frame_system::RawOrigin;

use super::*;
use crate::Pallet as Starknet;

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
