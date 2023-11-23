use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use blockifier::execution::contract_class::ContractClass;
use frame_support::assert_ok;
use futures::executor::block_on;
pub use madara_runtime as runtime;
pub use madara_runtime::{
    BuildStorage, GenesisConfig, RuntimeCall, SealingMode, SystemConfig, UncheckedExtrinsic, WASM_BINARY,
};
use mp_felt::Felt252Wrapper;
use mp_transactions::{DeclareTransactionV1, DeployAccountTransaction, InvokeTransactionV1};
use pallet_starknet::genesis_loader::{read_contract_class_from_json, GenesisData, GenesisLoader};
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use pallet_starknet::Call as StarknetCall;
use sc_block_builder::{BlockBuilder, BlockBuilderProvider, RecordProof};
use sc_client_api::ExecutionStrategy::NativeElseWasm;
use sc_client_api::HeaderBackend;
use sc_client_db::DatabaseSource;
use sp_api::ProvideRuntimeApi;
use sp_consensus::BlockOrigin;
use sp_inherents::InherentData;
use sp_runtime::traits::Block as BlockT;
use sp_state_machine::BasicExternalities;
use sp_timestamp::{Timestamp, INHERENT_IDENTIFIER};
use starknet_api::api_core::{ContractAddress, EntryPointSelector, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::Calldata;
use substrate_test_client::*;
use tempfile::tempdir;

use crate::sync::*;
use crate::tests::constants::*;
use crate::tests::helpers::*;

pub type Backend = sc_client_db::Backend<runtime::Block>;

pub struct MadaraExecutorDispatch;
impl sc_executor::NativeExecutionDispatch for MadaraExecutorDispatch {
    /// Only enable the benchmarking host functions when we actually want to benchmark.
    #[cfg(feature = "runtime-benchmarks")]
    type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
    /// Otherwise we only use the default Substrate host functions.
    #[cfg(not(feature = "runtime-benchmarks"))]
    type ExtendHostFunctions = ();

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        madara_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        madara_runtime::native_version()
    }
}

pub type ExecutorDispatch = sc_executor::NativeElseWasmExecutor<MadaraExecutorDispatch>;

/// Test client type.
pub type Client = client::Client<
    Backend,
    client::LocalCallExecutor<runtime::Block, Backend, ExecutorDispatch>,
    runtime::Block,
    runtime::RuntimeApi,
>;

#[derive(Default)]
pub struct GenesisParameters;

impl substrate_test_client::GenesisInit for GenesisParameters {
    fn genesis_storage(&self) -> Storage {
        let genesis_data: GenesisData = serde_json::from_str(std::include_str!("./genesis.json")).unwrap();
        let genesis_loader = GenesisLoader::new(project_root::get_project_root().unwrap(), genesis_data);

        let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string()).unwrap();

        let mut storage = GenesisConfig {
            system: SystemConfig { code: wasm_binary.to_vec() },
            aura: Default::default(),
            grandpa: Default::default(),
            starknet: genesis_loader.into(),
        }
        .build_storage()
        .unwrap();

        BasicExternalities::execute_with_storage(&mut storage, || {
            madara_runtime::Sealing::set(&SealingMode::Manual);
        });

        storage
    }
}

pub type TestClientBuilder<E, B> = substrate_test_client::TestClientBuilder<runtime::Block, E, B, GenesisParameters>;

pub trait TestClientBuilderExt: Sized {
    /// Create test client builder.
    fn new() -> Self;

    /// Build the test client.
    fn build(self) -> (Client, Arc<Backend>);
}

impl TestClientBuilderExt
    for substrate_test_client::TestClientBuilder<
        runtime::Block,
        client::LocalCallExecutor<runtime::Block, Backend, ExecutorDispatch>,
        Backend,
        GenesisParameters,
    >
{
    fn new() -> Self {
        Self::default()
    }

    fn build(self) -> (Client, Arc<Backend>) {
        let backend = self.backend();
        (self.set_execution_strategy(NativeElseWasm).build_with_native_executor(None).0, backend)
    }
}

// create_test_client with has apply genesis then build and import the first block.
pub fn create_test_client() -> (Client, Arc<Backend>) {
    let (mut client, backend) = TestClientBuilder::new().build();

    let block_info = client.info();
    let mut builder = client.new_block_at(block_info.best_hash, Default::default(), RecordProof::Yes).unwrap();
    let mut inherent_data = InherentData::new();
    inherent_data.put_data(INHERENT_IDENTIFIER, &Timestamp::new(100)).unwrap();

    let inherent_exts = builder.create_inherents(inherent_data).unwrap();
    for ex in inherent_exts {
        builder.push(ex).unwrap();
    }

    let block = builder.build().unwrap();

    assert_ok!(block_on(client.import(BlockOrigin::Own, block.block)));
    (client, backend)
}

