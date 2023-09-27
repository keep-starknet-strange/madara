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

use blockifier::execution::entry_point::{CallEntryPoint, CallType, EntryPointExecutionContext};
use blockifier::state::cached_state::ContractStorageKey;
use blockifier::state::state_api::State;
use blockifier::transaction::objects::{TransactionExecutionInfo, TransactionExecutionResult};
use starknet_api::state::StorageKey;
use starknet_api::transaction::{Calldata, Event as StarknetEvent, Fee};

#[cfg(test)]
mod tests;

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
use mp_block::{Block as StarknetBlock, Header as StarknetHeader};
use mp_digest_log::MADARA_ENGINE_ID;
use mp_fee::INITIAL_GAS;
use mp_felt::Felt252Wrapper;
use mp_hashers::HasherT;
use mp_sequencer_address::{InherentError, InherentType, DEFAULT_SEQUENCER_ADDRESS, INHERENT_IDENTIFIER};
use mp_state::{FeeConfig, StateChanges};
use mp_storage::{StarknetStorageSchemaVersion, PALLET_STARKNET_SCHEMA};
use mp_transactions::execution::{Execute, Validate};
use mp_transactions::{
    DeclareTransaction, DeployAccountTransaction, HandleL1MessageTransaction, InvokeTransaction, Transaction,
    UserAndL1HandlerTransaction, UserTransaction,
};
use sp_runtime::traits::UniqueSaturatedInto;
use sp_runtime::DigestItem;
use sp_std::result;
use starknet_api::api_core::{ChainId, ClassHash, CompiledClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::TransactionHash;
use starknet_crypto::FieldElement;

use crate::alloc::string::ToString;
use crate::types::StorageSlot;

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
        type SystemHash: HasherT;
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
    #[pallet::unbounded]
    #[pallet::getter(fn pending)]
    pub(super) type Pending<T: Config> = StorageValue<_, Vec<Transaction>, ValueQuery>;

    // Keep the hashes of the transactions stored in Pending
    // One should not be updated without the other !!!
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn pending_hashes)]
    pub(super) type PendingHashes<T: Config> = StorageValue<_, Vec<TransactionHash>, ValueQuery>;

    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn tx_events)]
    pub(super) type TxEvents<T: Config> = StorageMap<_, Identity, TransactionHash, Vec<StarknetEvent>, ValueQuery>;

    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn tx_revert_error)]
    pub(super) type TxRevertError<T: Config> = StorageMap<_, Identity, TransactionHash, String, OptionQuery>;
    /// The Starknet pallet storage items.
    /// STORAGE
    /// Mapping of contract address to state root.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn contract_state_root_by_address)]
    pub(super) type ContractsStateRoots<T: Config> =
        StorageMap<_, Identity, ContractAddress, Felt252Wrapper, OptionQuery>;

    /// Pending storage slot updates
    /// STORAGE
    /// Mapping storage key to storage value.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn pending_storage_changes)]
    pub(super) type PendingStorageChanges<T: Config> =
        StorageMap<_, Identity, ContractAddress, Vec<StorageSlot>, ValueQuery>;

    /// Mapping for block number and hashes.
    /// Safe to use `Identity` as the key is already a hash.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn block_hash)]
    pub(super) type BlockHash<T: Config> = StorageMap<_, Identity, u64, Felt252Wrapper, ValueQuery>;

    /// Mapping from Starknet contract address to the contract's class hash.
    /// Safe to use `Identity` as the key is already a hash.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn contract_class_hash_by_address)]
    pub(super) type ContractClassHashes<T: Config> = StorageMap<_, Identity, ContractAddress, ClassHash, ValueQuery>;

    /// Mapping from Starknet class hash to contract class.
    /// Safe to use `Identity` as the key is already a hash.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn contract_class_by_class_hash)]
    pub(super) type ContractClasses<T: Config> = StorageMap<_, Identity, ClassHash, ContractClass, OptionQuery>;

    /// Mapping from Starknet Sierra class hash to  Casm compiled contract class.
    /// Safe to use `Identity` as the key is already a hash.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn compiled_class_hash_by_class_hash)]
    pub(super) type CompiledClassHashes<T: Config> = StorageMap<_, Identity, ClassHash, CompiledClassHash, OptionQuery>;

    /// Mapping from Starknet contract address to its nonce.
    /// Safe to use `Identity` as the key is already a hash.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn nonce)]
    pub(super) type Nonces<T: Config> = StorageMap<_, Identity, ContractAddress, Nonce, ValueQuery>;

    /// Mapping from Starknet contract storage key to its value.
    /// Safe to use `Identity` as the key is already a hash.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn storage)]
    pub(super) type StorageView<T: Config> = StorageMap<_, Identity, ContractStorageKey, StarkFelt, ValueQuery>;

    /// The last processed Ethereum block number for L1 messages consumption.
    /// This is used to avoid re-processing the same Ethereum block multiple times.
    /// This is used by the offchain worker.
    /// # TODO
    /// * Find a more relevant name for this.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn last_known_eth_block)]
    pub(super) type LastKnownEthBlock<T: Config> = StorageValue<_, u64>;

    /// The address of the fee token ERC20 contract.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn fee_token_address)]
    pub(super) type FeeTokenAddress<T: Config> = StorageValue<_, ContractAddress, ValueQuery>;

    /// Current sequencer address.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn sequencer_address)]
    pub type SequencerAddress<T: Config> = StorageValue<_, ContractAddress, ValueQuery>;

    /// Ensure the sequencer address was updated for this block.
    #[pallet::storage]
    #[pallet::unbounded]
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
        pub contracts: Vec<(ContractAddress, ClassHash)>,
        /// The contract classes to be deployed at genesis.
        /// This is a vector of tuples, where the first element is the contract class hash and the
        /// second element is the contract class definition.
        /// Same as `contracts`, this can be used to start the chain with a set of pre-deployed
        /// contracts classes.
        pub contract_classes: Vec<(ClassHash, ContractClass)>,
        pub storage: Vec<(ContractStorageKey, StarkFelt)>,
        /// The address of the fee token.
        /// Must be set to the address of the fee token ERC20 contract.
        pub fee_token_address: ContractAddress,
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
                fee_token_address: ContractAddress::default(),
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
        StarknetEvent(StarknetEvent),
        /// Emitted when fee token address is changed.
        /// This is emitted by the `set_fee_token_address` extrinsic.
        /// [old_fee_token_address, new_fee_token_address]
        FeeTokenAddressChanged {
            old_fee_token_address: ContractAddress,
            new_fee_token_address: ContractAddress,
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
        TooManyEmittedStarknetEvents,
        StateReaderError,
        EmitEventError,
        StateDiffError,
        ContractNotFound,
        TransactionConversionError,
        SequencerAddressNotValid,
        InvalidContractClassForThisDeclareVersion,
        Unimplemented,
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

            let addr = StarkFelt::new(addr).map_err(|_| Error::<T>::SequencerAddressNotValid)?;
            let addr = ContractAddress(addr.try_into().map_err(|_| Error::<T>::SequencerAddressNotValid)?);
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

            let input_transaction = transaction;

            let chain_id = Self::chain_id();
            let transaction = input_transaction.into_executable::<T::SystemHash>(chain_id, false);

            let sender_address = match &transaction.tx {
                starknet_api::transaction::InvokeTransaction::V0(tx) => tx.contract_address,
                starknet_api::transaction::InvokeTransaction::V1(tx) => tx.sender_address,
            };
            // Check if contract is deployed
            ensure!(ContractClassHashes::<T>::contains_key(sender_address), Error::<T>::AccountNotDeployed);

            // Execute
            let tx_execution_infos = transaction
                .execute(
                    &mut BlockifierStateAdapter::<T>::default(),
                    &Self::get_block_context(),
                    false,
                    T::DisableNonceValidation::get(),
                )
                .map_err(|e| {
                    log::error!("failed to execute invoke tx: {:?}", e);
                    Error::<T>::TransactionExecutionFailed
                })?;

            let tx_hash = transaction.tx_hash;
            Self::emit_and_store_tx_and_fees_events(
                tx_hash,
                tx_execution_infos.execute_call_info,
                tx_execution_infos.fee_transfer_call_info,
            );

            Self::store_transaction(tx_hash, Transaction::Invoke(input_transaction), tx_execution_infos.revert_error);

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
        pub fn declare(
            origin: OriginFor<T>,
            transaction: DeclareTransaction,
            contract_class: ContractClass,
        ) -> DispatchResult {
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;

            let input_transaction = transaction;
            let chain_id = Self::chain_id();
            let transaction = input_transaction
                .try_into_executable::<T::SystemHash>(chain_id, contract_class, false)
                .map_err(|_| Error::<T>::InvalidContractClassForThisDeclareVersion)?;

            // Check class hash is not already declared
            ensure!(
                !ContractClasses::<T>::contains_key(transaction.tx().class_hash()),
                Error::<T>::ClassHashAlreadyDeclared
            );
            // Check if contract is deployed
            ensure!(
                ContractClassHashes::<T>::contains_key(transaction.tx().sender_address()),
                Error::<T>::AccountNotDeployed
            );

            // Execute
            let tx_execution_infos = transaction
                .execute(
                    &mut BlockifierStateAdapter::<T>::default(),
                    &Self::get_block_context(),
                    false,
                    T::DisableNonceValidation::get(),
                )
                .map_err(|_| Error::<T>::TransactionExecutionFailed)?;

            let tx_hash = transaction.tx_hash();
            Self::emit_and_store_tx_and_fees_events(
                tx_hash,
                tx_execution_infos.execute_call_info,
                tx_execution_infos.fee_transfer_call_info,
            );

            Self::store_transaction(tx_hash, Transaction::Declare(input_transaction), tx_execution_infos.revert_error);

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

            let input_transaction = transaction;
            let chain_id = T::ChainId::get();
            let transaction = input_transaction.into_executable::<T::SystemHash>(chain_id, false);

            // Check if contract is deployed
            ensure!(
                !ContractClassHashes::<T>::contains_key(transaction.contract_address),
                Error::<T>::AccountAlreadyDeployed
            );

            // Execute
            let tx_execution_infos = transaction
                .execute(
                    &mut BlockifierStateAdapter::<T>::default(),
                    &Self::get_block_context(),
                    false,
                    T::DisableNonceValidation::get(),
                )
                .map_err(|e| {
                    log::error!("failed to deploy accout: {:?}", e);
                    Error::<T>::TransactionExecutionFailed
                })?;

            let tx_hash = transaction.tx_hash;
            Self::emit_and_store_tx_and_fees_events(
                tx_hash,
                tx_execution_infos.execute_call_info,
                tx_execution_infos.fee_transfer_call_info,
            );

            Self::store_transaction(
                tx_hash,
                Transaction::DeployAccount(input_transaction),
                tx_execution_infos.revert_error,
            );

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
        pub fn consume_l1_message(
            origin: OriginFor<T>,
            transaction: HandleL1MessageTransaction,
            paid_fee_on_l1: Fee,
        ) -> DispatchResult {
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;

            let input_transaction = transaction;
            let chain_id = Self::chain_id();
            let transaction = input_transaction.into_executable::<T::SystemHash>(chain_id, paid_fee_on_l1, false);

            // Execute
            let tx_execution_infos = transaction
                .execute(
                    &mut BlockifierStateAdapter::<T>::default(),
                    &Self::get_block_context(),
                    false,
                    T::DisableNonceValidation::get(),
                )
                .map_err(|_| Error::<T>::TransactionExecutionFailed)?;

            let tx_hash = transaction.tx_hash;
            Self::emit_and_store_tx_and_fees_events(
                tx_hash,
                tx_execution_infos.execute_call_info,
                tx_execution_infos.fee_transfer_call_info,
            );

            Self::store_transaction(
                tx_hash,
                Transaction::L1Handler(input_transaction),
                tx_execution_infos.revert_error,
            );

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

            let chain_id = Self::chain_id();
            let block_context = Self::get_block_context();
            let mut state: BlockifierStateAdapter<T> = BlockifierStateAdapter::<T>::default();
            let mut execution_resources = ExecutionResources::default();
            let mut initial_gas = blockifier::abi::constants::INITIAL_GAS_COST;

            let transaction = Self::get_call_transaction(call.clone()).map_err(|_| InvalidTransaction::Call)?;

            // Check the nonce is correct
            let (sender_address, sender_nonce, transaction_nonce) =
                if let UserAndL1HandlerTransaction::User(ref transaction) = transaction {
                    let sender_address: ContractAddress = transaction.sender_address().into();
                    let sender_nonce: Felt252Wrapper = Pallet::<T>::nonce(sender_address).into();
                    let transaction_nonce = transaction.nonce();

                    // Reject transaction with an already used Nonce
                    if sender_nonce > *transaction_nonce {
                        Err(InvalidTransaction::Stale)?;
                    }

                    // A transaction with a nonce higher than the expected nonce is placed in
                    // the future queue of the transaction pool.
                    if sender_nonce < *transaction_nonce {
                        log!(
                            info,
                            "Nonce is too high. Expected: {:?}, got: {:?}. This transaction will be placed in the \
                             transaction pool and executed in the future when the nonce is reached.",
                            sender_nonce,
                            transaction_nonce
                        );
                    }

                    (transaction.sender_address(), sender_nonce, *transaction_nonce)
                } else {
                    // TODO: create and check L1 messages Nonce
                    unimplemented!()
                };

            // Validate the user transactions
            if let UserAndL1HandlerTransaction::User(transaction) = transaction {
                match transaction {
                    UserTransaction::Declare(tx, contract_class) => tx
                        .try_into_executable::<T::SystemHash>(chain_id, contract_class, false)
                        .map_err(|_| InvalidTransaction::BadProof)?
                        .validate_tx(&mut state, &block_context, &mut execution_resources, &mut initial_gas, false),
                    // There is no way to validate it before the account is actuallly deployed
                    UserTransaction::DeployAccount(_) => Ok(None),
                    UserTransaction::Invoke(tx) => tx.into_executable::<T::SystemHash>(chain_id, false).validate_tx(
                        &mut state,
                        &block_context,
                        &mut execution_resources,
                        &mut initial_gas,
                        false,
                    ),
                }
                .map_err(|_| InvalidTransaction::BadProof)?;
            }

            let nonce_for_priority: u64 =
                transaction_nonce.try_into().map_err(|_| InvalidTransaction::Custom(NONCE_DECODE_FAILURE))?;

            let mut valid_transaction_builder = ValidTransaction::with_tag_prefix("starknet")
                .priority(u64::MAX - nonce_for_priority)
                .and_provides((sender_address, transaction_nonce))
                .longevity(T::TransactionLongevity::get())
                .propagate(true);

            // Enforce waiting for the tx with the previous nonce,
            // to be either executed or ordered before in the block
            if transaction_nonce > sender_nonce {
                valid_transaction_builder = valid_transaction_builder
                    .and_requires((sender_address, Felt252Wrapper(transaction_nonce.0 - FieldElement::ONE)));
            }

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
    fn get_call_transaction(call: Call<T>) -> Result<UserAndL1HandlerTransaction, ()> {
        let tx = match call {
            Call::<T>::invoke { transaction } => UserTransaction::Invoke(transaction).into(),
            Call::<T>::declare { transaction, contract_class } => {
                UserTransaction::Declare(transaction, contract_class).into()
            }
            Call::<T>::deploy_account { transaction } => UserTransaction::DeployAccount(transaction).into(),
            Call::<T>::consume_l1_message { transaction, paid_fee_on_l1 } => {
                UserAndL1HandlerTransaction::L1Handler(transaction, paid_fee_on_l1)
            }
            _ => return Err(()),
        };

        Ok(tx)
    }

    /// Creates a [BlockContext] object. The [BlockContext] is needed by the blockifier to execute
    /// properly the transaction. Substrate caches data so it's fine to call multiple times this
    /// function, only the first transaction/block will be "slow" to load these data.
    fn get_block_context() -> BlockContext {
        let block_number = UniqueSaturatedInto::<u64>::unique_saturated_into(frame_system::Pallet::<T>::block_number());
        let block_timestamp = Self::block_timestamp();

        let fee_token_address = Self::fee_token_address();
        let sequencer_address = Self::sequencer_address();

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
        TxEvents::<T>::iter_values().map(|v| v.len() as u128).sum()
    }

    /// Call a smart contract function.
    pub fn call_contract(
        address: ContractAddress,
        function_selector: EntryPointSelector,
        calldata: Calldata,
    ) -> Result<Vec<Felt252Wrapper>, DispatchError> {
        // Get current block context
        let block_context = Self::get_block_context();
        // Get class hash
        let class_hash = ContractClassHashes::<T>::try_get(address).map_err(|_| Error::<T>::ContractNotFound)?;

        let entrypoint = CallEntryPoint {
            class_hash: Some(class_hash),
            code_address: None,
            entry_point_type: EntryPointType::External,
            entry_point_selector: function_selector,
            calldata,
            storage_address: address,
            caller_address: ContractAddress::default(),
            call_type: CallType::Call,
            initial_gas: INITIAL_GAS,
        };

        let max_n_steps = block_context.invoke_tx_max_n_steps;
        let mut resources = ExecutionResources::default();
        let mut entry_point_execution_context =
            EntryPointExecutionContext::new(block_context, Default::default(), max_n_steps as usize);

        match entrypoint.execute(
            &mut BlockifierStateAdapter::<T>::default(),
            &mut resources,
            &mut entry_point_execution_context,
        ) {
            Ok(v) => {
                log!(debug, "Successfully called a smart contract function: {:?}", v);
                let result = v.execution.retdata.0.iter().map(|x| (*x).into()).collect();
                Ok(result)
            }
            Err(e) => {
                log!(error, "failed to call smart contract {:?}", e);
                Err(Error::<T>::TransactionExecutionFailed.into())
            }
        }
    }

    /// Get storage value at
    pub fn get_storage_at(contract_address: ContractAddress, key: StorageKey) -> Result<StarkFelt, DispatchError> {
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
        let transactions = Self::pending();
        let transaction_hashes = Self::pending_hashes();
        assert_eq!(
            transactions.len(),
            transaction_hashes.len(),
            "transactions and transaction hashes should be the same length"
        );
        let transaction_count = transactions.len();

        let parent_block_hash = Self::parent_block_hash(&block_number);
        let events: Vec<StarknetEvent> = transaction_hashes.iter().flat_map(TxEvents::<T>::take).collect();

        let global_state_root = Felt252Wrapper::default();

        let sequencer_address = Self::sequencer_address();
        let block_timestamp = Self::block_timestamp();

        let chain_id = Self::chain_id();
        let (transaction_commitment, event_commitment) =
            mp_commitments::calculate_commitments::<T::SystemHash>(&transactions, &events, chain_id);
        let protocol_version = T::ProtocolVersion::get();
        let extra_data = None;

        let block = StarknetBlock::new(
            StarknetHeader::new(
                parent_block_hash.into(),
                block_number,
                global_state_root.into(),
                sequencer_address,
                block_timestamp,
                transaction_count as u128,
                transaction_commitment.into(),
                events.len() as u128,
                event_commitment.into(),
                protocol_version,
                extra_data,
            ),
            transactions,
        );
        // Save the block number <> hash mapping.
        let blockhash = block.header().hash::<T::SystemHash>();
        BlockHash::<T>::insert(block_number, blockhash);

        // Kill pending storage.
        // There is no need to kill `TxEvents` as we used `take` while iterating over it.
        Pending::<T>::kill();
        PendingHashes::<T>::kill();

        let digest = DigestItem::Consensus(MADARA_ENGINE_ID, mp_digest_log::Log::Block(block).encode());
        frame_system::Pallet::<T>::deposit_log(digest);
    }

    /// Emit events from the call info.
    ///
    /// # Arguments
    ///
    /// * `call_info` — A ref to the call info structure.
    /// * `next_order` — Next expected event order, has to be 0 for a top level invocation
    ///
    /// # Returns
    ///
    /// Next expected event order
    #[inline(always)]
    fn emit_events_in_call_info(tx_hash: TransactionHash, call_info: &CallInfo, next_order: usize) -> usize {
        let mut event_idx = 0;
        let mut inner_call_idx = 0;
        let mut next_order = next_order;

        loop {
            // Emit current call's events as long as they have sequential orders
            if event_idx < call_info.execution.events.len() {
                let ordered_event = &call_info.execution.events[event_idx];
                if ordered_event.order == next_order {
                    let event = StarknetEvent {
                        from_address: call_info.call.storage_address,
                        content: ordered_event.event.clone(),
                    };
                    Self::deposit_event(Event::<T>::StarknetEvent(event.clone()));
                    TxEvents::<T>::append(tx_hash, event);
                    next_order += 1;
                    event_idx += 1;
                    continue;
                }
            }

            // Go deeper to find the continuation of the sequence
            if inner_call_idx < call_info.inner_calls.len() {
                next_order =
                    Self::emit_events_in_call_info(tx_hash, &call_info.inner_calls[inner_call_idx], next_order);
                inner_call_idx += 1;
                continue;
            }

            // At this point we have iterated over all sequential events and visited all internal calls
            break;
        }

        next_order
    }

    /// Estimate the fee associated with transaction
    pub fn estimate_fee(transaction: UserTransaction) -> Result<(u64, u64), DispatchError> {
        let chain_id = Self::chain_id();

        fn execute_tx_and_rollback<S: State + StateChanges + FeeConfig>(
            tx: impl Execute,
            state: &mut S,
            block_context: &BlockContext,
            disable_nonce_validation: bool,
        ) -> TransactionExecutionResult<TransactionExecutionInfo> {
            // TODO: initialization can probably be skiped by using mem::MaybeUninit
            let mut execution_result = Ok(Default::default());
            let _: Result<_, DispatchError> = storage::transactional::with_transaction(|| {
                execution_result = tx.execute(state, block_context, true, disable_nonce_validation);
                storage::TransactionOutcome::Rollback(Ok(()))
            });
            execution_result
        }

        let mut blockifier_state_adapter = BlockifierStateAdapter::<T>::default();
        let block_context = Self::get_block_context();
        let disable_nonce_validation = T::DisableNonceValidation::get();

        let execution_result = match transaction {
            UserTransaction::Declare(tx, contract_class) => execute_tx_and_rollback(
                tx.try_into_executable::<T::SystemHash>(chain_id, contract_class, true)
                    .map_err(|_| Error::<T>::InvalidContractClass)?,
                &mut blockifier_state_adapter,
                &block_context,
                disable_nonce_validation,
            ),
            UserTransaction::DeployAccount(tx) => execute_tx_and_rollback(
                tx.into_executable::<T::SystemHash>(chain_id, true),
                &mut blockifier_state_adapter,
                &block_context,
                disable_nonce_validation,
            ),
            UserTransaction::Invoke(tx) => execute_tx_and_rollback(
                tx.into_executable::<T::SystemHash>(chain_id, true),
                &mut blockifier_state_adapter,
                &block_context,
                disable_nonce_validation,
            ),
        };

        match execution_result {
            Ok(tx_exec_info) => {
                log!(debug, "Successfully estimated fee: {:?}", tx_exec_info);
                if let Some(gas_usage) = tx_exec_info.actual_resources.0.get("l1_gas_usage") {
                    Ok((tx_exec_info.actual_fee.0 as u64, *gas_usage as u64))
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

    pub fn emit_and_store_tx_and_fees_events(
        tx_hash: TransactionHash,
        execute_call_info: Option<CallInfo>,
        fee_transfer_call_info: Option<CallInfo>,
    ) {
        if let Some(call_info) = execute_call_info {
            let _ = Self::emit_events_in_call_info(tx_hash, &call_info, 0);
        }
        if let Some(call_info) = fee_transfer_call_info {
            let _ = Self::emit_events_in_call_info(tx_hash, &call_info, 0);
        }
    }

    fn store_transaction(tx_hash: TransactionHash, tx: Transaction, revert_reason: Option<String>) {
        Pending::<T>::append(tx);
        PendingHashes::<T>::append(tx_hash);
        TxRevertError::<T>::set(tx_hash, revert_reason);
    }

    pub fn chain_id() -> Felt252Wrapper {
        T::ChainId::get()
    }
}
