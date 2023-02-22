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
	/// The Cairo run configuration.
	pub static ref CAIRO_RUN_CONFIG: CairoRunConfig<'static> = CairoRunConfig::default();
	/// The Fibonacci Cairo program, hardcoded in JSON format.
	static ref FIBONACCI_PROGRAM: Program = Program::from_bytes(include_bytes!("./samples/fib.json"), Some("main")).unwrap();
	static ref ADD_PROGRAM: Program = Program::from_bytes(include_bytes!("./samples/add.json"), Some("main")).unwrap();
}

const ENTRY_POINT_MAIN: Option<&str> = Some("main");

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
	/// * Replace the hardcoded input with the actual input.
	/// * Proper error handling (remove `unwrap()` calls).
	fn execute(
		&self,
		cairo_program: &CairoAssemblyProgram<T>,
		_input: &CairoAssemblyProgramInput,
	) -> Result<CairoAssemblyProgramOutput, Error<T>> {
		// Instantiate the Cairo program.
		let program = Program::from_bytes(&cairo_program.code, ENTRY_POINT_MAIN).unwrap();
		// Run the Cairo program.
		CairoVmExecutor::run_program(&program).unwrap();
		// TODO: retrieve the output from the execution of the Cairo program.
		Ok(CairoAssemblyProgramOutput::empty())
	}
}

impl CairoVmExecutor {
	/// Runs the program with the given id.
	///
	/// This function is used for testing purposes. It is not used in the production version of the
	/// app.
	///
	/// # Arguments
	///
	/// * `id` - The id of the program to run.
	///
	/// # Notes
	///
	/// This function is only used for testing purposes. It is not used in production.
	pub fn run_hardcoded_program(id: u8) {
		match id {
			0 => {
				log::info!("Running fibonacci program");
				CairoVmExecutor::run_program(&FIBONACCI_PROGRAM).unwrap();
			},
			1 => {
				log::info!("Running add program");
				CairoVmExecutor::run_program(&ADD_PROGRAM).unwrap();
			},
			_ => log::info!("Invalid program id"),
		};
	}

	/// Runs a Cairo program in the Cairo VM
	///
	/// # Arguments
	/// * `program` - the Cairo program to run
	///
	/// # Returns
	/// * `Result<(), ()>` - the result of running the Cairo program
	///
	/// # Errors
	/// * `()` - if the Cairo program fails to run
	fn run_program(program: &Program) -> Result<(), ()> {
		log::info!("starting execution of Cairo program in Cairo VM");

		let mut hint_executor = BuiltinHintProcessor::new_empty();
		let mut cairo_runner =
			CairoRunner::new(program, CAIRO_RUN_CONFIG.layout, CAIRO_RUN_CONFIG.proof_mode)
				.unwrap();
		let mut vm = VirtualMachine::new(CAIRO_RUN_CONFIG.trace_enabled);

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

		Ok(())
	}
}
