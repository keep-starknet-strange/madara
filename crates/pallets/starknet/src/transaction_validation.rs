//! Transaction validation logic.
use frame_support::traits::EnsureOrigin;

/// Representation of the origin of a Starknet transaction.
/// For now, we still don't know how to represent the origin of a Starknet transaction,
/// given that Starknet has native account abstraction.
/// For now, we just use a dummy origin.
/// See: `https://github.com/keep-starknet-strange/madara/issues/21`
pub enum RawOrigin {
    StarknetTransaction,
}

/// Ensure that the origin is a Starknet transaction.
/// See: `https://github.com/keep-starknet-strange/madara/issues/21`
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
/// See: `https://github.com/keep-starknet-strange/madara/issues/21`
pub struct EnsureStarknetTransaction;
impl<OuterOrigin: Into<Result<RawOrigin, OuterOrigin>> + From<RawOrigin>> EnsureOrigin<OuterOrigin>
    for EnsureStarknetTransaction
{
    type Success = ();

    /// Try to convert the origin into a `RawOrigin::StarknetTransaction`.
    /// # Arguments
    /// * `o` - The origin to check.
    /// # Returns
    /// * `Result<Self::Success, O>` - The result of the check.
    fn try_origin(o: OuterOrigin) -> Result<Self::Success, OuterOrigin> {
        o.into().map(|o| match o {
            RawOrigin::StarknetTransaction => (),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<OuterOrigin, ()> {
        Ok(OuterOrigin::from(RawOrigin::StarknetTransaction))
    }
}
