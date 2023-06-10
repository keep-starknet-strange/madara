use frame_benchmarking::frame_support::pallet_prelude::IsType;
use futures::channel::mpsc;
use madara_runtime::Hash;
use mc_rpc::{Starknet, StarknetRpcApiServer};
use mc_storage::overrides_handle;
use mp_starknet::execution::types::EntryPointTypeWrapper::Constructor;
use mp_starknet::execution::types::{ContractAddressWrapper, Felt252Wrapper};
use mp_starknet::transaction::types::EventWrapper;
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use sc_cli::{ChainSpec, CliConfiguration, RuntimeVersion, SubstrateCli};
use sc_client_api::BlockBackend;
use sc_consensus_manual_seal::EngineCommand;
use sc_executor::sp_wasm_interface::wasmtime::Engine;
use sc_service::{Arc, BasePath, Configuration, WarpSyncParams};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::bounded_vec;
use sp_runtime::BoundedVec;

use crate::chain_spec;
use crate::cli::{Cli, Sealing};
use crate::rpc::{FullDeps, StarknetDeps};
use crate::service::{build_manual_seal_import_queue, new_full, new_partial, FullClient};

#[tokio::test]
async fn get_events_work() {
    let cli = Cli::from_iter(["--dev", "--sealing=instant"].iter());

    let config = cli.create_configuration(&cli.run, tokio::runtime::Handle::try_current().unwrap()).unwrap();
    println!("adasdasdsadsada");

    let build_import_queue = build_manual_seal_import_queue;

    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (block_import, grandpa_link, mut telemetry, madara_backend),
    } = new_partial(&config, build_import_queue).unwrap();

    let mut net_config = sc_network::config::FullNetworkConfiguration::new(&config.network);

    let grandpa_protocol_name = sc_consensus_grandpa::protocol_standard_name(
        &client.block_hash(0).ok().flatten().expect("Genesis block exists; qed"),
        &config.chain_spec,
    );

    net_config.add_notification_protocol(sc_consensus_grandpa::grandpa_peers_set_config(grandpa_protocol_name.clone()));
    let warp_sync = Arc::new(sc_consensus_grandpa::warp_proof::NetworkProvider::new(
        backend.clone(),
        grandpa_link.shared_authority_set().clone(),
        Vec::default(),
    ));
    let warp_sync_params = Some(WarpSyncParams::WithProvider(warp_sync));

    let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            net_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync_params,
        })
        .unwrap();

    if config.offchain_worker.enabled {
        sc_service::build_offchain_workers(&config, task_manager.spawn_handle(), client.clone(), network.clone());
    }

    let role = config.role.clone();
    let force_authoring = config.force_authoring;
    let backoff_authoring_blocks: Option<()> = None;
    let name = config.network.node_name.clone();
    let enable_grandpa = !config.disable_grandpa && false;
    let prometheus_registry = config.prometheus_registry().cloned();
    let starting_block = client.info().best_number;

    let overrides = overrides_handle(client.clone());
    let starknet_rpc_params = StarknetDeps {
        client: client.clone(),
        madara_backend: madara_backend.clone(),
        overrides,
        sync_service: sync_service.clone(),
        starting_block,
    };

    let client = client.clone();
    let pool = transaction_pool.clone();
    let hasher = client.runtime_api().get_hasher(client.info().best_hash).unwrap().into();

    let event1 = build_event_wrapper_for_test(&["0x1"], 1);
    let event2 = build_event_wrapper_for_test(&["0x2"], 1);
    let event3 = build_event_wrapper_for_test(&["0x3"], 1);
    let event4 = build_event_wrapper_for_test(&["0x4"], 1);

    let events = vec![event1, event2, event3, event4];

    let starknet = Starknet::new(
        client,
        starknet_rpc_params.madara_backend,
        starknet_rpc_params.overrides,
        pool,
        starknet_rpc_params.sync_service,
        starknet_rpc_params.starting_block,
        hasher,
    );
    let filtered_events = starknet.filter_events_by_params(
        events,
        None,
        vec![vec![Felt252Wrapper::from_hex_be("0x1").unwrap().into()]],
        None,
    );
    dbg!(filtered_events);
}

fn build_event_wrapper_for_test(keys: &[&str], address_int: u64) -> EventWrapper {
    let keys_felt = keys.iter().map(|key| Felt252Wrapper::from_hex_be(key).unwrap()).collect::<Vec<Felt252Wrapper>>();
    EventWrapper {
        keys: BoundedVec::try_from(keys_felt).unwrap(),
        data: bounded_vec!(),
        from_address: ContractAddressWrapper::from(address_int),
        transaction_hash: Felt252Wrapper::from(1_u64),
    }
}
