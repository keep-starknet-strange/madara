//! Transaction validation logic.
use frame_support::traits::EnsureOrigin;

use crate::types::RawOrigin;

/// Ensure that the origin is a Starknet transaction.
/// See: `https://github.com/keep-starknet-strange/kaioshin/issues/21`
/// # Arguments
/// * `o` - The origin to check.
/// # Returns
/// * `Result<(), &'static str>` - The result of the check.
pub fn ensure_starknet_transaction<OuterOrigin>(o: OuterOrigin) -> Result<(), &'static str>
where
	OuterOrigin: Into<Result<RawOrigin, OuterOrigin>>,
{
	match o.into() {
		Ok(RawOrigin::StarknetTransaction) => Ok(()),
		_ => Err("bad origin: expected to be an Starknet transaction"),
	}
}

/// Ensure that the origin is a Starknet transaction.
/// See: `https://github.com/keep-starknet-strange/kaioshin/issues/21`
pub struct EnsureStarknetTransaction;
impl<O: Into<Result<RawOrigin, O>> + From<RawOrigin>> EnsureOrigin<O>
	for EnsureStarknetTransaction
{
	type Success = ();

	/// Try to convert the origin into a `RawOrigin::StarknetTransaction`.
	/// # Arguments
	/// * `o` - The origin to check.
	/// # Returns
	/// * `Result<Self::Success, O>` - The result of the check.
	fn try_origin(o: O) -> Result<Self::Success, O> {
		o.into().map(|o| match o {
			RawOrigin::StarknetTransaction => (),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> O {
		O::from(RawOrigin::StarknetTransaction(Default::default()))
	}
}
