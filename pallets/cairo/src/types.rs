//! Cairo Execution Engine pallet custom types.
use crate::Config;
use frame_support::pallet_prelude::*;

/// Identifier of a Cairo assembly program.
pub type CairoAssemblyProgamId = [u8; 32];
/// Identifier of a Sierra program.
pub type SierraProgramId = [u8; 32];

// TODO: Find a way to make those constants (`MaxCairoAssemblyProgramInputLength` and
// `MaxCairoAssemblyProgramInputNumber`) configurable at the pallet level. For now we fix them with
// a constant value because there are some issues when using `CairoAssemblyProgramInput` as a type
// parameter in a dispatchable call.

/// The maximum length of a single input that can be passed to a Cairo assembly program.
type MaxCairoAssemblyProgramInputLength = ConstU32<1073741824>;
/// The maximum number of inputs that can be passed to a Cairo assembly program.
type MaxCairoAssemblyProgramInputNumber = ConstU32<1073741824>;

/// Sierra program representation.
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct SierraProgram<T: Config> {
	/// The identifier of the Sierra program.
	pub id: SierraProgramId,
	/// The code of the Sierra program.
	pub code: BoundedVec<u8, T::MaxSierraProgramLength>,
	/// The account that deployed the Sierra program.
	pub deployer_account: T::AccountId,
	/// Id of the compiled Cairo assembly program if it has been compiled.
	/// If the Sierra program has not been compiled, this field is set to `None`.
	/// If the Sierra program has been compiled, this field is set to
	/// `Some(cairo_assembly_program_id)`.
	pub cairo_assembly_program_id: Option<CairoAssemblyProgamId>,
}

/// Cairo assembly program representation.
/// A Cairo assembly program is a program that is compiled from a Sierra program.
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct CairoAssemblyProgram<T: Config> {
	/// The identifier of the Cairo assembly program.
	/// We allow this field to be `None` because we might want to separate the compilation step
	/// from the generation of the Cairo assembly program identifier.
	/// If at some point that it does not make sense, we can remove the `Option` and make this
	/// field mandatory.
	pub id: CairoAssemblyProgamId,
	/// The identifier of the Sierra program that was compiled to the Cairo assembly program.
	/// None if the Cairo assembly program was not compiled from a Sierra program.
	pub sierra_program_id: Option<SierraProgramId>,
	/// The code of the Cairo assembly program.
	pub code: BoundedVec<u8, T::MaxCairoAssemblyProgramLength>,
}

/// Cairo assembly program input.
/// This is the input that is passed to the Cairo VM when executing a Cairo assembly program.
/// The input is a vector of vector of bytes.
/// Each individual vector of bytes is a single input and can have a maximum length of
/// `MaxCairoAssemblyProgramInputLength`. The maximum number of inputs is
/// `MaxCairoAssemblyProgramInputNumber`.
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct CairoAssemblyProgramInput(
	BoundedVec<
		BoundedVec<u8, MaxCairoAssemblyProgramInputLength>,
		MaxCairoAssemblyProgramInputNumber,
	>,
);

/// Cairo assembly program output.
/// This is the output that is returned by the Cairo VM when executing a Cairo assembly program.
/// The output is a vector of vector of bytes.
/// Each individual vector of bytes is a single output and can have a maximum length of
/// `MaxCairoAssemblyProgramInputLength`. The maximum number of outputs is
/// `MaxCairoAssemblyProgramInputNumber`.
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct CairoAssemblyProgramOutput(
	BoundedVec<
		BoundedVec<u8, MaxCairoAssemblyProgramInputLength>,
		MaxCairoAssemblyProgramInputNumber,
	>,
);

impl CairoAssemblyProgramOutput {
	pub fn empty() -> Self {
		CairoAssemblyProgramOutput(BoundedVec::with_bounded_capacity(0))
	}
}
