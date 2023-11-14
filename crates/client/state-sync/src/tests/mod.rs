//! Madara client testing utilities.
pub mod constants;
pub mod helpers;
pub mod l1;

use std::sync::Arc;

use blockifier::state::cached_state::CommitmentStateDiff;
use constants::*;
use frame_support::assert_ok;
use futures::executor::block_on;
use helpers::*;
pub use madara_runtime as runtime;
pub use madara_runtime::{
    BuildStorage, GenesisConfig, RuntimeCall, SealingMode, SystemConfig, UncheckedExtrinsic, WASM_BINARY,
};
use mp_felt::Felt252Wrapper;
use mp_transactions::InvokeTransactionV1;
use pallet_starknet::genesis_loader::{GenesisData, GenesisLoader};
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use pallet_starknet::Call as StarknetCall;
use sc_block_builder::{BlockBuilderProvider, RecordProof};
use sc_client_api::ExecutionStrategy::NativeElseWasm;
use sc_client_api::HeaderBackend;
use sp_api::ProvideRuntimeApi;
use sp_consensus::BlockOrigin;
use sp_inherents::InherentData;
use sp_state_machine::BasicExternalities;
use sp_timestamp::{Timestamp, INHERENT_IDENTIFIER};
use starknet_api::api_core::{ContractAddress, EntryPointSelector, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::Calldata;
pub use substrate_test_client::*;

use crate::sync::*;

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

#[test]
fn test_basic_state_diff() {
    // 1. make block (transfer, deploy)
    // 2. client new block builder
    // 3. block builder build block, get {block, state changes, applied state root}
    // 4. get state diff from state changes
    // 4. apply state diff to backend.
    // 5. check starknet contract state by runtime api
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

    let call: RuntimeCall =
        StarknetCall::invoke { transaction: mp_transactions::InvokeTransaction::V1(deploy_transaction) }.into();
    let ext = UncheckedExtrinsic { signature: None, function: call };

    let block_info = client.info();

    let mut builder = client.new_block_at(block_info.best_hash, Default::default(), RecordProof::Yes).unwrap();
    let mut inherents = InherentData::new();
    inherents
        .put_data(INHERENT_IDENTIFIER, &Timestamp::new(6000u64 * (block_info.best_number as u64 + 1) + 2))
        .unwrap();

    let inherents_exs = builder.create_inherents(inherents).unwrap();
    for ex in inherents_exs {
        builder.push(ex).unwrap();
    }
    builder.push(ext).unwrap();

    let block = builder.build().unwrap();
    let ics = InnerStorageChangeSet {
        changes: block.storage_changes.main_storage_changes.clone(),
        child_changes: block.storage_changes.child_storage_changes.clone(),
    };

    let commitment_state_diff: CommitmentStateDiff = ics.into();
    let ics2 = InnerStorageChangeSet::from(commitment_state_diff.clone());
    let commitment_state_diff2: CommitmentStateDiff = ics2.into();

    assert_eq!(commitment_state_diff, commitment_state_diff2);

    let mut sync_worker = StateSyncWorker::new(client.clone(), backend);
    sync_worker.apply_state_diff(2, commitment_state_diff2).unwrap();

    // call contract
    let expected_erc20_address = ContractAddress(PatriciaKey(
        StarkFelt::try_from("00dc58c1280862c95964106ef9eba5d9ed8c0c16d05883093e4540f22b829dff").unwrap(),
    ));

    let block_info = client.info();
    let call_args = build_get_balance_contract_call(sender_account.0.0);

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

pub fn build_get_balance_contract_call(account_address: StarkFelt) -> (EntryPointSelector, Calldata) {
    let balance_of_selector = EntryPointSelector(
        StarkFelt::try_from("0x02e4263afad30923c891518314c3c95dbe830a16874e8abc5777a9a20b54c76e").unwrap(),
    );
    let calldata = Calldata(Arc::new(vec![
        account_address, // owner address
    ]));

    (balance_of_selector, calldata)
}
