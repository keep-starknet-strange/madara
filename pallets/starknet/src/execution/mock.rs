use crate::{execution::StarknetExecutor, types::StarknetContract, Config, Error};

/// Mocks the `CairoExecutor` trait for testing purposes.
#[derive(Default)]
pub struct StarknetExecutorMock;

/// Implementation of the `CairoExecutor` trait for the `CairoExecutorMock`.
/// This implementation is used for testing purposes.
impl<T: Config> StarknetExecutor<T> for StarknetExecutorMock {
	/// Executes a Starknet contract.
	/// # Arguments
	/// * `starknet_contract` - The Starknet contraxct to execute.
	/// # Returns
	/// The result of executing the Starknet contract.
	fn execute(&self, _starknet_contract: &StarknetContract) -> Result<(), Error<T>> {
		Ok(())
	}
}
