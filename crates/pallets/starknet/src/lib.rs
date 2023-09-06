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
/// An adapter for the blockifier state related traits
pub mod blockifier_state_adapter;
#[cfg(feature = "std")]
pub mod genesis_loader;
/// The implementation of the message type.
pub mod message;
/// The Starknet pallet's runtime API
pub mod runtime_api;
/// Transaction validation logic.
pub mod transaction_validation;
/// The Starknet pallet's runtime custom types.
pub mod types;
/// Util functions for madara.
#[cfg(feature = "std")]
pub mod utils;

/// Everything needed to run the pallet offchain workers
mod offchain_worker;

#[cfg(test)]
mod tests;

pub use pallet::*;

#[macro_use]
pub extern crate alloc;
use alloc::str::from_utf8_unchecked;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use blockifier::block_context::BlockContext;
use blockifier::execution::contract_class::ContractClass;
use blockifier::execution::entry_point::{CallInfo, ExecutionResources};
use blockifier_state_adapter::BlockifierStateAdapter;
use frame_support::pallet_prelude::*;
use frame_support::traits::Time;
use frame_system::pallet_prelude::*;
use mp_digest_log::MADARA_ENGINE_ID;
use mp_starknet::block::{Block as StarknetBlock, Header as StarknetHeader, MaxTransactions};
use mp_starknet::constants::INITIAL_GAS;
use mp_starknet::crypto::commitment::{self};
use mp_starknet::execution::types::{
    CallEntryPointWrapper, ClassHashWrapper, ContractAddressWrapper, EntryPointTypeWrapper, Felt252Wrapper,
};
use mp_starknet::sequencer_address::{InherentError, InherentType, DEFAULT_SEQUENCER_ADDRESS, INHERENT_IDENTIFIER};
use mp_starknet::storage::{StarknetStorageSchemaVersion, PALLET_STARKNET_SCHEMA};
use mp_starknet::traits::hash::{DefaultHasher, HasherT};
use mp_starknet::transaction::types::{
    DeclareTransaction, DeployAccountTransaction, EventError, EventWrapper as StarknetEventType, InvokeTransaction,
    Transaction, TransactionExecutionInfoWrapper, TransactionReceiptWrapper, TxType,
};
use sp_runtime::traits::UniqueSaturatedInto;
use sp_runtime::DigestItem;
use sp_std::result;
use starknet_api::api_core::{ChainId, ContractAddress};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::EventContent;
use starknet_crypto::FieldElement;

use crate::alloc::string::ToString;
use crate::types::{ContractStorageKeyWrapper, NonceWrapper, StorageKeyWrapper};

pub(crate) const LOG_TARGET: &str = "runtime::starknet";

pub const ETHEREUM_EXECUTION_RPC: &[u8] = b"starknet::ETHEREUM_EXECUTION_RPC";
pub const ETHEREUM_CONSENSUS_RPC: &[u8] = b"starknet::ETHEREUM_CONSENSUS_RPC";
pub(crate) const NONCE_DECODE_FAILURE: u8 = 1;

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $pattern:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: $crate::LOG_TARGET,
			concat!("[{:?}] 🐺 ", $pattern), <frame_system::Pallet<T>>::block_number() $(, $values)*
		)
	};
}

#[frame_support::pallet]
pub mod pallet {