fn build_get_balance_contract_call(account_address: StarkFelt) -> (EntryPointSelector, Calldata) {
    let balance_of_selector = EntryPointSelector(
        StarkFelt::try_from("0x02e4263afad30923c891518314c3c95dbe830a16874e8abc5777a9a20b54c76e").unwrap(),
    );
    let calldata = Calldata(Arc::new(vec![
        account_address, // owner address
    ]));

    (balance_of_selector, calldata)
}

fn apply_inherents_for_block_builder<'a, Block: BlockT, C, BE>(
    block_builder: &mut BlockBuilder<'a, Block, C, BE>,
    block_info: &sp_blockchain::Info<madara_runtime::Block>,
) where
    BE: sc_client_api::backend::Backend<Block> + Send + Sync + 'static,
    C: BlockBuilderProvider<BE, Block, C> + HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync + 'static,
    C::Api: sp_api::ApiExt<Block, StateBackend = sc_client_api::backend::StateBackendFor<BE, Block>>
        + sc_block_builder::BlockBuilderApi<Block>,
{
    let mut inherents = InherentData::new();
    inherents
        .put_data(INHERENT_IDENTIFIER, &Timestamp::new(6000u64 * (block_info.best_number as u64 + 1) + 2))
        .unwrap();

    let inherents_exs = block_builder.create_inherents(inherents).unwrap();
    for ex in inherents_exs {
        block_builder.push(ex).unwrap();
    }
}

pub fn get_contract_class(resource_path: &str, version: u8) -> ContractClass {
    let cargo_dir = String::from(env!("CARGO_MANIFEST_DIR"));
    let build_path = match version {
        0 => "/../../../cairo-contracts/build/",
        1 => "/../../../cairo-contracts/build/cairo_1/",
        _ => unimplemented!("Unsupported version {} to get contract class", version),
    };
    let full_path = cargo_dir + build_path + resource_path;
    let full_path: PathBuf = [full_path].iter().collect();
    let raw_contract_class = fs::read_to_string(full_path).unwrap();
    read_contract_class_from_json(&raw_contract_class, version)
}

#[test]
fn test_apply_deploy_contract_state_diff() {
    let (client, backend) = create_test_client();
    let client = Arc::new(client);

    let sender_account = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));
    let felt_252_sender_account = sender_account.into();
    // ERC20 is already declared for the fees.
    // Deploy ERC20 contract
    let deploy_transaction = InvokeTransactionV1 {
        max_fee: u128::MAX,
        signature: vec![],
        nonce: Felt252Wrapper::ZERO,
        sender_address: felt_252_sender_account,
        calldata: vec![
            felt_252_sender_account, // Simple contract address
            Felt252Wrapper::from_hex_be("0x02730079d734ee55315f4f141eaed376bddd8c2133523d223a344c5604e0f7f8").unwrap(), /* deploy_contract selector */
            Felt252Wrapper::from_hex_be("0x9").unwrap(), // Calldata len
            Felt252Wrapper::from_hex_be(TOKEN_CONTRACT_CLASS_HASH).unwrap(), // Class hash
            Felt252Wrapper::ONE,                         // Contract address salt
            Felt252Wrapper::from_hex_be("0x6").unwrap(), // Constructor_calldata_len
            Felt252Wrapper::from_hex_be("0xA").unwrap(), // Name
            Felt252Wrapper::from_hex_be("0x1").unwrap(), // Symbol
            Felt252Wrapper::from_hex_be("0x2").unwrap(), // Decimals
            Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(), // Initial supply low
            Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(), // Initial supply high
            felt_252_sender_account,                     // recipient
        ],
    };

    let block_info = client.info();
    let mut builder = client.new_block_at(block_info.best_hash, Default::default(), RecordProof::Yes).unwrap();

    apply_inherents_for_block_builder(&mut builder, &block_info);

    let call: RuntimeCall =
        StarknetCall::invoke { transaction: mp_transactions::InvokeTransaction::V1(deploy_transaction) }.into();
    let ext = UncheckedExtrinsic { signature: None, function: call };
    builder.push(ext).unwrap();

    // build a block, get storage changes after deploy contract.
    let block = builder.build().unwrap();
    let ics = InnerStorageChangeSet {
        changes: block.storage_changes.main_storage_changes.clone(),
        child_changes: block.storage_changes.child_storage_changes.clone(),
    };

    let inner_state_diff: SyncStateDiff = ics.into();
    let ics2 = InnerStorageChangeSet::from(inner_state_diff.clone());
    let inner_state_diff2: SyncStateDiff = ics2.into();

    assert_eq!(inner_state_diff, inner_state_diff2);
    // apply storage diff by StateSyncWorker
    let madara_db = create_temp_madara_backend();
    let mut sync_worker = StateWriter::new(client.clone(), backend, madara_db);
    sync_worker.apply_state_diff(2, inner_state_diff2).unwrap();

    let expected_erc20_address = ContractAddress(PatriciaKey(
        StarkFelt::try_from("00dc58c1280862c95964106ef9eba5d9ed8c0c16d05883093e4540f22b829dff").unwrap(),
    ));

    let block_info = client.info();
    let call_args = build_get_balance_contract_call(sender_account.0.0);

    // call the deployed contract,
    pretty_assertions::assert_eq!(
        client
            .runtime_api()
            .call(block_info.best_hash, expected_erc20_address, call_args.0, call_args.1)
            .unwrap()
            .unwrap(),
        vec![
            Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(),
            Felt252Wrapper::from_hex_be("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap()
        ]
    );
}

