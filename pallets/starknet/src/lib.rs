// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

/// Starknet pallet.
/// Definition of the pallet's runtime storage items, events, errors, and dispatchable
/// functions.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
use sp_core::ConstU32;

/// The Starknet pallet's runtime custom types.
pub mod types;

/// Transaction validation logic.
pub mod transaction_validation;

/// State root logic.
pub mod state_root;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// TODO: Uncomment when benchmarking is implemented.
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// Make this configurable.
type MaxTransactionsPendingBlock = ConstU32<1073741824>;

pub use self::pallet::*;
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, traits::Randomness};
	use frame_system::pallet_prelude::*;
	use kp_starknet::{crypto::hash, transaction::Transaction};
	use sp_core::U256;
	use starknet_crypto::FieldElement;

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
		/// How Starknet state root is calculated.
		type StateRoot: Get<U256>;
	}

	/// The Starknet pallet hooks.
	/// HOOKS
	/// # TODO
	/// * Implement the hooks.
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// The block is being finalized. Implement to have something happen.
		fn on_finalize(_n: T::BlockNumber) {}

		/// The block is being initialized. Implement to have something happen.
		fn on_initialize(_: T::BlockNumber) -> Weight {
			Weight::zero()
		}

		/// Perform a module upgrade.
		fn on_runtime_upgrade() -> Weight {
			Weight::zero()
		}
	}

	/// The Starknet pallet storage items.
	/// STORAGE
	/// Current building block's transactions.
	#[pallet::storage]
	#[pallet::getter(fn pending)]
	pub(super) type Pending<T: Config> =
		StorageValue<_, BoundedVec<Transaction, MaxTransactionsPendingBlock>, ValueQuery>;

	/// The Starknet pallet events.
	/// EVENTS
	/// See: `<https://docs.substrate.io/main-docs/build/events-errors/>`
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		HealthCheckHashComputed { x: [u8; 32], y: [u8; 32], hash: [u8; 32] },
	}

	/// The Starknet pallet custom errors.
	/// ERRORS
	#[pallet::error]
	pub enum Error<T> {}

	/// The Starknet pallet external functions.
	/// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	/// These functions materialize as "extrinsics", which are often compared to transactions.
	/// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Ping the pallet to check if it is alive.
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn ping(origin: OriginFor<T>) -> DispatchResult {
			// Make sure the caller is from a signed origin and retrieve the signer.
			let _deployer_account = ensure_signed(origin)?;

			log::info!("Keep Starknet Strange!");

			Self::health_check()?;

			Ok(())
		}
	}

	/// The Starknet pallet internal functions.
	impl<T: Config> Pallet<T> {
		fn health_check() -> Result<(), DispatchError> {
			// Compute a hash of known values to check if the pallet is alive.
			let x = FieldElement::from(1_u32);
			let y = FieldElement::from(2_u32);
			let hash = hash::hash(hash::HashType::Pedersen, &x, &y);
			let x = x.to_bytes_be();
			let y = y.to_bytes_be();
			let hash = hash.to_bytes_be();
			// Emit an event to notify the user.
			Self::deposit_event(Event::HealthCheckHashComputed { x, y, hash });
			Ok(())
		}
	}
}
