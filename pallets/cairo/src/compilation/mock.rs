use frame_support::BoundedVec;

use super::SierraCompiler;
use crate::types::CairoAssemblyProgram;
use crate::{Config, Error};

/// Mocks the `SierraCompiler` trait for testing purposes.
#[derive(Default)]
pub struct SierraCompilerMock;

impl<T: Config> SierraCompiler<T> for SierraCompilerMock {
    /// Compiles a Sierra program into a Cairo assembly program.
    fn compile(&self, sierra_program: &crate::types::SierraProgram<T>) -> Result<CairoAssemblyProgram<T>, Error<T>> {
        Ok(CairoAssemblyProgram {
            // TODO: Think if we should generate id during compilation or not.
            // For mock it is ok to use the same id as the sierra program.
            id: sierra_program.id.clone(),
            sierra_program_id: Some(sierra_program.id),
            code: BoundedVec::with_bounded_capacity(0),
        })
    }
}