#[test]
fn test_apply_declare_contract_state_diff() {
    let account_addr = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));

    let erc20_class = get_contract_class("ERC20.json", 0);
    let erc20_class_hash =
        Felt252Wrapper::from_hex_be("0x057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

    let transaction = DeclareTransactionV1 {
        sender_address: account_addr.into(),
        class_hash: erc20_class_hash,
        nonce: Felt252Wrapper::ZERO,
        max_fee: u128::MAX,
        signature: vec![],
    };

    let (client, backend) = create_test_client();
    let client = Arc::new(client);

    let block_info = client.info();
    let mut builder = client.new_block_at(block_info.best_hash, Default::default(), RecordProof::Yes).unwrap();
    apply_inherents_for_block_builder(&mut builder, &block_info);

    let call: RuntimeCall = StarknetCall::declare {
        transaction: mp_transactions::DeclareTransaction::V1(transaction),
        contract_class: erc20_class.clone(),
    }
    .into();

    let ext = UncheckedExtrinsic { signature: None, function: call };
    builder.push(ext.clone()).unwrap();

    let block = builder.build().unwrap();
    let ics = InnerStorageChangeSet {
        changes: block.storage_changes.main_storage_changes.clone(),
        child_changes: block.storage_changes.child_storage_changes.clone(),
    };

    let state_diff: SyncStateDiff = ics.into();
    let ics2 = InnerStorageChangeSet::from(state_diff.clone());
    let state_diff2: SyncStateDiff = ics2.into();

    assert_eq!(state_diff, state_diff2);

    // apply storage diff by StateSyncWorker
    let madara_db = create_temp_madara_backend();
    let mut sync_worker = StateWriter::new(client.clone(), backend, madara_db);
    sync_worker.apply_state_diff(2, state_diff2).unwrap();

    let block_info = client.info();
    let declared_contract = client
        .runtime_api()
        .contract_class_by_class_hash(block_info.best_hash, erc20_class_hash.into())
        .unwrap()
        .unwrap();
    assert_eq!(erc20_class, declared_contract);
}

#[test]
fn test_apply_deploy_account_state_diff() {
    let (client, backend) = create_test_client();
    let client = Arc::new(client);
    let block_info = client.info();
    let mut builder = client.new_block_at(block_info.best_hash, Default::default(), RecordProof::Yes).unwrap();
    apply_inherents_for_block_builder(&mut builder, &block_info);

    let (account_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::NoValidate));
    let deploy_tx = DeployAccountTransaction {
        max_fee: u128::MAX,
        signature: vec![],
        nonce: Felt252Wrapper::ZERO,
        contract_address_salt: *SALT,
        constructor_calldata: calldata.0.iter().map(|e| Felt252Wrapper::from(*e)).collect(),
        class_hash: account_class_hash.into(),
    };

    // The balance of this deploy account is write in genesis.
    let address: ContractAddress = deploy_tx.account_address().into();
    let call: RuntimeCall = StarknetCall::deploy_account { transaction: deploy_tx }.into();
    let ext = UncheckedExtrinsic { signature: None, function: call };
    builder.push(ext.clone()).unwrap();

    let block = builder.build().unwrap();
    let ics = InnerStorageChangeSet {
        changes: block.storage_changes.main_storage_changes.clone(),
        child_changes: block.storage_changes.child_storage_changes.clone(),
    };

    let state_diff: SyncStateDiff = ics.into();
    let ics2 = InnerStorageChangeSet::from(state_diff.clone());
    let state_diff2: SyncStateDiff = ics2.into();

    assert_eq!(state_diff, state_diff2);

    // apply storage diff by StateSyncWorker
    let madara_db = create_temp_madara_backend();
    let mut sync_worker = StateWriter::new(client.clone(), backend, madara_db);
    sync_worker.apply_state_diff(2, state_diff2).unwrap();

    let block_info = client.info();
    let deployed_account_class_hash =
        client.runtime_api().contract_class_hash_by_address(block_info.best_hash, address.into()).unwrap();

    assert_eq!(deployed_account_class_hash, account_class_hash);
}

fn create_temp_madara_backend() -> Arc<mc_db::Backend<runtime::Block>> {
    let temp_dir = tempdir().unwrap();
    let temp_dir_path = temp_dir.path();
    let madara_db = mc_db::Backend::<runtime::Block>::open(
        &DatabaseSource::RocksDb { path: temp_dir_path.to_path_buf(), cache_size: 1024 },
        temp_dir_path,
        false,
    )
    .unwrap();
    Arc::new(madara_db)
}
