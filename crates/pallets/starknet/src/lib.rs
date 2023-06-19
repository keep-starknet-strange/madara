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
/// The implementation of the message type.
pub mod message;
/// The Starknet pallet's runtime API
pub mod runtime_api;
/// State root logic.
pub mod state_root;
/// Transaction validation logic.
pub mod transaction_validation;
/// The Starknet pallet's runtime custom types.
pub mod types;

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
use blockifier::execution::entry_point::{CallInfo, ExecutionResources};
use blockifier_state_adapter::BlockifierStateAdapter;
use frame_support::pallet_prelude::*;
use frame_support::traits::Time;
use frame_system::pallet_prelude::*;
use mp_digest_log::MADARA_ENGINE_ID;
use mp_starknet::block::{Block as StarknetBlock, Header as StarknetHeader, MaxTransactions};
use mp_starknet::crypto::commitment;
use mp_starknet::execution::types::{
    CallEntryPointWrapper, ClassHashWrapper, ContractAddressWrapper, ContractClassWrapper, EntryPointTypeWrapper,
    Felt252Wrapper,
};
use mp_starknet::storage::{StarknetStorageSchemaVersion, PALLET_STARKNET_SCHEMA};
use mp_starknet::traits::hash::{CryptoHasherT, DefaultHasher, HasherT};
use mp_starknet::transaction::types::{
    DeclareTransaction, DeployAccountTransaction, EventError, EventWrapper as StarknetEventType, InvokeTransaction,
    Transaction, TransactionExecutionInfoWrapper, TransactionReceiptWrapper, TxType,
};
use sp_runtime::traits::UniqueSaturatedInto;
use sp_runtime::DigestItem;
use starknet_api::api_core::{ChainId, ContractAddress};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::hash::StarkFelt;
use starknet_api::stdlib::collections::HashMap;
use starknet_api::transaction::{EventContent, TransactionHash};

use crate::alloc::string::ToString;
use crate::types::{ContractStorageKeyWrapper, NonceWrapper, StorageKeyWrapper};

pub(crate) const LOG_TARGET: &str = "runtime::starknet";

// TODO: don't use a const for this but a real sequencer address for block header
// FIXME https://github.com/keep-starknet-strange/madara/issues/243
pub const SEQUENCER_ADDRESS: [u8; 32] =
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 222, 173];

