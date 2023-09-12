#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use mp_starknet::execution::types::Felt252Wrapper;
pub use pallet::*;
pub use pallet_starknet;

#[frame_support::pallet]
pub mod pallet {

    use super::*;

    /// The maximum length of the async message pool.
    type AsyncMessagePoolMaxLength = ConstU32<10000>;
    /// The length of a selector.
    type SelectorLength = ConstU32<4>;

    #[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, scale_info::TypeInfo, MaxEncodedLen)]
    #[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
    pub struct AsyncMessage {
        /// The account that sent the message.
        pub sender_account: Felt252Wrapper,
        /// The address of the contract to run autonomous call on.
        pub target_contract: Felt252Wrapper,
        /// The selector of the function to call.
        pub target_selector: BoundedVec<Felt252Wrapper, SelectorLength>,
        /// An optional selector to check if the call should be made.
        pub should_run_selector: Option<BoundedVec<Felt252Wrapper, SelectorLength>>,
        /// An optional selector to check if the message should be removed from the pool.
        pub should_kill_selector: Option<BoundedVec<Felt252Wrapper, SelectorLength>>,
        /// The maximal amount of Cairo steps to use for the call.
        pub step_limit: u64,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_starknet::Config {
        // Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// The block is being initialized. Implement to have something happen.
        fn on_initialize(_: T::BlockNumber) -> Weight {
            Self::process_async_message_pool();
            Weight::zero()
        }
    }

    /// The async contracts pallet storage items.
    #[pallet::storage]
    #[pallet::getter(fn async_message_pool)]
    pub(super) type AsyncMessagePool<T: Config> =
        StorageValue<_, BoundedVec<AsyncMessage, AsyncMessagePoolMaxLength>, OptionQuery>;

    /// Errors.
    #[pallet::error]
    pub enum Error<T> {
        ReachedBoundedVecLimit,
        ContractClassHashUnknown,
    }

    /// Events.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        KeepStarknetStrange,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// # Arguments
        ///
        /// * `origin` - The origin of the transaction.
        #[pallet::call_index(0)]
        #[pallet::weight({0})]
        pub fn register_async_message(origin: OriginFor<T>, message: AsyncMessage) -> DispatchResult {
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;
            assert!(
                pallet_starknet::ContractClasses::<T>::contains_key(&message.target_contract),
                "ContractClassHashUnknown"
            );
            AsyncMessagePool::<T>::try_append(message).or(Err(Error::<T>::ReachedBoundedVecLimit))?;

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn process_async_message_pool() {
            // Read the async message pool
            let async_message_pool = AsyncMessagePool::<T>::get().unwrap_or_default();
            // Iterate over the async message pool
            for (index, async_message) in async_message_pool.iter().enumerate() {
                Self::process_async_message(index, async_message.clone());
            }
        }

        fn process_async_message(index: usize, async_message: AsyncMessage) {}
    }
}
