#![cfg_attr(not(feature = "std"), no_std)]


/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

pub mod types;
pub mod origin;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::GetDispatchInfo, traits::UnfilteredDispatchable};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The runtime origin type.
		type RuntimeOrigin: From<crate::types::Origin>;

		/// A sudo-able call.
		type RuntimeCall: Parameter
			+ UnfilteredDispatchable<RuntimeOrigin = <Self as Config>::RuntimeOrigin>
			+ GetDispatchInfo;

	}
	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		StarknetCall { result: DispatchResult },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn starknet_call(origin: OriginFor<T>, call: Box<<T as Config>::RuntimeCall>) -> DispatchResult {
			ensure_none(origin)?;
			let res = call.dispatch_bypass_filter(crate::types::Origin::StarknetTransaction.into());
			Self::deposit_event(Event::StarknetCall{ result: res.map(|_| ()).map_err(|e| e.error) });
			Ok(())
		}
	}
}