    use mp_starknet::execution::types::CompiledClassHashWrapper;

    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    /// We're coupling the starknet pallet to the tx payment pallet to be able to override the fee
    /// mechanism and comply with starknet which uses an ER20 as fee token
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// The hashing function to use.
        type SystemHash: HasherT + DefaultHasher;
        /// The time idk what.
        type TimestampProvider: Time;
        /// A configuration for base priority of unsigned transactions.
        ///
        /// This is exposed so that it can be tuned for particular runtime, when
        /// multiple pallets send unsigned transactions.
        #[pallet::constant]
        type UnsignedPriority: Get<TransactionPriority>;
        /// A configuration for longevity of transactions.
        ///
        /// This is exposed so that it can be tuned for particular runtime to
        /// set how long transactions are kept in the mempool.
        #[pallet::constant]
        type TransactionLongevity: Get<TransactionLongevity>;
        /// A bool to disable transaction fees and make all transactions free
        #[pallet::constant]
        type DisableTransactionFee: Get<bool>;
        /// A bool to disable Nonce validation
        type DisableNonceValidation: Get<bool>;
        #[pallet::constant]
        type InvokeTxMaxNSteps: Get<u32>;
        #[pallet::constant]
        type ValidateMaxNSteps: Get<u32>;
        #[pallet::constant]
        type ProtocolVersion: Get<u8>;
        #[pallet::constant]
        type ChainId: Get<Felt252Wrapper>;
        #[pallet::constant]
        type MaxRecursionDepth: Get<u32>;
    }

    /// The Starknet pallet hooks.
    /// HOOKS
    /// # TODO
    /// * Implement the hooks.
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// The block is being finalized.
        fn on_finalize(_n: T::BlockNumber) {
            assert!(SeqAddrUpdate::<T>::take(), "Sequencer address must be set for the block");

            // Create a new Starknet block and store it.
            <Pallet<T>>::store_block(UniqueSaturatedInto::<u64>::unique_saturated_into(
                frame_system::Pallet::<T>::block_number(),
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
        fn offchain_worker(n: T::BlockNumber) {
            log!(info, "Running offchain worker at block {:?}.", n);

            match Self::process_l1_messages() {
                Ok(_) => log!(info, "Successfully executed L1 messages"),
                Err(err) => match err {
                    offchain_worker::OffchainWorkerError::NoLastKnownEthBlock => {
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
        StorageValue<_, BoundedVec<(Transaction, TransactionReceiptWrapper), MaxTransactions>, ValueQuery>;

    /// Current building block's events.
    // TODO: This is redundant information but more performant
    // than removing this and computing events from the tx reciepts.
    // More info: https://github.com/keep-starknet-strange/madara/pull/561
    #[pallet::storage]
    #[pallet::getter(fn pending_events)]
    pub(super) type PendingEvents<T: Config> =
        StorageValue<_, BoundedVec<StarknetEventType, MaxTransactions>, ValueQuery>;

    /// Mapping for block number and hashes.
    /// Safe to use `Identity` as the key is already a hash.
    #[pallet::storage]
    #[pallet::getter(fn block_hash)]
    pub(super) type BlockHash<T: Config> = StorageMap<_, Identity, u64, Felt252Wrapper, ValueQuery>;

    /// Mapping from Starknet contract address to the contract's class hash.
    /// Safe to use `Identity` as the key is already a hash.
    #[pallet::storage]
    #[pallet::getter(fn contract_class_hash_by_address)]
    pub(super) type ContractClassHashes<T: Config> =
        StorageMap<_, Identity, ContractAddressWrapper, ClassHashWrapper, OptionQuery>;

    /// Mapping from Starknet class hash to contract class.
    /// Safe to use `Identity` as the key is already a hash.
    #[pallet::storage]
    #[pallet::getter(fn contract_class_by_class_hash)]
    pub(super) type ContractClasses<T: Config> = StorageMap<_, Identity, ClassHashWrapper, ContractClass, OptionQuery>;

    /// Mapping from Starknet Sierra class hash to  Casm compiled contract class.
    /// Safe to use `Identity` as the key is already a hash.
    #[pallet::storage]
    #[pallet::getter(fn compiled_class_hash_by_class_hash)]
    pub(super) type CompiledClassHashes<T: Config> =
        StorageMap<_, Identity, ClassHashWrapper, CompiledClassHashWrapper, OptionQuery>;

    /// Mapping from Starknet contract address to its nonce.
    /// Safe to use `Identity` as the key is already a hash.
    #[pallet::storage]
    #[pallet::getter(fn nonce)]
    pub(super) type Nonces<T: Config> = StorageMap<_, Identity, ContractAddressWrapper, NonceWrapper, ValueQuery>;

    /// Mapping from Starknet contract storage key to its value.
    /// Safe to use `Identity` as the key is already a hash.
    #[pallet::storage]
    #[pallet::getter(fn storage)]
    pub(super) type StorageView<T: Config> =
        StorageMap<_, Identity, ContractStorageKeyWrapper, Felt252Wrapper, ValueQuery>;

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

    /// Current sequencer address.
    #[pallet::storage]
    #[pallet::getter(fn sequencer_address)]
    pub type SequencerAddress<T: Config> = StorageValue<_, ContractAddressWrapper, ValueQuery>;

    /// Ensure the sequencer address was updated for this block.
    #[pallet::storage]
    #[pallet::getter(fn seq_addr_update)]
    pub type SeqAddrUpdate<T: Config> = StorageValue<_, bool, ValueQuery>;

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
        pub contract_classes: Vec<(ClassHashWrapper, ContractClass)>,
        pub storage: Vec<(ContractStorageKeyWrapper, Felt252Wrapper)>,
        /// The address of the fee token.
        /// Must be set to the address of the fee token ERC20 contract.
        pub fee_token_address: ContractAddressWrapper,
        pub _phantom: PhantomData<T>,
        pub seq_addr_updated: bool,
    }

    /// `Default` impl required by `pallet::GenesisBuild`.
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                contracts: vec![],
                contract_classes: vec![],
                storage: vec![],
                fee_token_address: ContractAddressWrapper::default(),
                _phantom: PhantomData,
                seq_addr_updated: true,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            <Pallet<T>>::store_block(0);
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
            SeqAddrUpdate::<T>::put(self.seq_addr_updated);
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
        TooManyPendingEvents,
        StateReaderError,
        EmitEventError,
        StateDiffError,
        ContractNotFound,
        ReachedBoundedVecLimit,
        TransactionConversionError,
        SequencerAddressNotValid,
    }

    /// The Starknet pallet external functions.
    /// Dispatchable functions allows users to interact with the pallet and invoke state changes.
    /// These functions materialize as "extrinsics", which are often compared to transactions.
    /// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set the current block author's sequencer address.
        ///
        /// This call should be invoked exactly once per block. It will set a default value at
        /// the finalization phase, if this call hasn't been invoked by that time.
        ///
        /// The dispatch origin for this call must be `Inherent`.
        #[pallet::call_index(0)]
        #[pallet::weight((0, DispatchClass::Mandatory))]
        pub fn set_sequencer_address(origin: OriginFor<T>, addr: [u8; 32]) -> DispatchResult {
            ensure_none(origin)?;
            // The `SeqAddrUpdate` storage item is initialized to `true` in the genesis build. In
            // block 1 we skip the storage update check, and the `on_finalize` hook
            // updates the storage item to `false`. Initializing the storage item with
            // `false` causes the `on_finalize` hook to panic.
            if UniqueSaturatedInto::<u64>::unique_saturated_into(frame_system::Pallet::<T>::block_number()) > 1 {
                assert!(!SeqAddrUpdate::<T>::exists(), "Sequencer address can be updated only once in the block");
            }

            let addr = ContractAddressWrapper::try_from(&addr).map_err(|_| Error::<T>::SequencerAddressNotValid)?;
            SequencerAddress::<T>::put(addr);
            SeqAddrUpdate::<T>::put(true);
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
        #[pallet::call_index(1)]
        #[pallet::weight({0})]
        pub fn invoke(origin: OriginFor<T>, transaction: InvokeTransaction) -> DispatchResult {
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;
            // Check if contract is deployed
            ensure!(ContractClassHashes::<T>::contains_key(transaction.sender_address), Error::<T>::AccountNotDeployed);

            // Get current block context
            let block_context = Self::get_block_context();
            let chain_id = T::ChainId::get();
            let transaction: Transaction = transaction.from_invoke(chain_id);

            let call_info = transaction.execute(
                &mut BlockifierStateAdapter::<T>::default(),
                &block_context,
                TxType::Invoke,
                T::DisableNonceValidation::get(),
            );
            let receipt = match call_info {
                Ok(TransactionExecutionInfoWrapper {
                    validate_call_info: _validate_call_info,
                    execute_call_info,
                    fee_transfer_call_info,
                    actual_fee,
                    actual_resources: _actual_resources,
                }) => {
                    log!(debug, "Invoke Transaction executed successfully: {:?}", execute_call_info);

                    let events = Self::emit_events_for_calls(execute_call_info, fee_transfer_call_info)?;

                    TransactionReceiptWrapper {
                        events: BoundedVec::try_from(events).map_err(|_| Error::<T>::ReachedBoundedVecLimit)?,
                        transaction_hash: transaction.hash,
                        tx_type: TxType::Invoke,
                        actual_fee: actual_fee.0.into(),
                    }
                }
                Err(e) => {
                    log!(error, "Invoke Transaction execution failed: {:?}", e);
                    return Err(Error::<T>::TransactionExecutionFailed.into());
                }
            };

            // Append the transaction to the pending transactions.
            Pending::<T>::try_append((transaction, receipt)).map_err(|_| Error::<T>::TooManyPendingTransactions)?;

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
        #[pallet::call_index(2)]
        #[pallet::weight({0})]
        pub fn declare(origin: OriginFor<T>, transaction: DeclareTransaction) -> DispatchResult {
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;

            let chain_id = T::ChainId::get();

            let transaction: Transaction = transaction.from_declare(chain_id);
            // Check that contract class is not None
            transaction.contract_class.clone().ok_or(Error::<T>::ContractClassMustBeSpecified)?;

            // Check that the class hash is not None
            let class_hash = transaction.call_entrypoint.class_hash.ok_or(Error::<T>::ClassHashMustBeSpecified)?;

            // Check if contract is deployed
            ensure!(ContractClassHashes::<T>::contains_key(transaction.sender_address), Error::<T>::AccountNotDeployed);

            // Check class hash is not already declared
            ensure!(!ContractClasses::<T>::contains_key(class_hash), Error::<T>::ClassHashAlreadyDeclared);

            // Get current block context
            let block_context = Self::get_block_context();

            // Execute transaction
            let call_info = transaction.execute(
                &mut BlockifierStateAdapter::<T>::default(),
                &block_context,
                TxType::Declare,
                T::DisableNonceValidation::get(),
            );
            let receipt = match call_info {
                Ok(TransactionExecutionInfoWrapper {
                    validate_call_info: _validate_call_info,
                    execute_call_info,
                    fee_transfer_call_info,
                    actual_fee,
                    actual_resources: _actual_resources,
                }) => {
                    log!(trace, "Declare Transaction executed successfully: {:?}", execute_call_info);

                    let events = Self::emit_events_for_calls(execute_call_info, fee_transfer_call_info)?;

                    TransactionReceiptWrapper {
                        events: BoundedVec::try_from(events).map_err(|_| Error::<T>::ReachedBoundedVecLimit)?,
                        transaction_hash: transaction.hash,
                        tx_type: TxType::Declare,
                        actual_fee: actual_fee.0.into(),
                    }
                }
                Err(e) => {
                    log!(error, "Declare Transaction execution failed: {:?}", e);
                    return Err(Error::<T>::TransactionExecutionFailed.into());
                }
            };

            // Append the transaction to the pending transactions.
            Pending::<T>::try_append((transaction, receipt)).or(Err(Error::<T>::TooManyPendingTransactions))?;

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
        #[pallet::call_index(3)]
        #[pallet::weight({0})]
        pub fn deploy_account(origin: OriginFor<T>, transaction: DeployAccountTransaction) -> DispatchResult {
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;

            let chain_id = T::ChainId::get();
            let transaction: Transaction =
                transaction.from_deploy(chain_id).map_err(|_| Error::<T>::TransactionConversionError)?;

            // Check if contract is deployed
            ensure!(
                !ContractClassHashes::<T>::contains_key(transaction.sender_address),
                Error::<T>::AccountAlreadyDeployed
            );

            // Get current block context
            let block_context = Self::get_block_context();

            // Execute transaction
            let call_info = transaction.execute(
                &mut BlockifierStateAdapter::<T>::default(),
                &block_context,
                TxType::DeployAccount,
                T::DisableNonceValidation::get(),
            );
            let receipt = match call_info {
                Ok(TransactionExecutionInfoWrapper {
                    validate_call_info: _validate_call_info,
                    execute_call_info,
                    fee_transfer_call_info,
                    actual_fee,
                    actual_resources: _actual_resources,
                }) => {
                    log!(trace, "Deploy_account Transaction executed successfully: {:?}", execute_call_info);

                    let events = Self::emit_events_for_calls(execute_call_info, fee_transfer_call_info)?;

                    TransactionReceiptWrapper {
                        events: BoundedVec::try_from(events).map_err(|_| Error::<T>::ReachedBoundedVecLimit)?,
                        transaction_hash: transaction.hash,
                        tx_type: TxType::DeployAccount,
                        actual_fee: actual_fee.0.into(),
                    }
                }
                Err(e) => {
                    log!(error, "Deploy_account Transaction execution failed: {:?}", e);
                    return Err(Error::<T>::TransactionExecutionFailed.into());
                }
            };

            // Append the transaction to the pending transactions.
            Pending::<T>::try_append((transaction, receipt)).or(Err(Error::<T>::TooManyPendingTransactions))?;

            // Associate contract class to class hash
            // TODO: update state root

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
        #[pallet::weight({0})]
        pub fn consume_l1_message(origin: OriginFor<T>, transaction: Transaction) -> DispatchResult {
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;

            // Check if contract is deployed
            ensure!(ContractClassHashes::<T>::contains_key(transaction.sender_address), Error::<T>::AccountNotDeployed);

            let block_context = Self::get_block_context();
            match transaction.execute(
                &mut BlockifierStateAdapter::<T>::default(),
                &block_context,
                TxType::L1Handler,
                true,
            ) {
                Ok(v) => {
                    log!(debug, "Successfully consumed a message from L1: {:?}", v);
                }
                Err(e) => {
                    log!(error, "Failed to consume a message from L1: {:?}", e);
                    return Err(Error::<T>::TransactionExecutionFailed.into());
                }
            }

            // Append the transaction to the pending transactions.
            Pending::<T>::try_append((transaction.clone(), TransactionReceiptWrapper::default()))
                .or(Err(Error::<T>::TooManyPendingTransactions))?;

            Ok(())
        }
    }

    #[pallet::inherent]
    impl<T: Config> ProvideInherent for Pallet<T> {
        type Call = Call<T>;
        type Error = InherentError;
        const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

        fn create_inherent(data: &InherentData) -> Option<Self::Call> {
            let inherent_data = data
                .get_data::<InherentType>(&INHERENT_IDENTIFIER)
                .expect("Sequencer address inherent data not correctly encoded")
                .unwrap_or(DEFAULT_SEQUENCER_ADDRESS);
            Some(Call::set_sequencer_address { addr: inherent_data })
        }

        fn check_inherent(_call: &Self::Call, _data: &InherentData) -> result::Result<(), Self::Error> {
            Ok(())
        }

        fn is_inherent(call: &Self::Call) -> bool {
            matches!(call, Call::set_sequencer_address { .. })
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        /// Validate unsigned call to this module.
        ///
        /// By default unsigned transactions are disallowed, but implementing the validator
        /// here we make sure that some particular calls (in this case all calls)
        /// are being whitelisted and marked as valid.
        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            // The priority right now is the max u64 - nonce because for unsigned transactions we need to
            // determine an absolute priority. For now we use that for the benchmark (lowest nonce goes first)
            // otherwise we have a nonce error and everything fails.
            // Once we have a real fee market this is where we'll chose the most profitable transaction.

            let transaction = Self::get_call_transaction(call.clone()).map_err(|_| InvalidTransaction::Call)?;

            let transaction_type = transaction.tx_type.clone();
            let transaction_nonce = transaction.nonce;
            let sender_address = transaction.sender_address;

            let nonce_for_priority: u64 =
                transaction_nonce.try_into().map_err(|_| InvalidTransaction::Custom(NONCE_DECODE_FAILURE))?;

            let mut valid_transaction_builder = ValidTransaction::with_tag_prefix("starknet")
                .priority(u64::MAX - nonce_for_priority)
                .and_provides((sender_address, transaction_nonce))
                .longevity(T::TransactionLongevity::get())
                .propagate(true);

            match transaction_type {
                TxType::Invoke | TxType::Declare => {
                    // validate the transaction
                    Self::validate_tx(transaction, transaction_type)?;
                    // add the requires tag
                    let sender_nonce = Pallet::<T>::nonce(sender_address);
                    if transaction_nonce.0 > sender_nonce.0 {
                        valid_transaction_builder = valid_transaction_builder
                            .and_requires((sender_address, Felt252Wrapper(transaction_nonce.0 - FieldElement::ONE)));
                    }
                }
                _ => (),
            };

            valid_transaction_builder.build()
        }

        /// From substrate documentation:
        /// Validate the call right before dispatch.
        /// This method should be used to prevent transactions already in the pool
        /// (i.e. passing validate_unsigned) from being included in blocks in case
        /// they became invalid since being added to the pool.
        ///
        /// In the default implementation of pre_dispatch for the ValidateUnsigned trait,
        /// this function calls the validate_unsigned function in order to verify validity
        /// before dispatch. In our case, since transaction was already validated in
        /// `validate_unsigned` we can just return Ok.
        fn pre_dispatch(_call: &Self::Call) -> Result<(), TransactionValidityError> {
            Ok(())
        }
    }
}

/// The Starknet pallet internal functions.
impl<T: Config> Pallet<T> {
    /// Returns the transaction for the Call
    ///
    /// # Arguments
    ///
    /// * `call` - The call to get the sender address for
    ///
    /// # Returns
    ///
    /// The transaction
    fn get_call_transaction(call: Call<T>) -> Result<Transaction, ()> {
        match call {
            Call::<T>::invoke { transaction } => Ok(transaction.from_invoke(T::ChainId::get())),
            Call::<T>::declare { transaction } => Ok(transaction.from_declare(T::ChainId::get())),
            Call::<T>::deploy_account { transaction } => transaction.from_deploy(T::ChainId::get()).map_err(|_| ()),
            Call::<T>::consume_l1_message { transaction } => Ok(transaction),
            _ => Err(()),
        }
    }

    /// Validates transaction and returns substrate error if any.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to be validated.
    /// * `tx_type` - The type of the transaction.
    ///
    /// # Error
    ///
    /// Returns an error if transaction validation fails.
    fn validate_tx(transaction: Transaction, tx_type: TxType) -> Result<(), TransactionValidityError> {
        let block_context = Self::get_block_context();
        let mut state: BlockifierStateAdapter<T> = BlockifierStateAdapter::<T>::default();
        let mut execution_resources = ExecutionResources::default();
        transaction.validate_account_tx(&mut state, &mut execution_resources, &block_context, &tx_type).map_err(
            |e| {
                log!(error, "Transaction pool validation failed: {:?}", e);
                TransactionValidityError::Invalid(InvalidTransaction::BadProof)
            },
        )?;

        Ok(())
    }

    /// Creates a [BlockContext] object. The [BlockContext] is needed by the blockifier to execute
    /// properly the transaction. Substrate caches data so it's fine to call multiple times this
    /// function, only the first transaction/block will be "slow" to load these data.
    fn get_block_context() -> BlockContext {
        let block_number = UniqueSaturatedInto::<u64>::unique_saturated_into(frame_system::Pallet::<T>::block_number());
        let block_timestamp = Self::block_timestamp();

        // Its value is checked when we set it so it's fine to unwrap
        let fee_token_address: StarkFelt = Self::fee_token_address().0.into();
        let fee_token_address = ContractAddress::try_from(fee_token_address).unwrap();
        let sequencer_address: StarkFelt = Self::sequencer_address().0.into();
        let sequencer_address = ContractAddress::try_from(sequencer_address).unwrap();

        let chain_id = Self::chain_id_str();

        let vm_resource_fee_cost = Default::default();
        // FIXME: https://github.com/keep-starknet-strange/madara/issues/329
        let gas_price = 10;
        BlockContext {
            block_number: BlockNumber(block_number),
            block_timestamp: BlockTimestamp(block_timestamp),
            chain_id: ChainId(chain_id),
            sequencer_address,
            fee_token_address,
            vm_resource_fee_cost,
            invoke_tx_max_n_steps: T::InvokeTxMaxNSteps::get(),
            validate_max_n_steps: T::ValidateMaxNSteps::get(),
            gas_price,
            max_recursion_depth: T::MaxRecursionDepth::get() as usize,
        }
    }

    /// convert chain_id
    #[inline(always)]
    pub fn chain_id_str() -> String {
        unsafe { from_utf8_unchecked(&T::ChainId::get().0.to_bytes_be()).to_string() }
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
    pub fn parent_block_hash(current_block_number: &u64) -> Felt252Wrapper {
        if current_block_number == &0 { Felt252Wrapper::ZERO } else { Self::block_hash(current_block_number - 1) }
    }

    /// Get the current block timestamp in seconds.
    ///
    /// # Returns
    ///
    /// The current block timestamp in seconds.
    #[inline(always)]
    pub fn block_timestamp() -> u64 {
        let timestamp_in_millisecond: u64 = T::TimestampProvider::now().unique_saturated_into();
        timestamp_in_millisecond / 1000
    }

    /// Get the number of transactions in the block.
    #[inline(always)]
    pub fn transaction_count() -> u128 {
        Self::pending().len() as u128
    }

    /// Get the number of events in the block.
    #[inline(always)]
    pub fn event_count() -> u128 {
        Self::pending_events().len() as u128
    }

    /// Call a smart contract function.
    pub fn call_contract(
        address: ContractAddressWrapper,
        function_selector: Felt252Wrapper,
        calldata: Vec<Felt252Wrapper>,
    ) -> Result<Vec<Felt252Wrapper>, DispatchError> {
        // Get current block context
        let block_context = Self::get_block_context();
        // Get class hash
        let class_hash = ContractClassHashes::<T>::try_get(address).map_err(|_| Error::<T>::ContractNotFound)?;

        let entrypoint = CallEntryPointWrapper::new(
            Some(class_hash),
            EntryPointTypeWrapper::External,
            Some(function_selector),
            BoundedVec::try_from(calldata).unwrap_or_default(),
            address,
            ContractAddressWrapper::default(),
            INITIAL_GAS.into(),
            None,
        );

        match entrypoint.execute(&mut BlockifierStateAdapter::<T>::default(), block_context) {
            Ok(v) => {
                log!(debug, "Successfully called a smart contract function: {:?}", v);
                let result = v.execution.retdata.0.iter().map(|x| (*x).into()).collect();
                Ok(result)
            }
            Err(e) => {
                log!(error, "Failed to call a smart contract function: {:?}", e);
                Err(Error::<T>::TransactionExecutionFailed.into())
            }
        }
    }

    /// Get storage value at
    pub fn get_storage_at(
        contract_address: ContractAddressWrapper,
        key: StorageKeyWrapper,
    ) -> Result<Felt252Wrapper, DispatchError> {
        // Get state
        ensure!(ContractClassHashes::<T>::contains_key(contract_address), Error::<T>::ContractNotFound);
        Ok(Self::storage((contract_address, key)))
    }

    /// Store a Starknet block in the blockchain.
    ///
    /// # Arguments
    ///
    /// * `block_number` - The block number.
    fn store_block(block_number: u64) {
        let parent_block_hash = Self::parent_block_hash(&block_number);
        let pending = Self::pending();

        let global_state_root = Felt252Wrapper::default();

        let sequencer_address = Self::sequencer_address();
        let block_timestamp = Self::block_timestamp();
        let transaction_count = pending.len() as u128;

        let mut transactions: Vec<Transaction> = Vec::with_capacity(pending.len());
        let mut receipts: Vec<TransactionReceiptWrapper> = Vec::with_capacity(pending.len());

        // For loop to iterate once on pending.
        for (transaction, receipt) in pending.into_iter() {
            transactions.push(transaction);
            receipts.push(receipt);
        }

        let events = Self::pending_events();
        let (transaction_commitment, event_commitment) =
            commitment::calculate_commitments::<T::SystemHash>(&transactions, &events);
        let protocol_version = T::ProtocolVersion::get();
        let extra_data = None;

        let block = StarknetBlock::new(
            StarknetHeader::new(
                parent_block_hash,
                block_number,
                global_state_root,
                sequencer_address,
                block_timestamp,
                transaction_count,
                transaction_commitment,
                events.len() as u128,
                event_commitment,
                protocol_version,
                extra_data,
            ),
            // Safe because `transactions` is build from the `pending` bounded vec,
            // which has the same size limit of `MaxTransactions`
            BoundedVec::try_from(transactions).expect("max(len(transactions)) <= MaxTransactions"),
            BoundedVec::try_from(receipts).expect("max(len(receipts)) <= MaxTransactions"),
        );
        // Save the block number <> hash mapping.
        let blockhash = block.header().hash(T::SystemHash::hasher());
        BlockHash::<T>::insert(block_number, blockhash);

        // Kill pending storage.
        Pending::<T>::kill();
        PendingEvents::<T>::kill();

        let digest = DigestItem::Consensus(MADARA_ENGINE_ID, mp_digest_log::Log::Block(block).encode());
        frame_system::Pallet::<T>::deposit_log(digest);
    }

    /// Emit events from the call info.
    ///
    /// # Arguments
    ///
    /// * `call_info` — A ref to the call info structure.
    /// * `events` — A mutable ref to a resulting list of events
    /// * `next_order` — Next expected event order, has to be 0 for a top level invocation
    ///
    /// # Returns
    ///
    /// Next expected event order
    #[inline(always)]
    fn emit_events_in_call_info(
        call_info: &CallInfo,
        events: &mut Vec<StarknetEventType>,
        next_order: usize,
    ) -> Result<usize, EventError> {
        let mut event_idx = 0;
        let mut inner_call_idx = 0;
        let mut next_order = next_order;

        loop {
            // Emit current call's events as long as they have sequential orders
            if event_idx < call_info.execution.events.len() {
                let ordered_event = &call_info.execution.events[event_idx];
                if ordered_event.order == next_order {
                    let event_type = Self::emit_event(&ordered_event.event, call_info.call.storage_address)?;
                    events.push(event_type);
                    next_order += 1;
                    event_idx += 1;
                    continue;
                }
            }

            // Go deeper to find the continuation of the sequence
            if inner_call_idx < call_info.inner_calls.len() {
                next_order =
                    Self::emit_events_in_call_info(&call_info.inner_calls[inner_call_idx], events, next_order)?;
                inner_call_idx += 1;
                continue;
            }

            // At this point we have iterated over all sequential events and visited all internal calls
            break;
        }

        if event_idx < call_info.execution.events.len() {
            // Normally this should not happen and we trust blockifier to produce correct event orders
            log!(
                debug,
                "Invalid event #{} order: expected {}, got {}\nCall info: {:#?}",
                event_idx,
                next_order,
                call_info.execution.events[event_idx].order,
                call_info
            );
            return Err(EventError::InconsistentOrdering);
        }

        Ok(next_order)
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
    fn emit_event(event: &EventContent, from_address: ContractAddress) -> Result<StarknetEventType, EventError> {
        log!(debug, "Transaction event: {:?}", event);
        let sn_event =
            StarknetEventType::builder().with_event_content(event.clone()).with_from_address(from_address).build()?;
        Self::deposit_event(Event::StarknetEvent(sn_event.clone()));

        PendingEvents::<T>::try_append(sn_event.clone()).map_err(|_| EventError::TooManyEvents)?;
        Ok(sn_event)
    }

    /// Estimate the fee associated with transaction
    pub fn estimate_fee(transaction: Transaction) -> Result<(u64, u64), DispatchError> {
        if !transaction.is_query {
            return Err(DispatchError::Other("Cannot estimate_fee with is_query = false"));
        }

        match transaction.execute(
            &mut BlockifierStateAdapter::<T>::default(),
            &Self::get_block_context(),
            transaction.tx_type.clone(),
            T::DisableNonceValidation::get(),
        ) {
            Ok(v) => {
                log!(debug, "Successfully estimated fee: {:?}", v);
                if let Some(gas_usage) = v.actual_resources.get("l1_gas_usage") {
                    Ok((v.actual_fee.0 as u64, *gas_usage as u64))
                } else {
                    Err(Error::<T>::TransactionExecutionFailed.into())
                }
            }
            Err(e) => {
                log!(error, "Failed to estimate fee: {:?}", e);
                Err(Error::<T>::TransactionExecutionFailed.into())
            }
        }
    }

    /// Returns the hasher used by the runtime.
    pub fn get_system_hash() -> T::SystemHash {
        T::SystemHash::hasher()
    }

    pub fn emit_events_for_calls(
        execute_call_info: Option<CallInfo>,
        fee_transfer_call_info: Option<CallInfo>,
    ) -> Result<Vec<StarknetEventType>, Error<T>> {
        let mut events = Vec::new();
        match (execute_call_info, fee_transfer_call_info) {
            (Some(exec), Some(fee)) => {
                Self::emit_events_in_call_info(&exec, &mut events, 0).map_err(|_| Error::<T>::EmitEventError)?;
                Self::emit_events_in_call_info(&fee, &mut events, 0).map_err(|_| Error::<T>::EmitEventError)?;
            }
            (_, Some(fee)) => {
                Self::emit_events_in_call_info(&fee, &mut events, 0).map_err(|_| Error::<T>::EmitEventError)?;
            }
            _ => {}
        };
        Ok(events)
    }

    pub fn chain_id() -> Felt252Wrapper {
        T::ChainId::get()
    }
}
