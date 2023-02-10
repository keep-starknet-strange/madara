//! Module for executing Starknet contracts.
pub mod mock;

use crate::{types::StarknetContract, Config, Error};

/// Trait for executing Starknet contracts.
pub trait StarknetExecutor<T: Config> {
	/// Executes a Starknet contract.
	/// # Arguments
	/// * `starknet_contract` - The Starknet contraxct to execute.
	/// # Returns
	/// The result of executing the Starknet contract.
	fn execute(&self, starknet_contract: &StarknetContract) -> Result<(), Error<T>>;
}
