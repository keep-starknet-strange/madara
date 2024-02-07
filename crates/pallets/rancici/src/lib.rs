#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::large_enum_variant)]
pub use pallet::*;

pub mod types;

// #[cfg(test)]
// mod tests;

use frame_support::pallet_prelude::*;
use frame_support::traits::Time;
use frame_system::pallet_prelude::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use starknet_api::api_core::ContractAddress;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    // Pallet storage for storing players' addresses for matchmaking
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn player_pool)]
    pub(super) type PlayerPool<T: Config> = StorageMap<_, Identity, ContractAddress, u64, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub player_pool: Vec<(ContractAddress, u64)>,
        pub _phantom: PhantomData<T>,
    }
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self { player_pool: vec![], _phantom: PhantomData }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for (player, xp) in &self.player_pool {
                PlayerPool::<T>::insert(player, *xp);
            }
        }
    }

    /// Rancici pallet events
    /// EVENTS
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        PlayerJoinedPool { player: ContractAddress },
        MatchedPlayers { player1: ContractAddress, player2: ContractAddress },
    }

    /// Rancici pallet errors
    /// ERRORS
    #[pallet::error]
    pub enum Error<T> {
        PlayerNotFoundInPool,
        PlayerAlreadyInPool,
        NotEnoughPlayersInPool,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight((0, DispatchClass::Mandatory))]
        pub fn join_pool(_origin: OriginFor<T>, address: ContractAddress, xp: u64) -> DispatchResult {
            ensure!(PlayerPool::<T>::contains_key(&address), Error::<T>::PlayerAlreadyInPool);
            PlayerPool::<T>::insert(address, xp);
            Self::deposit_event(Event::PlayerJoinedPool { player: address });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight((0, DispatchClass::Mandatory))]
        pub fn matchmaking(origin: OriginFor<T>) -> DispatchResult {
            let _sender = ensure_signed(origin)?;

            // Collect all players into a vector
            let players = <PlayerPool<T>>::iter().collect::<Vec<_>>();

            // Ensure there are at least two players in the pool
            ensure!(players.len() >= 2, Error::<T>::NotEnoughPlayersInPool);

            // Randomly select two players
            let (player1_index, player2_index) = {
                let mut rng = SmallRng::from_entropy();
                let player1_index = rng.gen_range(0..players.len());
                let player2_index = loop {
                    let index = rng.gen_range(0..players.len());
                    if index != player1_index {
                        break index;
                    }
                };
                (player1_index, player2_index)
            };
            let player1 = players[player1_index].0.clone();
            let player2 = players[player2_index].0.clone();

            // Remove players from storage
            PlayerPool::<T>::remove(&player1);
            PlayerPool::<T>::remove(&player2);

            // Emit event
            Self::deposit_event(Event::MatchedPlayers { player1, player2 });
            Ok(())
        }
    }
}
