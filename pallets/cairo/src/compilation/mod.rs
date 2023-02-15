//! Module for compiling Sierra programs into Cairo assembly programs.

/// Mock implementation of the Sierra compiler.
pub mod mock;

use crate::{
	types::{CairoAssemblyProgram, SierraProgram},
	Config, Error,
};

pub trait SierraCompiler<T: Config> {
	/// Compiles a Sierra program into a Cairo assembly program.
	/// # Arguments
	/// * `sierra_program` - The Sierra program to compile.
	/// # Returns
	/// * `Ok(CairoAssemblyProgram)` - The compiled Cairo assembly program.
	/// * `Err(Error)` - The error that occurred during compilation.
	fn compile(
		&self,
		sierra_program: &SierraProgram<T>,
	) -> Result<CairoAssemblyProgram<T>, Error<T>>;
}
