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
#![allow(clippy::large_enum_variant)]

/// Starknet pallet.
/// Definition of the pallet's runtime storage items, events, errors, and dispatchable
/// functions.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
/// An adapter for the blockifier state related traits
pub mod blockifier_state_adapter;
/// The implementation of the execution configuration.
pub mod execution_config;
#[cfg(feature = "std")]
pub mod genesis_loader;
/// Simulation, estimations and execution trace logic.
pub mod simulations;
/// Transaction validation logic.
pub mod transaction_validation;
/// The Starknet pallet's runtime custom types.
pub mod types;

#[cfg(test)]
mod tests;

#[macro_use]
pub extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::str::from_utf8_unchecked;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use blockifier::block_context::BlockContext;
use blockifier::execution::contract_class::ContractClass;
use blockifier::execution::entry_point::{
    CallEntryPoint, CallInfo, CallType, EntryPointExecutionContext, ExecutionResources,
};
use blockifier::execution::errors::{EntryPointExecutionError, PreExecutionError};
use blockifier::state::cached_state::ContractStorageKey;
use blockifier_state_adapter::BlockifierStateAdapter;
use frame_support::pallet_prelude::*;
use frame_support::traits::Time;
use frame_system::pallet_prelude::*;
use mp_block::{Block as StarknetBlock, Header as StarknetHeader};
use mp_digest_log::MADARA_ENGINE_ID;
use mp_fee::{ResourcePrice, INITIAL_GAS};
use mp_felt::Felt252Wrapper;
use mp_hashers::HasherT;
use mp_sequencer_address::{InherentError, InherentType, DEFAULT_SEQUENCER_ADDRESS, INHERENT_IDENTIFIER};
use mp_storage::{StarknetStorageSchemaVersion, PALLET_STARKNET_SCHEMA};
use mp_transactions::execution::Execute;
use mp_transactions::{
    DeclareTransaction, DeployAccountTransaction, HandleL1MessageTransaction, InvokeTransaction, Transaction,
    UserOrL1HandlerTransaction, UserTransaction,
};
use sp_runtime::traits::UniqueSaturatedInto;
use sp_runtime::DigestItem;
use starknet_api::api_core::{ChainId, CompiledClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::state::StorageKey;
use starknet_api::transaction::{Calldata, Event as StarknetEvent, Fee, MessageToL1, TransactionHash};
use starknet_crypto::FieldElement;
use transaction_validation::TxPriorityInfo;

use crate::alloc::string::ToString;
use crate::execution_config::RuntimeExecutionConfigBuilder;
use crate::types::{CasmClassHash, SierraClassHash, StorageSlot};

pub(crate) const LOG_TARGET: &str = "runtime::starknet";

pub const ETHEREUM_EXECUTION_RPC: &[u8] = b"starknet::ETHEREUM_EXECUTION_RPC";
pub const ETHEREUM_CONSENSUS_RPC: &[u8] = b"starknet::ETHEREUM_CONSENSUS_RPC";

pub const SN_OS_CONFIG_HASH_VERSION: &str = "StarknetOsConfig1";

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
        /// The block time
        type TimestampProvider: Time;
        /// The gas price
        #[pallet::constant]
        type L1GasPrice: Get<ResourcePrice>;
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
        #[pallet::constant]
        type ProgramHash: Get<Felt252Wrapper>;
    }

    /// The Starknet pallet hooks.
    /// HOOKS
    /// # TODO
    /// * Implement the hooks.
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// The block is being finalized.
        fn on_finalize(_n: BlockNumberFor<T>) {
            assert!(SeqAddrUpdate::<T>::take(), "Sequencer address must be set for the block");

            // Create a new Starknet block and store it.
            <Pallet<T>>::store_block(UniqueSaturatedInto::<u64>::unique_saturated_into(
                frame_system::Pallet::<T>::block_number(),
            ));
        }

        /// The block is being initialized. Implement to have something happen.
        fn on_initialize(_: BlockNumberFor<T>) -> Weight {
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
    #[pallet::getter(fn tx_messages)]
    pub(super) type TxMessages<T: Config> = StorageMap<_, Identity, TransactionHash, Vec<MessageToL1>, ValueQuery>;

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
    pub(super) type ContractClassHashes<T: Config> =
        StorageMap<_, Identity, ContractAddress, CasmClassHash, ValueQuery>;

    /// Mapping from Starknet class hash to contract class.
    /// Safe to use `Identity` as the key is already a hash.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn contract_class_by_class_hash)]
    pub(super) type ContractClasses<T: Config> = StorageMap<_, Identity, CasmClassHash, ContractClass, OptionQuery>;

    /// Mapping from Starknet Sierra class hash to  Casm compiled contract class.
    /// Safe to use `Identity` as the key is already a hash.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn compiled_class_hash_by_class_hash)]
    pub(super) type CompiledClassHashes<T: Config> =
        StorageMap<_, Identity, SierraClassHash, CompiledClassHash, OptionQuery>;

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

    /// Information about processed L1 Messages
    /// Based on Nonce value.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn l1_messages)]
    pub(super) type L1Messages<T: Config> = StorageValue<_, BTreeSet<Nonce>, ValueQuery>;

    /// Starknet genesis configuration.
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// The contracts to be deployed at genesis.
        /// This is a vector of tuples, where the first element is the contract address and the
        /// second element is the contract class hash.
        /// This can be used to start the chain with a set of pre-deployed contracts, for example in
        /// a test environment or in the case of a migration of an existing chain state.
        pub contracts: Vec<(ContractAddress, SierraClassHash)>,
        pub sierra_to_casm_class_hash: Vec<(SierraClassHash, CasmClassHash)>,
        /// The contract classes to be deployed at genesis.
        /// This is a vector of tuples, where the first element is the contract class hash and the
        /// second element is the contract class definition.
        /// Same as `contracts`, this can be used to start the chain with a set of pre-deployed
        /// contracts classes.
        pub contract_classes: Vec<(SierraClassHash, ContractClass)>,
        pub storage: Vec<(ContractStorageKey, StarkFelt)>,
        /// The address of the fee token.
        /// Must be set to the address of the fee token ERC20 contract.
        pub fee_token_address: ContractAddress,
        pub _phantom: PhantomData<T>,
    }

    /// `Default` impl required by `pallet::GenesisBuild`.
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                contracts: vec![],
                sierra_to_casm_class_hash: vec![],
                contract_classes: vec![],
                storage: vec![],
                fee_token_address: ContractAddress::default(),
                _phantom: PhantomData,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            <Pallet<T>>::store_block(0);
            frame_support::storage::unhashed::put::<StarknetStorageSchemaVersion>(
                PALLET_STARKNET_SCHEMA,
                &StarknetStorageSchemaVersion::V1,
            );

            for (class_hash, contract_class) in self.contract_classes.iter() {
                ContractClasses::<T>::insert(class_hash, contract_class);
            }

            for (sierra_class_hash, casm_class_hash) in self.sierra_to_casm_class_hash.iter() {
                CompiledClassHashes::<T>::insert(sierra_class_hash, CompiledClassHash(casm_class_hash.0));
            }

            for (address, class_hash) in self.contracts.iter() {
                ContractClassHashes::<T>::insert(address, class_hash);
            }

            for (key, value) in self.storage.iter() {
                StorageView::<T>::insert(key, value);
            }

            LastKnownEthBlock::<T>::set(None);
            // Set the fee token address from the genesis config.
            FeeTokenAddress::<T>::set(self.fee_token_address);
            SeqAddrUpdate::<T>::put(true);
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
        MissingRevertReason,
        MissingCallInfo,
        FailedToCreateATransactionalStorageExecution,
        L1MessageAlreadyExecuted,
        MissingL1GasUsage,
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
                    &RuntimeExecutionConfigBuilder::new::<T>().build(),
                )
                .map_err(|e| {
                    log::error!("failed to execute invoke tx: {:?}", e);
                    Error::<T>::TransactionExecutionFailed
                })?;

            let tx_hash = transaction.tx_hash;
            Self::emit_and_store_tx_and_fees_events(
                tx_hash,
                &tx_execution_infos.execute_call_info,
                &tx_execution_infos.fee_transfer_call_info,
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
                    &RuntimeExecutionConfigBuilder::new::<T>().build(),
                )
                .map_err(|_| Error::<T>::TransactionExecutionFailed)?;

            let tx_hash = transaction.tx_hash();
            Self::emit_and_store_tx_and_fees_events(
                tx_hash,
                &tx_execution_infos.execute_call_info,
                &tx_execution_infos.fee_transfer_call_info,
            );
            Self::store_transaction(tx_hash, Transaction::Declare(input_transaction), tx_execution_infos.revert_error);

            Ok(())
        }

        /// Since Starknet v0.10.1 the deploy_account transaction replaces the deploy transaction
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
                    &RuntimeExecutionConfigBuilder::new::<T>().build(),
                )
                .map_err(|e| {
                    log::error!("failed to deploy account: {:?}", e);
                    Error::<T>::TransactionExecutionFailed
                })?;

            let tx_hash = transaction.tx_hash;
            Self::emit_and_store_tx_and_fees_events(
                tx_hash,
                &tx_execution_infos.execute_call_info,
                &tx_execution_infos.fee_transfer_call_info,
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

            let nonce: Nonce = transaction.tx.nonce;

            // Ensure that L1 Message has not been executed
            Self::ensure_l1_message_not_executed(&nonce).map_err(|_| Error::<T>::L1MessageAlreadyExecuted)?;

            // Store infornamtion about message being processed
            // The next instruction executes the message
            // Either successfully  or not
            L1Messages::<T>::mutate(|nonces| nonces.insert(nonce));

            // Execute
            let tx_execution_infos = transaction
                .execute(
                    &mut BlockifierStateAdapter::<T>::default(),
                    &Self::get_block_context(),
                    &RuntimeExecutionConfigBuilder::new::<T>().build(),
                )
                .map_err(|e| {
                    log::error!("Failed to consume l1 message: {}", e);
                    Error::<T>::TransactionExecutionFailed
                })?;

            let tx_hash = transaction.tx_hash;
            Self::emit_and_store_tx_and_fees_events(
                tx_hash,
                &tx_execution_infos.execute_call_info,
                &tx_execution_infos.fee_transfer_call_info,
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

            let tx_priority_info = Self::validate_unsigned_tx_nonce(&transaction)?;

            Self::validate_unsigned_tx(&transaction)?;

            let mut valid_transaction_builder = ValidTransaction::with_tag_prefix("starknet")
                .priority(u64::MAX)
                .longevity(T::TransactionLongevity::get())
                .propagate(true);

            match tx_priority_info {
                // Make sure txs from same account are executed in correct order (nonce based ordering)
                TxPriorityInfo::RegularTxs { sender_address, transaction_nonce, sender_nonce } => {
                    valid_transaction_builder =
                        valid_transaction_builder.and_provides((sender_address, Felt252Wrapper(transaction_nonce.0)));
                    if transaction_nonce > sender_nonce {
                        valid_transaction_builder = valid_transaction_builder
                            .and_requires((sender_address, Felt252Wrapper(transaction_nonce.0 - FieldElement::ONE)));
                    }
                }
                TxPriorityInfo::L1Handler { nonce } => {
                    valid_transaction_builder =
                        valid_transaction_builder.and_provides((Felt252Wrapper::ZERO, Felt252Wrapper(nonce.0)));
                }
                _ => {}
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
    fn get_call_transaction(call: Call<T>) -> Result<UserOrL1HandlerTransaction, ()> {
        let tx = match call {
            Call::<T>::invoke { transaction } => UserTransaction::Invoke(transaction).into(),
            Call::<T>::declare { transaction, contract_class } => {
                UserTransaction::Declare(transaction, contract_class).into()
            }
            Call::<T>::deploy_account { transaction } => UserTransaction::DeployAccount(transaction).into(),
            Call::<T>::consume_l1_message { transaction, paid_fee_on_l1 } => {
                UserOrL1HandlerTransaction::L1Handler(transaction, paid_fee_on_l1)
            }
            _ => return Err(()),
        };

        Ok(tx)
    }

    /// Creates a [BlockContext] object. The [BlockContext] is needed by the blockifier to execute
    /// properly the transaction. Substrate caches data so it's fine to call multiple times this
    /// function, only the first transaction/block will be "slow" to load these data.
    pub fn get_block_context() -> BlockContext {
        let block_number = UniqueSaturatedInto::<u64>::unique_saturated_into(frame_system::Pallet::<T>::block_number());
        let block_timestamp = Self::block_timestamp();

        let fee_token_address = Self::fee_token_address();
        let sequencer_address = Self::sequencer_address();

        let chain_id = Self::chain_id_str();

        let vm_resource_fee_cost = Default::default();
        BlockContext {
            block_number: BlockNumber(block_number),
            block_timestamp: BlockTimestamp(block_timestamp),
            chain_id: ChainId(chain_id),
            sequencer_address,
            fee_token_address,
            vm_resource_fee_cost,
            invoke_tx_max_n_steps: T::InvokeTxMaxNSteps::get(),
            validate_max_n_steps: T::ValidateMaxNSteps::get(),
            gas_price: T::L1GasPrice::get().price_in_wei,
            max_recursion_depth: T::MaxRecursionDepth::get(),
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
            EntryPointExecutionContext::new(block_context, Default::default(), max_n_steps);

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

        let sequencer_address = Self::sequencer_address();
        let block_timestamp = Self::block_timestamp();

        let protocol_version = T::ProtocolVersion::get();
        let extra_data = None;

        let l1_gas_price = T::L1GasPrice::get();

        let block = StarknetBlock::new(
            StarknetHeader::new(
                parent_block_hash.into(),
                block_number,
                sequencer_address,
                block_timestamp,
                transaction_count as u128,
                events.len() as u128,
                protocol_version,
                l1_gas_price,
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

    /// Aggregate L2 > L1 messages from the call info.
    ///
    /// # Arguments
    ///
    /// * `tx_hash` - The hash of the transaction being processed
    /// * `call_info` — A ref to the call info structure.
    /// * `next_order` — Next expected message order, has to be 0 for a top level invocation
    ///
    /// # Returns
    ///
    /// Next expected message order
    fn aggregate_messages_in_call_info(tx_hash: TransactionHash, call_info: &CallInfo, next_order: usize) -> usize {
        let mut message_idx = 0;
        let mut inner_call_idx = 0;
        let mut next_order = next_order;

        loop {
            // Store current call's messages as long as they have sequential orders
            if message_idx < call_info.execution.l2_to_l1_messages.len() {
                let ordered_message = &call_info.execution.l2_to_l1_messages[message_idx];
                if ordered_message.order == next_order {
                    let message = MessageToL1 {
                        from_address: call_info.call.storage_address,
                        to_address: ordered_message.message.to_address,
                        payload: ordered_message.message.payload.clone(),
                    };
                    TxMessages::<T>::append(tx_hash, message);
                    next_order += 1;
                    message_idx += 1;
                    continue;
                }
            }

            // Go deeper to find the continuation of the sequence
            if inner_call_idx < call_info.inner_calls.len() {
                next_order =
                    Self::aggregate_messages_in_call_info(tx_hash, &call_info.inner_calls[inner_call_idx], next_order);
                inner_call_idx += 1;
                continue;
            }

            // At this point we have iterated over all sequential messages and visited all internal calls
            break;
        }

        next_order
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

    pub fn emit_and_store_tx_and_fees_events(
        tx_hash: TransactionHash,
        execute_call_info: &Option<CallInfo>,
        fee_transfer_call_info: &Option<CallInfo>,
    ) {
        if let Some(call_info) = execute_call_info {
            Self::emit_events_in_call_info(tx_hash, call_info, 0);
            Self::aggregate_messages_in_call_info(tx_hash, call_info, 0);
        }
        if let Some(call_info) = fee_transfer_call_info {
            Self::emit_events_in_call_info(tx_hash, call_info, 0);
            Self::aggregate_messages_in_call_info(tx_hash, call_info, 0);
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

    pub fn program_hash() -> Felt252Wrapper {
        T::ProgramHash::get()
    }

    pub fn config_hash() -> StarkHash {
        T::SystemHash::compute_hash_on_elements(&[
            FieldElement::from_byte_slice_be(SN_OS_CONFIG_HASH_VERSION.as_bytes()).unwrap(),
            T::ChainId::get().into(),
            Self::fee_token_address().0.0.into(),
        ])
        .into()
    }

    pub fn is_transaction_fee_disabled() -> bool {
        T::DisableTransactionFee::get()
    }
}
