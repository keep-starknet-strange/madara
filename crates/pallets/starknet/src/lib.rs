//! A Substrate pallet implementation for Starknet, a decentralized, permissionless, and scalable
//! zk-rollup for general-purpose smart contracts.
//! See the [Starknet documentation](https://docs.starknet.io/) for more information.
//! The code consists of the following sections:
//! 1. Config: The trait Config is defined, which is used to configure the pallet by specifying the
//! parameters and types on which it depends. The trait also includes associated types for
//! RuntimeEvent, StateRoot, SystemHash, and TimestampProvider.
//!
//! 2. Hooks: The Hooks trait is implemented for the pallet, which includes methods to be executed
//! during the block lifecycle: on_finalize, on_initialize, on_runtime_upgrade, and offchain_worker.
//!
//! 3. Storage: Several storage items are defined, including Pending, CurrentBlock, BlockHash,
//! ContractClassHashes, ContractClasses, Nonces, StorageView, LastKnownEthBlock, and
//! FeeTokenAddress. These storage items are used to store and manage data related to the Starknet
//! pallet.
//!
//! 4. Genesis Configuration: The GenesisConfig struct is defined, which is used to set up the
//! initial state of the pallet during genesis. The struct includes fields for contracts,
//! contract_classes, storage, fee_token_address, and _phantom. A GenesisBuild implementation is
//! provided to build the initial state during genesis.
//!
//! 5. Events: A set of events are defined in the Event enum, including KeepStarknetStrange,
//! StarknetEvent, and FeeTokenAddressChanged. These events are emitted during the execution of
//! various pallet functions.
//!
//! 6.Errors: A set of custom errors are defined in the Error enum, which is used to represent
//! various error conditions during the execution of the pallet.
//!
//! 7. Dispatchable Functions: The Pallet struct implements several dispatchable functions (ping,
//! invoke, ...), which allow users to interact with the pallet and invoke state changes. These
//! functions are annotated with weight and return a DispatchResult.
// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::large_enum_variant)]
/// Starknet pallet.
/// Definition of the pallet's runtime storage items, events, errors, and dispatchable
/// functions.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
use sp_core::ConstU32;

/// The Starknet pallet's runtime custom types.
pub mod types;

/// The implementation of the message type.
pub mod message;

/// Transaction validation logic.
pub mod transaction_validation;

/// State root logic.
pub mod state_root;

/// The Starknet pallet's runtime API
pub mod runtime_api;

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

pub(crate) const LOG_TARGET: &str = "runtime::starknet";

pub const ETHEREUM_EXECUTION_RPC: &[u8] = b"starknet::ETHEREUM_EXECUTION_RPC";
pub const ETHEREUM_CONSENSUS_RPC: &[u8] = b"starknet::ETHEREUM_CONSENSUS_RPC";

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: $crate::LOG_TARGET,
			concat!("[{:?}] 🐺 ", $patter), <frame_system::Pallet<T>>::block_number() $(, $values)*
		)
	};
}

#[frame_support::pallet]
pub mod pallet {
    pub extern crate alloc;
    use alloc::str::from_utf8;
    use alloc::string::{String, ToString};
    use alloc::vec;
    use alloc::vec::Vec;

    use blockifier::execution::contract_class::ContractClass;
    use blockifier::execution::entry_point::CallInfo;
    use blockifier::state::cached_state::{CachedState, ContractStorageKey};
    use blockifier::state::state_api::State;
    use blockifier::test_utils::DictStateReader;
    use mp_digest_log::{PostLog, MADARA_ENGINE_ID};
    use mp_starknet::block::{Block as StarknetBlock, Header as StarknetHeader};
    use mp_starknet::crypto::commitment;
    use mp_starknet::crypto::hash::pedersen::PedersenHasher;
    use mp_starknet::execution::{ClassHashWrapper, ContractAddressWrapper, ContractClassWrapper};
    use mp_starknet::storage::{StarknetStorageSchemaVersion, PALLET_STARKNET_SCHEMA};
    use mp_starknet::traits::hash::Hasher;
    use mp_starknet::transaction::types::{
        EventError, EventWrapper as StarknetEventType, StateDiffError, Transaction, TxType,
    };
    use serde_json::from_str;
    use sp_core::{H256, U256};
    use sp_runtime::offchain::http;
    use sp_runtime::traits::UniqueSaturatedInto;
    use sp_runtime::DigestItem;
    use starknet_api::api_core::{ClassHash, ContractAddress, Nonce};
    use starknet_api::hash::StarkFelt;
    use starknet_api::state::{StateDiff, StorageKey};
    use starknet_api::stdlib::collections::HashMap;
    use starknet_api::transaction::EventContent;
    use starknet_api::StarknetApiError;
    use types::{EthBlockNumber, OffchainWorkerError};
    type ContractClassMapping = HashMap<ClassHash, ContractClass>;

