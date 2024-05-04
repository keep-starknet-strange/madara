//! L2 validity rollup, settling on Ethereum or as a L3 application-specific rollup, settling on
//! public Starknet L2.
//! For now this is the same because we don't support yet validity proofs and state updates to
//! another layer.
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
// include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
pub const WASM_BINARY: Option<&[u8]> = Some(&[]);

/// Runtime modules.
mod config;
pub mod opaque;
mod pallets;
mod runtime_tests;
mod types;

use blockifier::context::FeeTokenAddresses;
use blockifier::execution::contract_class::ContractClass;
use blockifier::state::cached_state::CommitmentStateDiff;
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::objects::TransactionExecutionInfo;
use blockifier::transaction::transaction_execution::Transaction;
use blockifier::transaction::transactions::L1HandlerTransaction;
pub use config::*;
pub use frame_support::traits::{ConstU128, ConstU32, ConstU64, ConstU8, KeyOwnerProofSystem, Randomness, StorageInfo};
pub use frame_support::weights::constants::{
    BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
};
pub use frame_support::weights::{IdentityFee, Weight};
pub use frame_support::{construct_runtime, parameter_types, StorageValue};
pub use frame_system::Call as SystemCall;
use mp_felt::Felt252Wrapper;
use mp_simulations::{InternalSubstrateError, SimulationError, SimulationFlags, TransactionSimulationResult};
use pallet_grandpa::{fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
/// Import the Starknet pallet.
pub use pallet_starknet;
use pallet_starknet::Call::{consume_l1_message, declare, deploy_account, invoke};
pub use pallet_starknet::DefaultChainId;
pub use pallet_timestamp::Call as TimestampCall;
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::crypto::KeyTypeId;
use sp_core::OpaqueMetadata;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT, NumberFor};
use sp_runtime::transaction_validity::{TransactionSource, TransactionValidity};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
use sp_runtime::{generic, ApplyExtrinsicResult};
pub use sp_runtime::{Perbill, Permill};
use sp_std::prelude::*;
use sp_version::RuntimeVersion;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::state::StorageKey;
use starknet_api::transaction::{Calldata, Event as StarknetEvent, MessageToL1, TransactionHash};
/// Import the types.
pub use types::*;
// For `format!`
extern crate alloc;

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
    pub struct Runtime {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Aura: pallet_aura,
        Grandpa: pallet_grandpa,
        // Include Starknet pallet.
        Starknet: pallet_starknet,
    }
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive =
    frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPalletsWithSystem>;

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
    define_benchmarks!(
        [frame_benchmarking, BaselineBench::<Runtime>]
        [frame_system, SystemBench::<Runtime>]
        [pallet_balances, Balances]
        [pallet_timestamp, Timestamp]
    );
}

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block);
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }


    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }

        fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
            Runtime::metadata_at_version(version)
        }

        fn metadata_versions() -> sp_std::vec::Vec<u32> {
            Runtime::metadata_versions()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }

        fn authorities() -> Vec<AuraId> {
            Aura::authorities().into_inner()
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            opaque::SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn current_set_id() -> fg_primitives::SetId {
            Grandpa::current_set_id()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            _key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            None
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            _authority_id: GrandpaId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            // NOTE: this is the only implementation possible since we've
            // defined our key owner proof type as a bottom type (i.e. a type
            // with no values).
            None
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_starknet_runtime_api::StarknetRuntimeApi<Block> for Runtime {

        fn get_storage_at(address: ContractAddress, key: StorageKey) -> Result<StarkFelt, SimulationError> {
            Starknet::get_storage_at(address, key)
        }

        fn call(address: ContractAddress, function_selector: EntryPointSelector, calldata: Calldata) -> Result<Vec<Felt252Wrapper>, SimulationError> {
            Starknet::call_contract(address, function_selector, calldata)
        }

        fn nonce(address: ContractAddress) -> Nonce{
            Starknet::nonce(address)
        }

        fn contract_class_hash_by_address(address: ContractAddress) -> ClassHash {
            ClassHash(Starknet::contract_class_hash_by_address(address))
        }

        fn contract_class_by_class_hash(class_hash: ClassHash) -> Option<ContractClass> {
            Starknet::contract_class_by_class_hash(class_hash.0)
        }

        fn chain_id() -> Felt252Wrapper {
            Starknet::chain_id()
        }

        fn program_hash() -> Felt252Wrapper {
            Starknet::program_hash()
        }

        fn config_hash() -> StarkHash {
            Starknet::config_hash()
        }

        fn fee_token_addresses() -> FeeTokenAddresses {
            Starknet::fee_token_addresses()
        }

        fn is_transaction_fee_disabled() -> bool {
            Starknet::is_transaction_fee_disabled()
        }

        fn estimate_fee(transactions: Vec<AccountTransaction>, simulation_flags: SimulationFlags) -> Result<Result<Vec<(u128, u128)>, SimulationError>, InternalSubstrateError> {
            Starknet::estimate_fee(transactions, &simulation_flags)
        }

        fn re_execute_transactions(transactions_before: Vec<Transaction>, transactions_to_trace: Vec<Transaction>, with_state_diff: bool) -> Result<Result<Vec<(TransactionExecutionInfo, Option<CommitmentStateDiff>)>, SimulationError>, InternalSubstrateError> {
            Starknet::re_execute_transactions(transactions_before, transactions_to_trace, with_state_diff)
        }

        fn estimate_message_fee(message: L1HandlerTransaction) -> Result<Result<(u128, u128, u128), SimulationError>, InternalSubstrateError> {
            Starknet::estimate_message_fee(message)
        }

        fn simulate_transactions(transactions: Vec<AccountTransaction>, simulation_flags: SimulationFlags) -> Result<Result<Vec<(CommitmentStateDiff, TransactionSimulationResult)>, SimulationError>, InternalSubstrateError> {
            Starknet::simulate_transactions(transactions, &simulation_flags)
        }

        fn simulate_message(message: L1HandlerTransaction, simulation_flags: SimulationFlags) -> Result<Result<TransactionExecutionInfo, SimulationError>, InternalSubstrateError> {
            Starknet::simulate_message(message, &simulation_flags)
        }

        fn extrinsic_filter(xts: Vec<<Block as BlockT>::Extrinsic>) -> Vec<Transaction> {
            xts.into_iter().filter_map(|xt| match xt.function {
                RuntimeCall::Starknet( invoke { transaction }) => Some(Transaction::AccountTransaction(AccountTransaction::Invoke(transaction))),
                RuntimeCall::Starknet( declare { transaction }) => Some(Transaction::AccountTransaction(AccountTransaction::Declare(transaction))),
                RuntimeCall::Starknet( deploy_account { transaction }) => Some(Transaction::AccountTransaction(AccountTransaction::DeployAccount(transaction))),
                RuntimeCall::Starknet( consume_l1_message { transaction }) => Some(Transaction::L1HandlerTransaction(transaction)),
                _ => None,
            }).collect::<Vec<Transaction>>()
        }

        fn get_index_and_tx_for_tx_hash(extrinsics: Vec<<Block as BlockT>::Extrinsic>, tx_hash: TransactionHash) -> Option<(u32, Transaction)> {
            // Find our tx and it's index
            let (tx_index, tx) =  extrinsics.into_iter().enumerate().find(|(_, xt)| {
                let computed_tx_hash = match &xt.function {
                    RuntimeCall::Starknet( invoke { transaction }) => transaction.tx_hash,
                    RuntimeCall::Starknet( declare { transaction, .. }) => transaction.tx_hash,
                    RuntimeCall::Starknet( deploy_account { transaction }) => transaction.tx_hash,
                    RuntimeCall::Starknet( consume_l1_message { transaction, .. }) => transaction.tx_hash,
                    _ => return false
                };

                computed_tx_hash == tx_hash
            })?;
            let transaction = match tx.function {
                RuntimeCall::Starknet( invoke { transaction }) => Transaction::AccountTransaction(AccountTransaction::Invoke(transaction)),
                RuntimeCall::Starknet( declare { transaction }) => Transaction::AccountTransaction(AccountTransaction::Declare(transaction)),
                RuntimeCall::Starknet( deploy_account { transaction }) => Transaction::AccountTransaction(AccountTransaction::DeployAccount(transaction)),
                RuntimeCall::Starknet( consume_l1_message { transaction }) => Transaction::L1HandlerTransaction(transaction),
                _ => unreachable!("The previous match made sure that at this point tx is one of those starknet calls"),
            };

            let tx_index = u32::try_from(tx_index).expect("More that u32::MAX extrinsics have been passed. That's too much. You should not be doing that.");
            Some((tx_index, transaction))
        }

        fn get_tx_messages_to_l1(tx_hash: TransactionHash) -> Vec<MessageToL1> {
            Starknet::tx_messages(tx_hash)
        }

        fn get_events_for_tx_by_hash(tx_hash: TransactionHash) -> Vec<StarknetEvent> {
            Starknet::tx_events(tx_hash)
        }

        fn get_tx_execution_outcome(tx_hash: TransactionHash) -> Option<Vec<u8>> {
            Starknet::tx_revert_error(tx_hash).map(|s| s.into_bytes())
        }

        fn get_block_context() -> blockifier::context::BlockContext {
           Starknet::get_block_context()
        }

        fn l1_nonce_unused(nonce: Nonce) -> bool {
            Starknet::ensure_l1_message_not_executed(&nonce).is_ok()
        }
    }

    impl pallet_starknet_runtime_api::ConvertTransactionRuntimeApi<Block> for Runtime {
        fn convert_account_transaction(transaction: AccountTransaction) -> UncheckedExtrinsic {
            let call = match transaction {
                AccountTransaction::Declare(tx) => {
                    pallet_starknet::Call::declare { transaction: tx }
                }
                AccountTransaction::DeployAccount(tx) => {
                    pallet_starknet::Call::deploy_account { transaction: tx  }
                }
                AccountTransaction::Invoke(tx) => {
                    pallet_starknet::Call::invoke { transaction: tx  }
                }
            };

            UncheckedExtrinsic::new_unsigned(call.into())
        }

        fn convert_l1_transaction(transaction: L1HandlerTransaction) -> UncheckedExtrinsic {
            let call =  pallet_starknet::Call::<Runtime>::consume_l1_message { transaction };

            UncheckedExtrinsic::new_unsigned(call.into())
        }

    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;
            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;

            let mut list = Vec::<BenchmarkList>::new();
            list_benchmarks!(list, extra);

            let storage_info = AllPalletsWithSystem::storage_info();

            (list, storage_info)
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch};

            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;

            impl frame_system_benchmarking::Config for Runtime {}
            impl baseline::Config for Runtime {}

            use frame_support::traits::WhitelistedStorageKeys;
            let whitelist: Vec<_> = AllPalletsWithSystem::whitelisted_storage_keys();

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);
            add_benchmarks!(params, batches);

            Ok(batches)
        }
    }

    #[cfg(feature = "try-runtime")]
    impl frame_try_runtime::TryRuntime<Block> for Runtime {
        fn on_runtime_upgrade() -> (Weight, Weight) {
            // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
            // have a backtrace here. If any of the pre/post migration checks fail, we shall stop
            // right here and right now.
            let weight = Executive::try_runtime_upgrade().unwrap();
            (weight, BlockWeights::get().max_block)
        }

        fn execute_block(
            block: Block,
            state_root_check: bool,
            select: frame_try_runtime::TryStateSelect
        ) -> Weight {
            // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
            // have a backtrace here.
            Executive::try_execute_block(block, state_root_check, select).expect("execute-block failed")
        }
    }
}
