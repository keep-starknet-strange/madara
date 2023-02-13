#![cfg_attr(not(feature = "std"), no_std)]

/// Cairo Execution Engine pallet.
/// Definition of the pallet's runtime storage items, events, errors, and dispatchable
/// functions.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

/// Module related to the compilation of Sierra programs into Cairo assembly programs.
pub mod compilation;
/// Module related to the execution of Cairo assembly programs.
pub mod execution;
/// Hashing functions.
/// This module contains the implementation of the hashing functions used by Cairo and Starknet.
pub mod hash;
/// The Cairo Execution Engine pallet's runtime custom types.
pub mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// TODO: Uncomment when benchmarking is implemented.
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {

	use crate::{
		compilation::{mock::SierraCompilerMock, SierraCompiler},
		execution::{mock::CairoExecutorMock, CairoExecutor},
		hash,
		types::{
			CairoAssemblyProgamId, CairoAssemblyProgram, CairoAssemblyProgramInput,
			CairoAssemblyProgramOutput, SierraProgram, SierraProgramId,
		},
	};
	use frame_support::{pallet_prelude::*, traits::Randomness};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The type of Randomness we want to specify for this pallet.
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		/// The maximum length of a Sierra program.
		/// This is used to bound the size of the Sierra program code.
		type MaxSierraProgramLength: Get<u32>;
		/// The maximum length of a Cairo assembly program.
		/// This is used to bound the size of the Cairo assembly program code.
		type MaxCairoAssemblyProgramLength: Get<u32>;
	}

	/// The Cairo Execution Engine pallet storage items.
	/// STORAGE

	/// List of all deployed Sierra programs.
	/// Since the key of the map is computed using a hash function, we can use the `Identity`
	/// hasher.
	#[pallet::storage]
	#[pallet::getter(fn sierra_programs)]
	pub(super) type SierraPrograms<T: Config> =
		StorageMap<_, Identity, SierraProgramId, SierraProgram<T>>;

	/// List of all compilied Cairo assembly programs.
	/// Since the key of the map is computed using a hash function, we can use the `Identity`
	/// hasher.
	#[pallet::storage]
	#[pallet::getter(fn cairo_assembly_programs)]
	pub(super) type CairoAssemblyPrograms<T: Config> =
		StorageMap<_, Identity, CairoAssemblyProgamId, CairoAssemblyProgram<T>>;

	/// The Cairo Execution Engine pallet events.
	/// EVENTS
	/// See: https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new Sierra program has been successfully deployed.
		SierraProgramDeployed { id: SierraProgramId, deployer_account: T::AccountId },
		/// A new Cairo assembly program has been successfully compiled.
		CairoAssemblyProgramCompiled {
			sierra_program_id: SierraProgramId,
			cairo_assembly_program_id: CairoAssemblyProgamId,
		},
		/// A Cairo assembly program has been successfully executed.
		CairoAssemblyProgramExecuted {
			cairo_assembly_program_id: CairoAssemblyProgamId,
			caller_account: T::AccountId,
		},
	}

	/// The Cairo Execution Engine pallet custom errors.
	/// ERRORS
	#[pallet::error]
	pub enum Error<T> {
		/// The program is too big and does not comply with maximum size restrictions set in
		/// config.
		ProgramTooBig,
		/// The Sierra program does not exist.
		SierraProgramNotFound,
		/// The Cairo assembly program does not exist.
		CairoAssemblyProgramNotFound,
	}

	/// The Cairo Execution Engine pallet external functions.
	/// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	/// These functions materialize as "extrinsics", which are often compared to transactions.
	/// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Deploy a new Sierra program.
		/// # Arguments
		/// - `origin`: The origin of the call
		/// - `sierra_code`: The Sierra code of the program to deploy.
		/// # TODO
		/// - do benchmarking to determine the appropriate `weight` for this call.
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn deploy_sierra_program(
			origin: OriginFor<T>,
			sierra_code: BoundedVec<u8, T::MaxSierraProgramLength>,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin and retrieve the signer.
			let deployer_account = ensure_signed(origin)?;

			log::info!("Deploying Sierra program from account: {:?}", deployer_account);

			// Call internal function to do the actual deployment.
			Self::do_deploy_sierra_program(&deployer_account, sierra_code)?;

			Ok(())
		}

		/// Compile a Sierra program into a Cairo assembly program and store it in storage.
		/// # Arguments
		/// - `origin`: The origin of the call
		/// - `sierra_program_id`: The id of the Sierra program to compile.
		/// # TODO
		/// - do benchmarking to determine the appropriate `weight` for this call.
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn compile_sierra_program(
			origin: OriginFor<T>,
			sierra_program_id: SierraProgramId,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin and retrieve the signer.
			let deployer_account = ensure_signed(origin)?;
			// Call internal function to do the actual compilation.
			Self::do_compile_sierra_program(&deployer_account, sierra_program_id)?;
			Ok(())
		}

		/// Execute a Cairo assembly program.
		/// # Arguments
		/// - `origin`: The origin of the call
		/// - `cairo_assembly_program_id`: The id of the Cairo assembly program to execute.
		/// - `input`: The input to the Cairo assembly program.
		/// # TODO
		/// - do benchmarking to determine the appropriate `weight` for this call.
		/// - implement the actual execution of the Cairo assembly program.
		/// - implement the return value of the execution.
		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn execute_cairo_assembly_program(
			origin: OriginFor<T>,
			cairo_assembly_program_id: CairoAssemblyProgamId,
			input: CairoAssemblyProgramInput,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin and retrieve the signer.
			let caller_account = ensure_signed(origin)?;
			// Call internal function to do the actual execution.
			Self::do_execute_cairo_assembly_program(
				&caller_account,
				cairo_assembly_program_id,
				input,
			)?;
			Ok(())
		}
	}

	/// The Cairo Execution Engine pallet internal functions.
	impl<T: Config> Pallet<T> {
		/// Deploy a new Sierra program.
		/// # Arguments
		/// - `deployer_account`: The account identifier of the deployer.
		/// - `sierra_code`: The Sierra code of the program to deploy.
		fn do_deploy_sierra_program(
			deployer_account: &T::AccountId,
			sierra_code: BoundedVec<u8, T::MaxSierraProgramLength>,
		) -> Result<SierraProgramId, DispatchError> {
			// TODO: the Sierra program id must be the hash of the code, so that we can find
			// duplicate programs and save storage space.
			let sierra_program_id = Self::gen_sierra_program_id(deployer_account, &sierra_code)?;

			// Validate the Sierra program
			Self::validate_sierra_code(&sierra_code)?;

			// Create the Sierra program instance.
			let sierra_program = SierraProgram::<T> {
				id: sierra_program_id,
				code: sierra_code,
				deployer_account: deployer_account.clone(),
				// The Cairo assembly program id is not known yet.
				// It will be set when the Sierra program is compiled to Cairo assembly.
				cairo_assembly_program_id: None,
			};

			// Insert the Sierra program in storage.
			SierraPrograms::<T>::insert(sierra_program_id, &sierra_program);

			// Emit events.
			Self::deposit_event(Event::SierraProgramDeployed {
				id: sierra_program_id,
				deployer_account: deployer_account.clone(),
			});

			Ok(sierra_program_id)
		}

		/// Compute the id of a Sierra program from it's code.
		/// TODO: move to hash of the code
		pub fn gen_sierra_program_id(
			_deployer_account: &T::AccountId,
			_sierra_code: &BoundedVec<u8, T::MaxSierraProgramLength>,
		) -> Result<SierraProgramId, DispatchError> {
			// Create some randomness.
			let random = T::Randomness::random(&b"sierra"[..]).0;

			// Add some uniqueness because multiple ids can generated in the same block.
			let unique_payload = (
				random,
				frame_system::Pallet::<T>::extrinsic_index().unwrap_or_default(),
				frame_system::Pallet::<T>::block_number(),
			);

			// Turns into a byte array.
			let encoded_payload = unique_payload.encode();
			// Compute the hash and return as id.
			// TODO: use poseidon hash when it is available.
			let _hash = hash::poseidon(&encoded_payload);
			Ok(frame_support::Hashable::blake2_256(&encoded_payload))
		}

		/// Validate the Sierra program code.
		/// TODO: implement proper validation, use the cairo compiler to check.
		pub fn validate_sierra_code(
			sierra_code: &BoundedVec<u8, T::MaxSierraProgramLength>,
		) -> Result<(), DispatchError> {
			// Check that the program is not too big.
			ensure!(
				sierra_code.len() <= T::MaxSierraProgramLength::get() as usize,
				Error::<T>::ProgramTooBig
			);
			Ok(())
		}

		/// Compile a Sierra program into a Cairo assembly program and store it in storage.
		/// # Arguments
		/// - `deployer_account`: The account identifier of the deployer.
		/// - `sierra_program_id`: The id of the Sierra program to compile.
		/// # Returns
		/// - `CairoAssemblyProgamId`: The id of the compiled Cairo assembly program.
		/// # TODO
		/// - implement the compilation.
		pub fn do_compile_sierra_program(
			_deployer_account: &T::AccountId,
			sierra_program_id: SierraProgramId,
		) -> Result<CairoAssemblyProgamId, DispatchError> {
			// Retrieve the Sierra program from storage.
			let mut sierra_program = SierraPrograms::<T>::get(sierra_program_id)
				.ok_or(Error::<T>::SierraProgramNotFound)?;
			// Create an instance of the Cairo compiler.
			let compiler = SierraCompilerMock::default();
			// Compile the Sierra program into a Cairo assembly program.
			let mut cairo_assembly_program = compiler.compile(&sierra_program)?;
			// Generate a unique id for the Cairo assembly program.
			let cairo_assembly_program_id =
				Self::gen_cairo_assembly_program_id(&cairo_assembly_program.code)?;
			// Set the id of the Cairo assembly program.
			cairo_assembly_program.id = Some(cairo_assembly_program_id);
			// Store the Cairo assembly program in storage.
			CairoAssemblyPrograms::<T>::insert(cairo_assembly_program_id, &cairo_assembly_program);
			// Update the Sierra program with the id of the Cairo assembly program.
			sierra_program.cairo_assembly_program_id = Some(cairo_assembly_program_id);
			// Update the Sierra program in storage.
			SierraPrograms::<T>::insert(sierra_program_id, &sierra_program);

			// Emit events.
			Self::deposit_event(Event::CairoAssemblyProgramCompiled {
				sierra_program_id,
				cairo_assembly_program_id,
			});
			// Return the id of the Cairo assembly program.
			Ok(cairo_assembly_program_id)
		}

		/// Execute a Cairo assembly program.
		/// # Arguments
		/// - `caller_account`: The account identifier of the caller.
		/// - `cairo_assembly_program_id`: The id of the Cairo assembly program to execute.
		/// - `input`: The input to pass to the Cairo assembly program.
		/// # Returns
		/// - `CairoAssemblyProgramOutput`: The output of the Cairo assembly program.
		/// # TODO
		/// - implement the execution.
		pub fn do_execute_cairo_assembly_program(
			caller_account: &T::AccountId,
			cairo_assembly_program_id: CairoAssemblyProgamId,
			input: CairoAssemblyProgramInput,
		) -> Result<CairoAssemblyProgramOutput, DispatchError> {
			// Retrieve the Cairo assembly program from storage.
			let cairo_assembly_program = CairoAssemblyPrograms::<T>::get(cairo_assembly_program_id)
				.ok_or(Error::<T>::CairoAssemblyProgramNotFound)?;
			// Create an instance of the Cairo executor.
			// For now, we use a mock executor.
			// When the Cairo VM will be integrated, we will use it.
			let cairo_executor = CairoExecutorMock::default();
			// Execute the Cairo assembly program.
			let output = cairo_executor.execute(&cairo_assembly_program, &input)?;

			// Emit events.
			Self::deposit_event(Event::CairoAssemblyProgramExecuted {
				cairo_assembly_program_id,
				caller_account: caller_account.clone(),
			});
			Ok(output)
		}

		/// Compute the id of a Sierra program from it's code.
		/// TODO: move to hash of the code
		pub fn gen_cairo_assembly_program_id(
			_cairo_assembly_code: &BoundedVec<u8, T::MaxCairoAssemblyProgramLength>,
		) -> Result<CairoAssemblyProgamId, DispatchError> {
			// Create some randomness.
			let random = T::Randomness::random(&b"casm"[..]).0;

			// Add some uniqueness because multiple ids can generated in the same block.
			let unique_payload = (
				random,
				frame_system::Pallet::<T>::extrinsic_index().unwrap_or_default(),
				frame_system::Pallet::<T>::block_number(),
			);

			// Turns into a byte array.
			let encoded_payload = unique_payload.encode();
			// Compute the hash and return as id.
			// TODO: use poseidon hash when it is available.
			let _hash = hash::poseidon(&encoded_payload);
			Ok(frame_support::Hashable::blake2_256(&encoded_payload))
		}
	}
}