    use frame_support::pallet_prelude::*;
    use frame_support::sp_runtime::offchain::storage::StorageValueRef;
    use frame_support::traits::{OriginTrait, Time};
    use frame_system::pallet_prelude::*;

    use super::*;
    use crate::message::{get_messages_events, LAST_FINALIZED_BLOCK_QUERY};
    use crate::types::{ContractStorageKeyWrapper, EthLogs, NonceWrapper, StarkFeltWrapper};

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// How Starknet state root is calculated.
        type StateRoot: Get<U256>;
        /// The hashing function to use.
        type SystemHash: Hasher;
        /// The time idk what.
        type TimestampProvider: Time;
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
            <Pallet<T>>::store_block(U256::from(UniqueSaturatedInto::<u128>::unique_saturated_into(
                frame_system::Pallet::<T>::block_number(),
            )));
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
        fn offchain_worker(n: T::BlockNumber) {
            log!(info, "Running offchain worker at block {:?}.", n);

            match Self::process_l1_messages() {
                Ok(_) => log!(info, "Successfully executed L1 messages"),
                Err(err) => match err {
                    OffchainWorkerError::NoLastKnownEthBlock => {
                        log!(info, "No last known Ethereum block number found. Skipping execution of L1 messages.")
                    }
                    _ => log!(error, "Failed to execute L1 messages: {:?}", err),
                },
            }
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
    pub(super) type CurrentBlock<T: Config> = StorageValue<_, StarknetBlock, ValueQuery>;

    // Mapping for block number and hashes.
    #[pallet::storage]
    #[pallet::getter(fn block_hash)]
    pub(super) type BlockHash<T: Config> = StorageMap<_, Twox64Concat, U256, H256, ValueQuery>;

    /// Mapping from Starknet contract address to the contract's class hash.
    #[pallet::storage]
    #[pallet::getter(fn contract_class_hash_by_address)]
    pub(super) type ContractClassHashes<T: Config> =
        StorageMap<_, Twox64Concat, ContractAddressWrapper, ClassHashWrapper, ValueQuery>;

    /// Mapping from Starknet class hash to contract class.
    #[pallet::storage]
    #[pallet::getter(fn contract_class_by_class_hash)]
    pub(super) type ContractClasses<T: Config> =
        StorageMap<_, Twox64Concat, ClassHashWrapper, ContractClassWrapper, ValueQuery>;

    /// Mapping from Starknet contract address to its nonce.
    #[pallet::storage]
    #[pallet::getter(fn nonce)]
    pub(super) type Nonces<T: Config> = StorageMap<_, Twox64Concat, ContractAddressWrapper, NonceWrapper, ValueQuery>;

    /// Mapping from Starknet contract storage key to its value.
    #[pallet::storage]
    #[pallet::getter(fn storage)]
    pub(super) type StorageView<T: Config> =
        StorageMap<_, Twox64Concat, ContractStorageKeyWrapper, StarkFeltWrapper, ValueQuery>;

    /// The last processed Ethereum block number for L1 messages consumption.
    /// This is used to avoid re-processing the same Ethereum block multiple times.
    /// This is used by the offchain worker.
    /// # TODO
    /// * Find a more relevant name for this.
    #[pallet::storage]
    #[pallet::getter(fn last_known_eth_block)]
    pub(super) type LastKnownEthBlock<T: Config> = StorageValue<_, u64>;

    /// The address of the fee token ERC20 contract.
    #[pallet::storage]
    #[pallet::getter(fn fee_token_address)]
    pub(super) type FeeTokenAddress<T: Config> = StorageValue<_, ContractAddressWrapper, ValueQuery>;

    /// Starknet genesis configuration.
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// The contracts to be deployed at genesis.
        /// This is a vector of tuples, where the first element is the contract address and the
        /// second element is the contract class hash.
        /// This can be used to start the chain with a set of pre-deployed contracts, for example in
        /// a test environment or in the case of a migration of an existing chain state.
        pub contracts: Vec<(ContractAddressWrapper, ClassHashWrapper)>,
        /// The contract classes to be deployed at genesis.
        /// This is a vector of tuples, where the first element is the contract class hash and the
        /// second element is the contract class definition.
        /// Same as `contracts`, this can be used to start the chain with a set of pre-deployed
        /// contracts classes.
        pub contract_classes: Vec<(ClassHashWrapper, ContractClassWrapper)>,
        pub storage: Vec<(ContractStorageKeyWrapper, StarkFeltWrapper)>,
        /// The address of the fee token.
        /// Must be set to the address of the fee token ERC20 contract.
        pub fee_token_address: ContractAddressWrapper,
        pub _phantom: PhantomData<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                contracts: vec![],
                contract_classes: vec![],
                storage: vec![],
                fee_token_address: ContractAddressWrapper::default(),
                _phantom: PhantomData,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            <Pallet<T>>::store_block(U256::zero());
            frame_support::storage::unhashed::put::<StarknetStorageSchemaVersion>(
                PALLET_STARKNET_SCHEMA,
                &StarknetStorageSchemaVersion::V1,
            );

            for (address, class_hash) in self.contracts.iter() {
                ContractClassHashes::<T>::insert(address, class_hash);
            }

            for (class_hash, contract_class) in self.contract_classes.iter() {
                ContractClasses::<T>::insert(class_hash, contract_class);
            }

            for (key, value) in self.storage.iter() {
                StorageView::<T>::insert(key, value);
            }

            LastKnownEthBlock::<T>::set(None);
            // Set the fee token address from the genesis config.
            FeeTokenAddress::<T>::set(self.fee_token_address);
        }
    }

    /// The Starknet pallet events.
    /// EVENTS
    /// See: `<https://docs.substrate.io/main-docs/build/events-errors/>`
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        KeepStarknetStrange,
        /// Regular Starknet event
        StarknetEvent(StarknetEventType),
        /// Emitted when fee token address is changed.
        /// This is emitted by the `set_fee_token_address` extrinsic.
        /// [old_fee_token_address, new_fee_token_address]
        FeeTokenAddressChanged {
            old_fee_token_address: ContractAddressWrapper,
            new_fee_token_address: ContractAddressWrapper,
        },
    }

    /// The Starknet pallet custom errors.
    /// ERRORS
    #[pallet::error]
    pub enum Error<T> {
        AccountNotDeployed,
        TransactionExecutionFailed,
        ClassHashAlreadyDeclared,
        ContractClassHashUnknown,
        ContractClassAlreadyAssociated,
        ContractClassMustBeSpecified,
        AccountAlreadyDeployed,
        ContractAddressAlreadyAssociated,
        InvalidContractClass,
        ClassHashMustBeSpecified,
        TooManyPendingTransactions,
        StateReaderError,
        EmitEventError,
        StateDiffError,
    }

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
            Pending::<T>::try_append(Transaction::default()).unwrap();
            log!(info, "Keep Starknet Strange!");
            Self::deposit_event(Event::KeepStarknetStrange);
            Ok(())
        }

        /// The invoke transaction is the main transaction type used to invoke contract functions in
        /// Starknet.
        /// See `https://docs.starknet.io/documentation/architecture_and_concepts/Blocks/transactions/#invoke_transaction`.
        /// # Arguments
        ///
        /// * `origin` - The origin of the transaction.
        /// * `transaction` - The Starknet transaction.
        ///
        ///  # Returns
        ///
        /// * `DispatchResult` - The result of the transaction.
        ///
        /// # TODO
        /// * Compute weight
        #[pallet::call_index(1)]
        #[pallet::weight(0)]
        pub fn invoke(_origin: OriginFor<T>, mut transaction: Transaction) -> DispatchResult {
            // TODO: add origin check when proxy pallet added

            // Check if contract is deployed
            ensure!(ContractClassHashes::<T>::contains_key(transaction.sender_address), Error::<T>::AccountNotDeployed);

            // Get current block
            let block = Self::current_block();
            // Get fee token address
            let fee_token_address = Self::fee_token_address();
            let state = &mut Self::create_state_reader()?;
            let call_info = transaction.execute(state, block, TxType::InvokeTx, None, fee_token_address);
            match call_info {
                Ok(Some(mut v)) => {
                    Self::emit_events(&mut v, &mut transaction).map_err(|_| Error::<T>::EmitEventError)?;
                    log!(debug, "Transaction executed successfully: {:?}", v);
                }
                Ok(None) => {
                    log!(error, "Transaction execution failed: no call info while it was expected");
                    return Err(Error::<T>::TransactionExecutionFailed.into());
                }
                Err(e) => {
                    log!(error, "Transaction execution failed: {:?}", e);
                    return Err(Error::<T>::TransactionExecutionFailed.into());
                }
            }

            Self::apply_state_diffs(state).map_err(|_| Error::<T>::StateDiffError)?;
            // Append the transaction to the pending transactions.
            Pending::<T>::try_append(transaction).unwrap();

            // TODO: Apply state diff and update state root

            Ok(())
        }

        /// The declare transaction is used to introduce new classes into the state of Starknet,
        /// enabling other contracts to deploy instances of those classes or using them in a library
        /// call. See `https://docs.starknet.io/documentation/architecture_and_concepts/Blocks/transactions/#declare_transaction`.
        /// # Arguments
        ///
        /// * `origin` - The origin of the transaction.
        /// * `transaction` - The Starknet transaction.
        ///
        ///  # Returns
        ///
        /// * `DispatchResult` - The result of the transaction.
        ///
        /// # TODO
        /// * Compute weight
        #[pallet::call_index(2)]
        #[pallet::weight(0)]
        pub fn declare(_origin: OriginFor<T>, transaction: Transaction) -> DispatchResult {
            // TODO: add origin check when proxy pallet added

            // Check if contract is deployed
            ensure!(ContractClassHashes::<T>::contains_key(transaction.sender_address), Error::<T>::AccountNotDeployed);

            // Check that class hash is not None
            ensure!(transaction.call_entrypoint.class_hash.is_some(), Error::<T>::ClassHashMustBeSpecified);

            let class_hash = transaction.call_entrypoint.class_hash.unwrap();

            // Check class hash is not already declared
            ensure!(!ContractClasses::<T>::contains_key(class_hash), Error::<T>::ClassHashAlreadyDeclared);

            // Check that contract class is not None
            ensure!(transaction.contract_class.is_some(), Error::<T>::ContractClassMustBeSpecified);

            // Get current block
            let block = Self::current_block();
            // Get fee token address
            let fee_token_address = Self::fee_token_address();
            // Create state reader from substrate storage
            let state = &mut Self::create_state_reader()?;

            // Parse contract class
            let contract_class = transaction
                .clone()
                .contract_class
                .unwrap()
                .to_starknet_contract_class()
                .or(Err(Error::<T>::InvalidContractClass))?;

            // Execute transaction
            match transaction.execute(state, block, TxType::DeclareTx, Some(contract_class.clone()), fee_token_address)
            {
                Ok(_) => {
                    log!(debug, "Declare Transaction executed successfully.");
                }
                Err(e) => {
                    log!(error, "Transaction execution failed: {:?}", e);
                    return Err(Error::<T>::TransactionExecutionFailed.into());
                }
            }

            // Append the transaction to the pending transactions.
            Pending::<T>::try_append(transaction.clone()).or(Err(Error::<T>::TooManyPendingTransactions))?;

            // Associate contract class to class hash
            Self::set_contract_class_hash(class_hash, contract_class.into())?;
            Self::apply_state_diffs(state).map_err(|_| Error::<T>::StateDiffError)?;

            // TODO: Update class hashes root

            Ok(())
        }

        /// Since StarkNet v0.10.1 the deploy_account transaction replaces the deploy transaction
        /// for deploying account contracts. To use it, you should first pre-fund your
        /// would-be account address so that you could pay the transaction fee (see here for more
        /// details) . You can then send the deploy_account transaction. See `https://docs.starknet.io/documentation/architecture_and_concepts/Blocks/transactions/#deploy_account_transaction`.
        /// # Arguments
        ///
        /// * `origin` - The origin of the transaction.
        /// * `transaction` - The Starknet transaction.
        ///
        ///  # Returns
        ///
        /// * `DispatchResult` - The result of the transaction.
        ///
        /// # TODO
        /// * Compute weight
        #[pallet::call_index(3)]
        #[pallet::weight(0)]
        pub fn deploy_account(_origin: OriginFor<T>, transaction: Transaction) -> DispatchResult {
            // TODO: add origin check when proxy pallet added

            // Check if contract is deployed
            ensure!(
                !ContractClassHashes::<T>::contains_key(transaction.sender_address),
                Error::<T>::AccountAlreadyDeployed
            );

            // Get current block
            let block = Self::current_block();
            // Get fee token address
            let fee_token_address = Self::fee_token_address();

            let state = &mut Self::create_state_reader()?;
            match transaction.execute(state, block, TxType::DeployAccountTx, None, fee_token_address) {
                Ok(v) => {
                    log!(debug, "Transaction executed successfully: {:?}", v.unwrap());
                }
                Err(e) => {
                    log!(error, "Transaction execution failed: {:?}", e);
                    return Err(Error::<T>::TransactionExecutionFailed.into());
                }
            }

            // Append the transaction to the pending transactions.
            Pending::<T>::try_append(transaction.clone()).unwrap();

            // Associate contract class to class hash
            // TODO: update state root
            Self::apply_state_diffs(state).map_err(|_| Error::<T>::StateDiffError)?;

            Ok(())
        }

        /// Consume a message from L1.
        ///
        /// # Arguments
        ///
        /// * `origin` - The origin of the transaction.
        /// * `transaction` - The Starknet transaction.
        ///
        /// # Returns
        ///
        /// * `DispatchResult` - The result of the transaction.
        ///
        /// # TODO
        /// * Compute weight
        #[pallet::call_index(4)]
        #[pallet::weight(0)]
        pub fn consume_l1_message(_origin: OriginFor<T>, transaction: Transaction) -> DispatchResult {
            // TODO: add origin check when proxy pallet added
            // Check if contract is deployed
            ensure!(ContractClassHashes::<T>::contains_key(transaction.sender_address), Error::<T>::AccountNotDeployed);

            let block = Self::current_block();
            let fee_token_address = Self::fee_token_address();
            let state = &mut Self::create_state_reader()?;
            match transaction.execute(state, block, TxType::L1HandlerTx, None, fee_token_address) {
                Ok(v) => {
                    log!(debug, "Transaction executed successfully: {:?}", v.unwrap());
                }
                Err(e) => {
                    log!(error, "Transaction execution failed: {:?}", e);
                    return Err(Error::<T>::TransactionExecutionFailed.into());
                }
            }

            // Append the transaction to the pending transactions.
            Pending::<T>::try_append(transaction.clone()).or(Err(Error::<T>::TooManyPendingTransactions))?;

            Self::apply_state_diffs(state).map_err(|_| Error::<T>::StateDiffError)?;

            Ok(())
        }

        /// Set the value of the fee token address.
        ///
        /// # Arguments
        ///
        /// * `origin` - The origin of the transaction.
        /// * `fee_token_address` - The value of the fee token address.
        ///
        /// # Returns
        ///
        /// * `DispatchResult` - The result of the transaction.
        ///
        /// # TODO
        /// * Add some limitations on how often this can be called.
        #[pallet::call_index(5)]
        #[pallet::weight(0)]
        pub fn set_fee_token_address(
            origin: OriginFor<T>,
            fee_token_address: ContractAddressWrapper,
        ) -> DispatchResult {
            // Only root can set the fee token address.
            ensure_root(origin)?;
            // Get current fee token address.
            let current_fee_token_address = Self::fee_token_address();
            // Update the fee token address.
            FeeTokenAddress::<T>::put(fee_token_address);
            // Emit event.
            Self::deposit_event(Event::FeeTokenAddressChanged {
                old_fee_token_address: current_fee_token_address,
                new_fee_token_address: fee_token_address,
            });
            Ok(())
        }
    }

    /// The Starknet pallet internal functions.
    impl<T: Config> Pallet<T> {
        /// Get current block hash.
        ///
        /// # Returns
        ///
        /// The current block hash.
        #[inline(always)]
        pub fn current_block_hash() -> H256 {
            Self::current_block().header().hash()
        }

        /// Get the block hash of the previous block.
        ///
        /// # Arguments
        ///
        /// * `current_block_number` - The number of the current block.
        ///
        /// # Returns
        ///
        /// The block hash of the parent (previous) block or 0 if the current block is 0.
        #[inline(always)]
        pub fn parent_block_hash(current_block_number: &U256) -> H256 {
            if current_block_number == &U256::zero() {
                H256::zero()
            } else {
                Self::block_hash(current_block_number - 1)
            }
        }

        /// Get the current block timestamp.
        ///
        /// # Returns
        ///
        /// The current block timestamp.
        #[inline(always)]
        pub fn block_timestamp() -> u64 {
            T::TimestampProvider::now().unique_saturated_into()
        }

        /// Get the number of transactions in the block.
        #[inline(always)]
        pub fn transaction_count() -> u128 {
            Self::pending().len() as u128
        }

        /// Get the number of events in the block.
        #[inline(always)]
        pub fn event_count() -> u128 {
            Self::pending().iter().flat_map(|tx| tx.events.iter()).count() as u128
        }

        /// Store a Starknet block in the blockchain.
        ///
        /// # Arguments
        ///
        /// * `block_number` - The block number.
        fn store_block(block_number: U256) {
            // TODO: Use actual values.
            let parent_block_hash = Self::parent_block_hash(&block_number);
            let pending = Self::pending();

            let global_state_root = U256::zero();
            let sequencer_address = ContractAddressWrapper::default();
            let block_timestamp = Self::block_timestamp();
            let transaction_count = pending.len() as u128;
            let (transaction_commitment, (event_commitment, event_count)) =
                commitment::calculate_commitments::<PedersenHasher>(&pending);
            let protocol_version = None;
            let extra_data = None;

            let block = StarknetBlock::new(StarknetHeader::new(
                parent_block_hash,
                block_number,
                global_state_root,
                sequencer_address,
                block_timestamp,
                transaction_count,
                transaction_commitment,
                event_count,
                event_commitment,
                protocol_version,
                extra_data,
            ));
            // Save the current block.
            CurrentBlock::<T>::put(block.clone());
            // Save the block number <> hash mapping.
            BlockHash::<T>::insert(block_number, block.header().hash());
            Pending::<T>::kill();

            let digest = DigestItem::Consensus(MADARA_ENGINE_ID, PostLog::BlockHash(block.header().hash()).encode());
            frame_system::Pallet::<T>::deposit_log(digest);
        }

        /// Associate a contract class hash with a contract class info
        ///
        /// # Arguments
        ///
        /// * `contract_class_hash` - The contract class hash.
        /// * `class_info` - The contract class info.
        fn set_contract_class_hash(
            contract_class_hash: ClassHashWrapper,
            class_info: ContractClassWrapper,
        ) -> Result<(), DispatchError> {
            // Check if the contract address is already associated with a contract class hash.
            ensure!(
                !ContractClasses::<T>::contains_key(contract_class_hash),
                Error::<T>::ContractClassAlreadyAssociated
            );
            ContractClasses::<T>::insert(contract_class_hash, class_info);

            Ok(())
        }

        /// Associate a contract address with a contract class hash.
        ///
        /// # Arguments
        ///
        /// * `contract_address` - The contract address.
        /// * `contract_class_hash` - The contract class hash.
        fn set_class_hash_at(
            contract_address: ContractAddressWrapper,
            contract_class_hash: ClassHashWrapper,
        ) -> Result<(), DispatchError> {
            // Check if the contract address is already associated with a contract class hash.
            ensure!(
                !ContractClassHashes::<T>::contains_key(contract_address),
                Error::<T>::ContractAddressAlreadyAssociated
            );

            ContractClassHashes::<T>::insert(contract_address, contract_class_hash);

            Ok(())
        }

        /// Emit events from the call info.
        ///
        /// # Arguments
        ///
        /// * `call_info` - The call info.
        ///
        /// # Returns
        ///
        /// The result of the operation.
        #[inline(always)]
        fn emit_events(call_info: &mut CallInfo, transaction: &mut Transaction) -> Result<(), EventError> {
            call_info.inner_calls.iter_mut().try_for_each(|inner_call: &mut CallInfo| {
                inner_call.execution.events.sort_by_key(|ordered_event| ordered_event.order);
                inner_call.execution.events.iter().try_for_each(|ordered_event| {
                    Self::emit_event(&ordered_event.event, inner_call.call.storage_address, transaction)
                })
            })
        }

        /// Emit an event from the call info in substrate.
        ///
        /// # Arguments
        ///
        /// * `event` - The Starknet event.
        /// * `from_address` - The contract address that emitted the event.
        ///
        /// # Error
        ///
        /// Returns an error if the event construction fails.
        #[inline(always)]
        fn emit_event(
            event: &EventContent,
            from_address: ContractAddress,
            transaction: &mut Transaction,
        ) -> Result<(), EventError> {
            log!(debug, "Transaction event: {:?}", event);
            let sn_event = StarknetEventType::builder()
                .with_event_content(event.clone())
                .with_from_address(from_address)
                .build()?;
            transaction.events.try_push(sn_event.clone()).map_err(|_| EventError::TooManyEvents)?;
            Self::deposit_event(Event::StarknetEvent(sn_event));
            Ok(())
        }

        /// Apply the state diff returned by the starknet execution.
        ///
        /// # Argument
        ///
        /// * `state` - The state constructed for the starknet execution engine.
        ///
        /// # Error
        ///
        /// Returns an error if it fails to apply the state diff of newly deployed contracts.
        pub fn apply_state_diffs(state: &CachedState<DictStateReader>) -> Result<(), StateDiffError> {
            // Get all the state diffs
            let StateDiff { deployed_contracts, storage_diffs, declared_classes: _declared_classes, nonces, .. } =
                state.to_state_diff();
            // Store the newly deployed contracts in substrate storage.
            deployed_contracts.iter().try_for_each(|(address, class_hash)| {
                Self::set_class_hash_at(address.0.0.0, class_hash.0.0).map_err(|_| {
                    log!(
                        error,
                        "Failed to save newly deployed contract at address: {:?} with class hash: {:?}",
                        address.0.0.0,
                        class_hash.0.0
                    );
                    StateDiffError::DeployedContractError
                })
            })?;
            // Store the modifications of storage vars.
            storage_diffs.iter().for_each(|(address, diffs)| {
                diffs.iter().for_each(|(key, value)| {
                    StorageView::<T>::insert((address.0.0.0, H256::from_slice(&key.0.0.0)), U256::from(value.0))
                })
            });
            // Store the new nonces.
            nonces.iter().for_each(|(address, nonce)| Nonces::<T>::insert(address.0.0.0, U256::from(nonce.0.0)));
            Ok(())
        }

        /// Create a state reader.
        ///
        /// # Returns
        ///
        /// The state reader.
        fn create_state_reader() -> Result<CachedState<DictStateReader>, DispatchError> {
            // TODO: Handle errors and propagate them to the caller.

            let address_to_class_hash: HashMap<ContractAddress, ClassHash> = ContractClassHashes::<T>::iter()
                .map(|(key, value)| {
                    (
                        ContractAddress::try_from(StarkFelt::new(key).unwrap()).unwrap(),
                        ClassHash(StarkFelt::new(value).unwrap()),
                    )
                })
                .collect();

            let address_to_nonce: HashMap<ContractAddress, Nonce> = Nonces::<T>::iter()
                .map(|(key, value)| {
                    (
                        ContractAddress::try_from(StarkFelt::new(key).unwrap()).unwrap(),
                        Nonce(StarkFelt::new(value.into()).unwrap()),
                    )
                })
                .collect();

            let storage_view: HashMap<ContractStorageKey, StarkFelt> = StorageView::<T>::iter()
                .map(|(key, value)| {
                    (
                        (
                            ContractAddress::try_from(StarkFelt::new(key.0).unwrap()).unwrap(),
                            StorageKey::try_from(StarkFelt::new(key.1.into()).unwrap()).unwrap(),
                        ),
                        StarkFelt::new(value.into()).unwrap(),
                    )
                })
                .collect();

            let class_hash_to_class: ContractClassMapping = ContractClasses::<T>::iter()
                .map(|(key, value)| {
                    let class_hash = ClassHash(StarkFelt::new(key)?);
                    let contract_class = value.to_starknet_contract_class().unwrap();
                    Ok((class_hash, contract_class))
                })
                .collect::<Result<ContractClassMapping, StarknetApiError>>()
                .map_err(|_| Error::<T>::StateReaderError)?
                .into_iter()
                .collect();

            Ok(CachedState::new(DictStateReader {
                address_to_class_hash,
                address_to_nonce,
                storage_view,
                class_hash_to_class,
            }))
        }

        /// Returns Ethereum RPC URL from Storage
        fn get_eth_rpc_url() -> Result<String, OffchainWorkerError> {
            let eth_execution_rpc_url = StorageValueRef::persistent(ETHEREUM_EXECUTION_RPC)
                .get::<Vec<u8>>()
                .map_err(|_| OffchainWorkerError::GetStorageFailed)?
                .ok_or(OffchainWorkerError::EthRpcNotSet)?;

            let endpoint: &str =
                core::str::from_utf8(&eth_execution_rpc_url).map_err(|_| OffchainWorkerError::FormatBytesFailed)?;

            if endpoint.is_empty() {
                return Err(OffchainWorkerError::EthRpcNotSet);
            }

            Ok(endpoint.to_string())
        }

        /// Queries an Eth json rpc node.
        ///
        /// # Arguments
        ///
        /// * `request` - The request to be sent.
        ///
        /// # Returns
        ///
        /// The result of the query formatted as a `String`.
        ///
        /// # Errors
        ///
        /// If and error happens during the query or deserialization.
        fn query_eth(request: &str) -> Result<String, OffchainWorkerError> {
            let res = http::Request::post(&Self::get_eth_rpc_url()?, vec![request])
                .add_header("content-type", "application/json")
                .send()
                .map_err(OffchainWorkerError::HttpError)?
                .wait()
                .map_err(OffchainWorkerError::RequestError)?;
            let body_bytes = res.body().collect::<Vec<u8>>();
            Ok(from_utf8(&body_bytes).map_err(OffchainWorkerError::ToBytesError)?.to_string())
        }

        /// Fetches L1 messages and execute them.
        /// This function is called by the offchain worker.
        /// It is executed in a separate thread.
        /// # Returns
        /// The result of the offchain worker execution.
        fn process_l1_messages() -> Result<(), OffchainWorkerError> {
            // Get the last known block from storage.
            let last_known_eth_block = Self::last_known_eth_block().ok_or(OffchainWorkerError::NoLastKnownEthBlock)?;
            // Query L1 for the last finalized block.
            let body_str = Self::query_eth(LAST_FINALIZED_BLOCK_QUERY)?;
            let res: EthBlockNumber = from_str(&body_str).map_err(|_| OffchainWorkerError::SerdeError)?;
            let last_finalized_block = u64::from_str_radix(&res.result.number[2..], 16).unwrap();
            // Check if there are new messages to be processed.
            if last_finalized_block > last_known_eth_block {
                // Read the new messages from L1.
                let body_str = Self::query_eth(&get_messages_events(last_known_eth_block, last_finalized_block))?;
                let res: EthLogs = from_str(&body_str).map_err(|_| OffchainWorkerError::SerdeError)?;
                // Iterate over the messages and execute them.
                res.result.iter().try_for_each(|message| {
                    // Execute the message.
                    Self::consume_l1_message(OriginFor::<T>::none(), message.try_into_transaction()?)
                        .map_err(OffchainWorkerError::ConsumeMessageError)
                })?;
            }
            Ok(())
        }
    }
}
