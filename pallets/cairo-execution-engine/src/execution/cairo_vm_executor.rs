use crate::{
	execution::CairoExecutor,
	types::{CairoAssemblyProgram, CairoAssemblyProgramInput, CairoAssemblyProgramOutput},
	Config, Error,
};

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
		// Read the Cairo program from hardcoded file.
		// TODO: Replace with the actual Cairo program from the `cairo_program` argument.
		//const PROGRAM_JSON: &str = include_str!("./array_sum.json");
		//let program = Program::from_reader(Cursor::new(PROGRAM_JSON), Some("main")).unwrap();

		// Instantiate the Virtual Machine.
		//let mut vm = VirtualMachine::new(false);
		// Instantiate the Cairo runner.
		/*let mut cairo_runner = CairoRunner::new(&program, "all", false).unwrap();
		let mut hint_processor = BuiltinHintProcessor::new_empty();
		let func_name = "main";
		let entrypoint = program
			.identifiers
			.get(&format!("__main__.{}", &func_name))
			.unwrap()
			.pc
			.unwrap();
		cairo_runner.initialize_builtins(&mut vm).unwrap();
		cairo_runner.initialize_segments(&mut vm, None);
		let args = vec![];
		// Execute the Cairo program.
		cairo_runner
			.run_from_entrypoint(entrypoint, &args, false, &mut vm, &mut hint_processor)
			.unwrap();
		let mut buffer = Cursor::new(Vec::new());
		// Read the output and write it to the buffer.
		cairo_runner.write_output(&mut vm, &mut buffer).unwrap();
		// Print the output.
		//log::info!("{}", String::from_utf8(buffer.into_inner()).unwrap().as_str());*/
		Ok(CairoAssemblyProgramOutput::empty())
	}
}
