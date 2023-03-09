use crate::execution::CairoExecutor;
use crate::types::{CairoAssemblyProgram, CairoAssemblyProgramInput, CairoAssemblyProgramOutput};
use crate::{Config, Error};

/// Mocks the `CairoExecutor` trait for testing purposes.
#[derive(Default)]
pub struct CairoExecutorMock;

/// Implementation of the `CairoExecutor` trait for the `CairoExecutorMock`.
/// This implementation is used for testing purposes.
impl<T: Config> CairoExecutor<T> for CairoExecutorMock {
    /// Executes a Cairo program.
    /// # Arguments
    /// * `cairo_program` - The Cairo program to execute.
    /// * `input` - The input to pass to the Cairo program.
    /// # Returns
    /// The result of executing the Cairo program.
    fn execute(
        &self,
        _cairo_program: &CairoAssemblyProgram<T>,
        _input: &CairoAssemblyProgramInput,
    ) -> Result<CairoAssemblyProgramOutput, Error<T>> {
        Ok(CairoAssemblyProgramOutput::empty())
    }
}
