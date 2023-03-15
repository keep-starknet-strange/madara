//! Transaction validation logic.
use frame_support::traits::EnsureOrigin;

use crate::types::Origin;

/// Ensure that the origin is a Starknet transaction.
/// See: `https://github.com/keep-starknet-strange/kaioshin/issues/21`
/// # Arguments
/// * `o` - The origin to check.
/// # Returns
/// * `Result<(), &'static str>` - The result of the check.
pub fn ensure_starknet_transaction<OuterOrigin>(o: OuterOrigin) -> Result<(), &'static str>
where
    OuterOrigin: Into<Result<Origin, OuterOrigin>>,
{
    match o.into() {
        Ok(Origin::StarknetTransaction) => Ok(()),
        _ => Err("bad origin: expected to be an Starknet transaction"),
    }
}

/// Ensure that the origin is a Starknet transaction.
/// See: `https://github.com/keep-starknet-strange/kaioshin/issues/21`
pub struct EnsureStarknetTransaction;
impl<OuterOrigin: Into<Result<Origin, OuterOrigin>> + From<Origin>> EnsureOrigin<OuterOrigin>
    for EnsureStarknetTransaction
{
    type Success = ();

    /// Try to convert the origin into a `Origin::StarknetTransaction`.
    /// # Arguments
    /// * `o` - The origin to check.
    /// # Returns
    /// * `Result<Self::Success, O>` - The result of the check.
    fn try_origin(o: OuterOrigin) -> Result<Self::Success, OuterOrigin> {
        o.into().map(|o| match o {
            Origin::StarknetTransaction => (),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<OuterOrigin, ()> {
        Ok(OuterOrigin::from(Origin::StarknetTransaction))
    }
}