pub const ETHEREUM_EXECUTION_RPC: &[u8] = b"starknet::ETHEREUM_EXECUTION_RPC";
pub const ETHEREUM_CONSENSUS_RPC: &[u8] = b"starknet::ETHEREUM_CONSENSUS_RPC";

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

    use starknet_crypto::FieldElement;

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
        /// How Starknet state root is calculated.
        type StateRoot: Get<Felt252Wrapper>;
        /// The hashing function to use.
        type SystemHash: HasherT + DefaultHasher + CryptoHasherT;
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
        #[pallet::constant]
        type InvokeTxMaxNSteps: Get<u32>;
        #[pallet::constant]
        type ValidateMaxNSteps: Get<u32>;
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
    pub(super) type ContractClasses<T: Config> =
        StorageMap<_, Identity, ClassHashWrapper, ContractClassWrapper, OptionQuery>;

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

    /// The chain id.
    #[pallet::storage]
    #[pallet::getter(fn chain_id)]
    pub(super) type ChainId<T: Config> = StorageValue<_, Felt252Wrapper, ValueQuery>;

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
        pub storage: Vec<(ContractStorageKeyWrapper, Felt252Wrapper)>,
        /// The address of the fee token.
        /// Must be set to the address of the fee token ERC20 contract.
        pub fee_token_address: ContractAddressWrapper,
        pub _phantom: PhantomData<T>,
        /// The chain id.
        pub chain_id: Felt252Wrapper,
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
                chain_id: Default::default(),
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
            // Set the chain id from the genesis config.
            ChainId::<T>::put(self.chain_id);
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
    }

    /// The Starknet pallet external functions.
    /// Dispatchable functions allows users to interact with the pallet and invoke state changes.
    /// These functions materialize as "extrinsics", which are often compared to transactions.
    /// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
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
        #[pallet::call_index(0)]
        #[pallet::weight({0})]
        pub fn invoke(origin: OriginFor<T>, transaction: InvokeTransaction) -> DispatchResult {
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;
            // Check if contract is deployed
            ensure!(ContractClassHashes::<T>::contains_key(transaction.sender_address), Error::<T>::AccountNotDeployed);

            // Get current block context
            let block_context = Self::get_block_context();
            let chain_id = Self::chain_id();
            let transaction: Transaction = transaction.from_invoke(chain_id);
            let call_info =
                transaction.execute(&mut BlockifierStateAdapter::<T>::default(), &block_context, TxType::Invoke, None);
            let receipt = match call_info {
                Ok(TransactionExecutionInfoWrapper {
                    validate_call_info: _validate_call_info,
                    execute_call_info,
                    fee_transfer_call_info,
                    actual_fee,
                    actual_resources: _actual_resources,
                }) => {
                    log!(debug, "Transaction executed successfully: {:?}", execute_call_info);

                    let events = Self::emit_events_for_calls(
                        TransactionHash(transaction.hash.into()),
                        execute_call_info,
                        fee_transfer_call_info,
                    )?;

                    TransactionReceiptWrapper {
                        events: BoundedVec::try_from(events).map_err(|_| Error::<T>::ReachedBoundedVecLimit)?,
                        transaction_hash: transaction.hash,
                        tx_type: TxType::Invoke,
                        actual_fee: actual_fee.0.into(),
                    }
                }
                Err(e) => {
                    log!(error, "Transaction execution failed: {:?}", e);
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
        #[pallet::call_index(1)]
        #[pallet::weight({0})]
        pub fn declare(origin: OriginFor<T>, transaction: DeclareTransaction) -> DispatchResult {
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;

            let chain_id = Self::chain_id();

            let transaction: Transaction = transaction.from_declare(chain_id);
            // Check that contract class is not None
            let contract_class = transaction.contract_class.clone().ok_or(Error::<T>::ContractClassMustBeSpecified)?;

            // Check that the class hash is not None
            let class_hash = transaction.call_entrypoint.class_hash.ok_or(Error::<T>::ClassHashMustBeSpecified)?;

            // Check if contract is deployed
            ensure!(ContractClassHashes::<T>::contains_key(transaction.sender_address), Error::<T>::AccountNotDeployed);

            // Check class hash is not already declared
            ensure!(!ContractClasses::<T>::contains_key(class_hash), Error::<T>::ClassHashAlreadyDeclared);

            // Get current block context
            let block_context = Self::get_block_context();

            // Parse contract class
            let contract_class = contract_class.try_into().or(Err(Error::<T>::InvalidContractClass))?;

            // Execute transaction
            let call_info = transaction.execute(
                &mut BlockifierStateAdapter::<T>::default(),
                &block_context,
                TxType::Declare,
                Some(contract_class),
            );
            let receipt = match call_info {
                Ok(TransactionExecutionInfoWrapper {
                    validate_call_info: _validate_call_info,
                    execute_call_info,
                    fee_transfer_call_info,
                    actual_fee,
                    actual_resources: _actual_resources,
                }) => {
                    log!(trace, "Transaction executed successfully: {:?}", execute_call_info);

                    let events = Self::emit_events_for_calls(
                        TransactionHash(transaction.hash.into()),
                        execute_call_info,
                        fee_transfer_call_info,
                    )?;

                    TransactionReceiptWrapper {
                        events: BoundedVec::try_from(events).map_err(|_| Error::<T>::ReachedBoundedVecLimit)?,
                        transaction_hash: transaction.hash,
                        tx_type: TxType::Declare,
                        actual_fee: actual_fee.0.into(),
                    }
                }
                Err(e) => {
                    log!(error, "Transaction execution failed: {:?}", e);
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
        #[pallet::call_index(2)]
        #[pallet::weight({0})]
        pub fn deploy_account(origin: OriginFor<T>, transaction: DeployAccountTransaction) -> DispatchResult {
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;

            let chain_id = Self::chain_id();
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
                None,
            );
            let receipt = match call_info {
                Ok(TransactionExecutionInfoWrapper {
                    validate_call_info: _validate_call_info,
                    execute_call_info,
                    fee_transfer_call_info,
                    actual_fee,
                    actual_resources: _actual_resources,
                }) => {
                    log!(trace, "Transaction executed successfully: {:?}", execute_call_info);

                    let events = Self::emit_events_for_calls(
                        TransactionHash(transaction.hash.into()),
                        execute_call_info,
                        fee_transfer_call_info,
                    )?;

                    TransactionReceiptWrapper {
                        events: BoundedVec::try_from(events).map_err(|_| Error::<T>::ReachedBoundedVecLimit)?,
                        transaction_hash: transaction.hash,
                        tx_type: TxType::DeployAccount,
                        actual_fee: actual_fee.0.into(),
                    }
                }
                Err(e) => {
                    log!(error, "Transaction execution failed: {:?}", e);
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
        #[pallet::call_index(3)]
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
                None,
            ) {
                Ok(v) => {
                    log!(debug, "Transaction executed successfully: {:?}", v);
                }
                Err(e) => {
                    log!(error, "Transaction execution failed: {:?}", e);
                    return Err(Error::<T>::TransactionExecutionFailed.into());
                }
            }

            // Append the transaction to the pending transactions.
            Pending::<T>::try_append((transaction.clone(), TransactionReceiptWrapper::default()))
                .or(Err(Error::<T>::TooManyPendingTransactions))?;

            Ok(())
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

            let mut valid_transaction_builder = ValidTransaction::with_tag_prefix("starknet")
                .priority(u64::MAX - (TryInto::<u64>::try_into(transaction_nonce)).unwrap())
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
            Call::<T>::invoke { transaction } => Ok(transaction.from_invoke(Self::chain_id())),
            Call::<T>::declare { transaction } => Ok(transaction.from_declare(Self::chain_id())),
            Call::<T>::deploy_account { transaction } => transaction.from_deploy(Self::chain_id()).map_err(|_| ()),
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
        transaction
            .validate_account_tx(&mut state, &mut execution_resources, &block_context, &tx_type)
            .map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::BadProof))?;

        Ok(())
    }

    /// Creates a [BlockContext] object. The [BlockContext] is needed by the blockifier to execute
    /// properly the transaction. Substrate caches data so it's fine to call multiple times this
    /// function, only the first transaction/block will be "slow" to load these data.
    fn get_block_context() -> BlockContext {
        let block_number = UniqueSaturatedInto::<u64>::unique_saturated_into(frame_system::Pallet::<T>::block_number());
        let block_timestamp = Self::block_timestamp();
        // Get fee token address. Its value is checked when we set it so it's fine to unwrap
        let fee_token_address =
            ContractAddress::try_from(StarkFelt::new(Self::fee_token_address().into()).unwrap()).unwrap();
        let chain_id = Self::chain_id_str();
        let sequencer_address = ContractAddress(starknet_api::api_core::PatriciaKey(StarkFelt(SEQUENCER_ADDRESS)));
        let vm_resource_fee_cost = HashMap::default();
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
        }
    }

    /// convert chain_id
    #[inline(always)]
    pub fn chain_id_str() -> String {
        unsafe { from_utf8_unchecked(&Self::chain_id().0.to_bytes_be()).to_string() }
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
        );

        match entrypoint.execute(&mut BlockifierStateAdapter::<T>::default(), block_context) {
            Ok(v) => {
                log!(debug, "Transaction executed successfully: {:?}", v);
                let result = v.execution.retdata.0.iter().map(|x| (*x).into()).collect();
                Ok(result)
            }
            Err(e) => {
                log!(error, "Transaction execution failed: {:?}", e);
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
        // TODO: Use actual values.
        let parent_block_hash = Self::parent_block_hash(&block_number);
        let pending = Self::pending();

        let global_state_root = Felt252Wrapper::ZERO;
        // TODO: use the real sequencer address (our own address)
        // FIXME #243
        let sequencer_address = SEQUENCER_ADDRESS;
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
        let protocol_version = None;
        let extra_data = None;

        let block = StarknetBlock::new(
            StarknetHeader::new(
                parent_block_hash,
                block_number,
                global_state_root,
                Felt252Wrapper::try_from(&sequencer_address).unwrap(),
                block_timestamp,
                transaction_count,
                transaction_commitment.try_into().unwrap(),
                events.len() as u128,
                event_commitment.try_into().unwrap(),
                protocol_version,
                extra_data,
            ),
            // Safe because `transactions` is build from the `pending` bounded vec,
            // which has the same size limit of `MaxTransactions`
            BoundedVec::try_from(transactions).unwrap(),
            BoundedVec::try_from(receipts).unwrap(),
        );
        // Save the block number <> hash mapping.
        let blockhash = block.header().hash(T::SystemHash::hasher());
        BlockHash::<T>::insert(block_number, blockhash);
        Pending::<T>::kill();
        PendingEvents::<T>::kill();

        let digest = DigestItem::Consensus(MADARA_ENGINE_ID, mp_digest_log::Log::Block(block).encode());
        frame_system::Pallet::<T>::deposit_log(digest);
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
    fn emit_events(call_info: &mut CallInfo, tx_hash: TransactionHash) -> Result<Vec<StarknetEventType>, EventError> {
        let mut events = Vec::new();

        call_info.execution.events.sort_by_key(|ordered_event| ordered_event.order);
        for ordered_event in &call_info.execution.events {
            let event_type = Self::emit_event(&ordered_event.event, call_info.call.storage_address, tx_hash)?;
            events.push(event_type);
        }

        for inner_call in &mut call_info.inner_calls {
            inner_call.execution.events.sort_by_key(|ordered_event| ordered_event.order);
            for ordered_event in &inner_call.execution.events {
                let event_type = Self::emit_event(&ordered_event.event, inner_call.call.storage_address, tx_hash)?;
                events.push(event_type);
            }
        }

        Ok(events)
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
        tx_hash: TransactionHash,
    ) -> Result<StarknetEventType, EventError> {
        log!(debug, "Transaction event: {:?}", event);
        let sn_event = StarknetEventType::builder()
            .with_event_content(event.clone())
            .with_from_address(from_address)
            .with_transaction_hash(tx_hash)
            .build()?;
        Self::deposit_event(Event::StarknetEvent(sn_event.clone()));

        PendingEvents::<T>::try_append(sn_event.clone()).map_err(|_| EventError::TooManyEvents)?;
        Ok(sn_event)
    }

    /// Estimate the fee associated with transaction
    pub fn estimate_fee(transaction: Transaction) -> Result<(u64, u64), DispatchError> {
        // Check if contract is deployed
        ensure!(ContractClassHashes::<T>::contains_key(transaction.sender_address), Error::<T>::AccountNotDeployed);

        match transaction.execute(
            &mut BlockifierStateAdapter::<T>::default(),
            &Self::get_block_context(),
            TxType::Invoke,
            None,
        ) {
            Ok(v) => {
                log!(debug, "Transaction executed successfully: {:?}", v);
                if let Some(gas_usage) = v.actual_resources.get("l1_gas_usage") {
                    Ok((v.actual_fee.0 as u64, *gas_usage as u64))
                } else {
                    Err(Error::<T>::TransactionExecutionFailed.into())
                }
            }
            Err(e) => {
                log!(error, "Transaction execution failed: {:?}", e);
                Err(Error::<T>::TransactionExecutionFailed.into())
            }
        }
    }

    /// Returns the hasher used by the runtime.
    pub fn get_system_hash() -> T::SystemHash {
        T::SystemHash::hasher()
    }

    pub fn emit_events_for_calls(
        tx_hash: TransactionHash,
        execute_call_info: Option<CallInfo>,
        fee_transfer_call_info: Option<CallInfo>,
    ) -> Result<Vec<StarknetEventType>, Error<T>> {
        let events = match (execute_call_info, fee_transfer_call_info) {
            (Some(mut exec), Some(mut fee)) => {
                let mut events = Self::emit_events(&mut exec, tx_hash).map_err(|_| Error::<T>::EmitEventError)?;
                events.append(&mut Self::emit_events(&mut fee, tx_hash).map_err(|_| Error::<T>::EmitEventError)?);
                events
            }
            (_, Some(mut fee)) => Self::emit_events(&mut fee, tx_hash).map_err(|_| Error::<T>::EmitEventError)?,
            _ => Vec::default(),
        };
        Ok(events)
    }
}
