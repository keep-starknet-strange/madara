use cairo_vm::{
	cairo_run::CairoRunConfig,
	hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor,
	types::program::Program,
	vm::{
		errors::vm_exception::VmException, runners::cairo_runner::CairoRunner,
		vm_core::VirtualMachine,
	},
};

use crate::{
	execution::CairoExecutor,
	types::{CairoAssemblyProgram, CairoAssemblyProgramInput, CairoAssemblyProgramOutput},
	Config, Error,
};

lazy_static! {
	/// The Cairo VM executor.
	pub static ref CAIRO_VM_EXECUTOR: CairoVmExecutor = CairoVmExecutor::default();
	/// The Fibonacci Cairo program, hardcoded in JSON format.
	static ref FIBONACCI_PROGRAM: Program = Program::from_bytes(include_bytes!("./samples/fib.json"), Some("main")).unwrap();
}

/// Cairo VM executor.
#[derive(Default)]
pub struct CairoVmExecutor;

/// Implementation of the `CairoExecutor` trait for the `CairoExecutorMock`.
/// This implementation is used for testing purposes.
impl<T: Config> CairoExecutor<T> for CairoVmExecutor {
	/// Executes a Cairo program.
	/// # Arguments
	/// * `cairo_program` - The Cairo program to execute.
	/// * `input` - The input to pass to the Cairo program.
	/// # Returns
	/// The result of executing the Cairo program.
	/// # TODO
	/// * Replace the hardcoded Cairo program with the actual Cairo program.
	/// * Replace the hardcoded input with the actual input.
	/// * Proper error handling (remove `unwrap()` calls).
	fn execute(
		&self,
		_cairo_program: &CairoAssemblyProgram<T>,
		_input: &CairoAssemblyProgramInput,
	) -> Result<CairoAssemblyProgramOutput, Error<T>> {
		log::info!("executing  Cairo program in Cairo VM");

		let cairo_run_config = CairoRunConfig::default();
		let mut hint_executor = BuiltinHintProcessor::new_empty();
		let mut cairo_runner = CairoRunner::new(
			&FIBONACCI_PROGRAM,
			cairo_run_config.layout,
			cairo_run_config.proof_mode,
		)
		.unwrap();
		let mut vm = VirtualMachine::new(cairo_run_config.trace_enabled);
		let end = cairo_runner.initialize(&mut vm).unwrap();
		cairo_runner
			.run_until_pc(end, &mut vm, &mut hint_executor)
			.map_err(|err| VmException::from_vm_error(&cairo_runner, &vm, err))
			.unwrap();
		cairo_runner.end_run(false, false, &mut vm, &mut hint_executor).unwrap();

		vm.verify_auto_deductions().unwrap();
		cairo_runner.read_return_values(&mut vm).unwrap();
		cairo_runner.relocate(&mut vm).unwrap();
		log::info!("finished execution of Cairo program in Cairo VM");
		Ok(CairoAssemblyProgramOutput::empty())
	}
}
