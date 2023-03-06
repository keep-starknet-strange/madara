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

/// Hashing logic.
pub mod hash;

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
	use hash::Hasher;
	use kp_starknet::{
		block::wrapper::{block::Block, header::Header},
		storage::{StarknetStorageSchema, PALLET_STARKNET_SCHEMA},
		transaction::Transaction,
	};
	use sp_core::{H256, U256};
	use sp_runtime::traits::UniqueSaturatedInto;

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
		/// The hashing function to use.
		type SystemHash: Hasher;
	}

	/// The Starknet pallet hooks.
	/// HOOKS
	/// # TODO
	/// * Implement the hooks.
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// The block is being finalized.
		fn on_finalize(_n: T::BlockNumber) {
			// Create a new Starknet block and store it.
			<Pallet<T>>::store_block(U256::from(
				UniqueSaturatedInto::<u128>::unique_saturated_into(
					frame_system::Pallet::<T>::block_number(),
				),
			));
		}

		/// The block is being initialized. Implement to have something happen.
		fn on_initialize(_: T::BlockNumber) -> Weight {
			Weight::zero()
		}

		/// Perform a module upgrade.
		fn on_runtime_upgrade() -> Weight {
			Weight::zero()
		}

		/// Run offchain tasks.
		/// See: `<https://docs.substrate.io/reference/how-to-guides/offchain-workers/>`
		/// # Arguments
		/// * `n` - The block number.
		/// # TODO
		/// * Investigate how we can use offchain workers for Starknet specific tasks. An example
		///   might be the communication with the prover.
		fn offchain_worker(n: T::BlockNumber) {
			log::trace!(
				target: "runtime::starknet",
				"Running offchain worker at block {:?}.",
				n,
			)
		}
	}

	/// The Starknet pallet storage items.
	/// STORAGE
	/// Current building block's transactions.
	#[pallet::storage]
	#[pallet::getter(fn pending)]
	pub(super) type Pending<T: Config> =
		StorageValue<_, BoundedVec<Transaction, MaxTransactionsPendingBlock>, ValueQuery>;

	/// The current Starknet block.
	#[pallet::storage]
	#[pallet::getter(fn current_block)]
	pub(super) type CurrentBlock<T: Config> = StorageValue<_, Block>;

	// Mapping for block number and hashes.
	#[pallet::storage]
	#[pallet::getter(fn block_hash)]
	pub(super) type BlockHash<T: Config> = StorageMap<_, Twox64Concat, U256, H256, ValueQuery>;

	/// Starknet genesis configuration.
	#[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig {}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			<Pallet<T>>::store_block(U256::zero());
			frame_support::storage::unhashed::put::<StarknetStorageSchema>(
				PALLET_STARKNET_SCHEMA,
				&StarknetStorageSchema::V1,
			);
		}
	}

	/// The Starknet pallet events.
	/// EVENTS
	/// See: `<https://docs.substrate.io/main-docs/build/events-errors/>`
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

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
			Ok(())
		}
	}

	/// The Starknet pallet internal functions.
	impl<T: Config> Pallet<T> {
		/// Get current block hash
		pub fn current_block_hash() -> Option<H256> {
			Self::current_block().map(|block| block.header.hash())
		}

		/// Store a Starknet block in the blockchain.
		/// # Arguments
		/// * `block_number` - The block number.
		/// # TODO
		/// * Implement the function.
		fn store_block(block_number: U256) {
			// TODO: Use actual values.
			let sequencer_address = U256::zero();
			let block = Block::new(Header::new(block_number, sequencer_address));
			// Save the current block.
			CurrentBlock::<T>::put(block.clone());
			// Save the block number <> hash mapping.
			BlockHash::<T>::insert(block_number, block.header.hash());
		}
	}
}
