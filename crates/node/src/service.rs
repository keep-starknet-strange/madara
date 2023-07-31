//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use std::cell::RefCell;
use std::sync::Arc;
use std::time::Duration;

use futures::channel::mpsc;
use futures::future;
use futures::prelude::*;
use madara_runtime::opaque::Block;
use madara_runtime::{self, Hash, RuntimeApi};
use mc_block_proposer::ProposerFactory;
use mc_mapping_sync::MappingSyncWorker;
use mc_storage::overrides_handle;
use mc_transaction_pool::FullPool;
use mp_starknet::sequencer_address::{
    InherentDataProvider as SeqAddrInherentDataProvider, DEFAULT_SEQUENCER_ADDRESS, SEQ_ADDR_STORAGE_KEY,
};
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use prometheus_endpoint::Registry;
use sc_client_api::{Backend, BlockBackend, BlockchainEvents, HeaderBackend};
use sc_consensus::BasicQueue;
use sc_consensus_aura::{SlotProportion, StartAuraParams};
use sc_consensus_grandpa::{GrandpaBlockImport, SharedVoterState};
pub use sc_executor::NativeElseWasmExecutor;
use sc_service::error::Error as ServiceError;
use sc_service::{new_db_backend, Configuration, TaskManager, WarpSyncParams};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker};
use sp_api::offchain::OffchainStorage;
use sp_api::{ConstructRuntimeApi, ProvideRuntimeApi, TransactionFor};
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;
use sp_offchain::STORAGE_PREFIX;
use sp_runtime::traits::BlakeTwo256;
use sp_trie::PrefixedMemoryDB;

use crate::cli::Sealing;
use crate::genesis_block::MadaraGenesisBlockBuilder;
use crate::rpc::StarknetDeps;
use crate::starknet::{db_config_dir, MadaraBackend};
// Our native executor instance.
pub struct ExecutorDispatch;

impl sc_executor::NativeExecutionDispatch for ExecutorDispatch {
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

pub(crate) type FullClient = sc_service::TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<ExecutorDispatch>>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

type BasicImportQueue<Client> = sc_consensus::DefaultImportQueue<Block, Client>;
type BoxBlockImport<Client> = sc_consensus::BoxBlockImport<Block, TransactionFor<Client, Block>>;

#[allow(clippy::type_complexity)]
pub fn new_partial<BIQ>(
    config: &Configuration,
    build_import_queue: BIQ,
) -> Result<
    sc_service::PartialComponents<
        FullClient,
        FullBackend,
        FullSelectChain,
        sc_consensus::DefaultImportQueue<Block, FullClient>,
        mc_transaction_pool::FullPool<Block, FullClient>,
        (
            BoxBlockImport<FullClient>,
            sc_consensus_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
            Option<Telemetry>,
            Arc<MadaraBackend>,
        ),
    >,
    ServiceError,
>
where
    RuntimeApi: ConstructRuntimeApi<Block, FullClient>,
    RuntimeApi: Send + Sync + 'static,
    BIQ: FnOnce(
        Arc<FullClient>,
        &Configuration,
        &TaskManager,
        Option<TelemetryHandle>,
        GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>,
        Arc<MadaraBackend>,
    ) -> Result<(BasicImportQueue<FullClient>, BoxBlockImport<FullClient>), ServiceError>,
{
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = sc_service::new_native_or_wasm_executor(config);

    let backend = new_db_backend(config.db_config())?;

    let genesis_block_builder = MadaraGenesisBlockBuilder::<Block, _, _>::new(
        config.chain_spec.as_storage_builder(),
        true,
        backend.clone(),
        executor.clone(),
    )
    .unwrap();

    let (client, backend, keystore_container, task_manager) = sc_service::new_full_parts_with_genesis_builder::<
        Block,
        RuntimeApi,
        _,
        MadaraGenesisBlockBuilder<Block, FullBackend, NativeElseWasmExecutor<ExecutorDispatch>>,
    >(
        config,
        telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
        executor,
        backend,
        genesis_block_builder,
    )?;

    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager.spawn_handle().spawn("telemetry", None, worker.run());
        telemetry
    });

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = mc_transaction_pool::BasicPool::new_full(
        mc_transaction_pool::Options::from(config.transaction_pool.clone()),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );

    let (grandpa_block_import, grandpa_link) = sc_consensus_grandpa::block_import(
        client.clone(),
        &client as &Arc<_>,
        select_chain.clone(),
        telemetry.as_ref().map(|x| x.handle()),
    )?;

    let madara_backend = Arc::new(MadaraBackend::open(&config.database, &db_config_dir(config))?);

    let (import_queue, block_import) = build_import_queue(
        client.clone(),
        config,
        &task_manager,
        telemetry.as_ref().map(|x| x.handle()),
        grandpa_block_import,
        madara_backend.clone(),
    )?;

    Ok(sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (block_import, grandpa_link, telemetry, madara_backend),
    })
}

/// Build the import queue for the template runtime (aura + grandpa).
pub fn build_aura_grandpa_import_queue(
    client: Arc<FullClient>,
    config: &Configuration,
    task_manager: &TaskManager,
    telemetry: Option<TelemetryHandle>,
    grandpa_block_import: GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>,
    _madara_backend: Arc<MadaraBackend>,
) -> Result<(BasicImportQueue<FullClient>, BoxBlockImport<FullClient>), ServiceError>
where
    RuntimeApi: ConstructRuntimeApi<Block, FullClient>,
    RuntimeApi: Send + Sync + 'static,
{
    let slot_duration = sc_consensus_aura::slot_duration(&*client)?;

    let create_inherent_data_providers = move |_, ()| async move {
        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
        let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
            *timestamp,
            slot_duration,
        );
        Ok((slot, timestamp))
    };

    let import_queue =
        sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _>(sc_consensus_aura::ImportQueueParams {
            block_import: grandpa_block_import.clone(),
            justification_import: Some(Box::new(grandpa_block_import.clone())),
            client,
            create_inherent_data_providers,
            spawner: &task_manager.spawn_essential_handle(),
            registry: config.prometheus_registry(),
            check_for_equivocation: Default::default(),
            telemetry,
            compatibility_mode: sc_consensus_aura::CompatibilityMode::None,
        })
        .map_err::<ServiceError, _>(Into::into)?;

    Ok((import_queue, Box::new(grandpa_block_import)))
}

/// Build the import queue for the template runtime (manual seal).
pub fn build_manual_seal_import_queue(
    client: Arc<FullClient>,
    config: &Configuration,
    task_manager: &TaskManager,
    _telemetry: Option<TelemetryHandle>,
    _grandpa_block_import: GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>,
    _madara_backend: Arc<MadaraBackend>,
) -> Result<(BasicImportQueue<FullClient>, BoxBlockImport<FullClient>), ServiceError>
where
    RuntimeApi: ConstructRuntimeApi<Block, FullClient>,
    RuntimeApi: Send + Sync + 'static,
{
    Ok((
        sc_consensus_manual_seal::import_queue(
            Box::new(client.clone()),
            &task_manager.spawn_essential_handle(),
            config.prometheus_registry(),
        ),
        Box::new(client),
    ))
}

/// Builds a new service for a full client.
pub fn new_full(config: Configuration, sealing: Option<Sealing>) -> Result<TaskManager, ServiceError> {
    let build_import_queue =
        if sealing.is_some() { build_manual_seal_import_queue } else { build_aura_grandpa_import_queue };

    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (block_import, grandpa_link, mut telemetry, madara_backend),
    } = new_partial(&config, build_import_queue)?;

    let mut net_config = sc_network::config::FullNetworkConfiguration::new(&config.network);

    let grandpa_protocol_name = sc_consensus_grandpa::protocol_standard_name(
        &client.block_hash(0).ok().flatten().expect("Genesis block exists; qed"),
        &config.chain_spec,
    );

    let warp_sync_params = if sealing.is_some() {
        None
    } else {
        net_config
            .add_notification_protocol(sc_consensus_grandpa::grandpa_peers_set_config(grandpa_protocol_name.clone()));
        let warp_sync = Arc::new(sc_consensus_grandpa::warp_proof::NetworkProvider::new(
            backend.clone(),
            grandpa_link.shared_authority_set().clone(),
            Vec::default(),
        ));
        Some(WarpSyncParams::WithProvider(warp_sync))
    };

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
        })?;

    if config.offchain_worker.enabled {
        sc_service::build_offchain_workers(&config, task_manager.spawn_handle(), client.clone(), network.clone());
    }

    let role = config.role.clone();
    let force_authoring = config.force_authoring;
    let backoff_authoring_blocks: Option<()> = None;
    let name = config.network.node_name.clone();
    let enable_grandpa = !config.disable_grandpa && sealing.is_none();
    let prometheus_registry = config.prometheus_registry().cloned();
    let starting_block = client.info().best_number;

    // Channel for the rpc handler to communicate with the authorship task.
    // TODO: commands_stream is is currently unused, but should be used to implement the `sealing`
    // parameter
    let (command_sink, commands_stream) = mpsc::channel(1000);

    let overrides = overrides_handle(client.clone());
    let starknet_rpc_params = StarknetDeps {
        client: client.clone(),
        madara_backend: madara_backend.clone(),
        overrides,
        sync_service: sync_service.clone(),
        starting_block,
    };

    let rpc_extensions_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();
        let graph = transaction_pool.pool().clone();

        Box::new(move |deny_unsafe, _| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                graph: graph.clone(),
                deny_unsafe,
                starknet: starknet_rpc_params.clone(),
                command_sink: if sealing.is_some() { Some(command_sink.clone()) } else { None },
            };
            crate::rpc::create_full(deps).map_err(Into::into)
        })
    };

    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network: network.clone(),
        client: client.clone(),
        keystore: keystore_container.keystore(),
        task_manager: &mut task_manager,
        transaction_pool: transaction_pool.clone(),
        rpc_builder: rpc_extensions_builder,
        backend: backend.clone(),
        system_rpc_tx,
        tx_handler_controller,
        sync_service: sync_service.clone(),
        config,
        telemetry: telemetry.as_mut(),
    })?;

    let hasher =
        client.runtime_api().get_hasher(client.info().best_hash).map_err(|e| ServiceError::Client(e.into()))?.into();

    task_manager.spawn_essential_handle().spawn(
        "mc-mapping-sync-worker",
        Some("madara"),
        MappingSyncWorker::new(
            client.import_notification_stream(),
            Duration::new(6, 0),
            client.clone(),
            backend.clone(),
            madara_backend,
            3,
            0,
            hasher,
        )
        .for_each(|()| future::ready(())),
    );

    if role.is_authority() {
        // manual-seal authorship
        if let Some(sealing) = sealing {
            run_manual_seal_authorship(
                sealing,
                client,
                transaction_pool,
                select_chain,
                block_import,
                &task_manager,
                prometheus_registry.as_ref(),
                commands_stream,
            )?;

            network_starter.start_network();

            log::info!("Manual Seal Ready");
            return Ok(task_manager);
        }

        let proposer_factory = ProposerFactory::new(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool,
            prometheus_registry.as_ref(),
        );

        let slot_duration = sc_consensus_aura::slot_duration(&*client)?;

        let aura = sc_consensus_aura::start_aura::<AuraPair, _, _, _, _, _, _, _, _, _, _>(StartAuraParams {
            slot_duration,
            client,
            select_chain,
            block_import,
            proposer_factory,
            create_inherent_data_providers: move |_, ()| {
                let offchain_storage = backend.offchain_storage();
                async move {
                    let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                    let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                        *timestamp,
                        slot_duration,
                    );

                    let ocw_storage = offchain_storage.clone();
                    let prefix = &STORAGE_PREFIX;
                    let key = SEQ_ADDR_STORAGE_KEY;

                    let sequencer_address = if let Some(storage) = ocw_storage {
                        SeqAddrInherentDataProvider::try_from(
                            storage.get(prefix, key).unwrap_or(DEFAULT_SEQUENCER_ADDRESS.to_vec()),
                        )
                        .unwrap_or_default()
                    } else {
                        SeqAddrInherentDataProvider::default()
                    };

                    Ok((slot, timestamp, sequencer_address))
                }
            },
            force_authoring,
            backoff_authoring_blocks,
            keystore: keystore_container.keystore(),
            sync_oracle: sync_service.clone(),
            justification_sync_link: sync_service.clone(),
            block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
            max_block_proposal_slot_portion: None,
            telemetry: telemetry.as_ref().map(|x| x.handle()),
            compatibility_mode: Default::default(),
        })?;

        // the AURA authoring task is considered essential, i.e. if it
        // fails we take down the service with it.
        task_manager.spawn_essential_handle().spawn_blocking("aura", Some("block-authoring"), aura);
    }

    if enable_grandpa {
        // if the node isn't actively participating in consensus then it doesn't
        // need a keystore, regardless of which protocol we use below.
        let keystore = if role.is_authority() { Some(keystore_container.keystore()) } else { None };

        let grandpa_config = sc_consensus_grandpa::Config {
            // FIXME #1578 make this available through chainspec
            gossip_duration: Duration::from_millis(333),
            justification_period: 512,
            name: Some(name),
            observer_enabled: false,
            keystore,
            local_role: role,
            telemetry: telemetry.as_ref().map(|x| x.handle()),
            protocol_name: grandpa_protocol_name,
        };

        // start the full GRANDPA voter
        // NOTE: non-authorities could run the GRANDPA observer protocol, but at
        // this point the full voter should provide better guarantees of block
        // and vote data availability than the observer. The observer has not
        // been tested extensively yet and having most nodes in a network run it
        // could lead to finality stalls.
        let grandpa_config = sc_consensus_grandpa::GrandpaParams {
            config: grandpa_config,
            link: grandpa_link,
            network,
            sync: Arc::new(sync_service),
            voting_rule: sc_consensus_grandpa::VotingRulesBuilder::default().build(),
            prometheus_registry,
            shared_voter_state: SharedVoterState::empty(),
            telemetry: telemetry.as_ref().map(|x| x.handle()),
        };

        // the GRANDPA voter task is considered infallible, i.e.
        // if it fails we take down the service with it.
        task_manager.spawn_essential_handle().spawn_blocking(
            "grandpa-voter",
            None,
            sc_consensus_grandpa::run_grandpa_voter(grandpa_config)?,
        );
    }

    network_starter.start_network();
    Ok(task_manager)
}

#[allow(clippy::too_many_arguments)]
fn run_manual_seal_authorship(
    sealing: Sealing,
    client: Arc<FullClient>,
    transaction_pool: Arc<FullPool<Block, FullClient>>,
    select_chain: FullSelectChain,
    block_import: BoxBlockImport<FullClient>,
    task_manager: &TaskManager,
    prometheus_registry: Option<&Registry>,
    commands_stream: mpsc::Receiver<sc_consensus_manual_seal::rpc::EngineCommand<Hash>>,
) -> Result<(), ServiceError>
where
    RuntimeApi: ConstructRuntimeApi<Block, FullClient>,
    RuntimeApi: Send + Sync + 'static,
{
    let proposer_factory = ProposerFactory::new(
        task_manager.spawn_handle(),
        client.clone(),
        transaction_pool.clone(),
        prometheus_registry,
    );

    thread_local!(static TIMESTAMP: RefCell<u64> = RefCell::new(0));

    /// Provide a mock duration starting at 0 in millisecond for timestamp inherent.
    /// Each call will increment timestamp by slot_duration making Aura think time has passed.
    struct MockTimestampInherentDataProvider;

    #[async_trait::async_trait]
    impl sp_inherents::InherentDataProvider for MockTimestampInherentDataProvider {
        async fn provide_inherent_data(
            &self,
            inherent_data: &mut sp_inherents::InherentData,
        ) -> Result<(), sp_inherents::Error> {
            TIMESTAMP.with(|x| {
                *x.borrow_mut() += madara_runtime::SLOT_DURATION;
                inherent_data.put_data(sp_timestamp::INHERENT_IDENTIFIER, &*x.borrow())
            })
        }

        async fn try_handle_error(
            &self,
            _identifier: &sp_inherents::InherentIdentifier,
            _error: &[u8],
        ) -> Option<Result<(), sp_inherents::Error>> {
            // The pallet never reports error.
            None
        }
    }

    let create_inherent_data_providers = move |_, ()| async move {
        let timestamp = MockTimestampInherentDataProvider;
        Ok(timestamp)
    };

    let manual_seal = match sealing {
        Sealing::Manual => future::Either::Left(sc_consensus_manual_seal::run_manual_seal(
            sc_consensus_manual_seal::ManualSealParams {
                block_import,
                env: proposer_factory,
                client,
                pool: transaction_pool,
                commands_stream,
                select_chain,
                consensus_data_provider: None,
                create_inherent_data_providers,
            },
        )),
        Sealing::Instant => future::Either::Right(sc_consensus_manual_seal::run_instant_seal(
            sc_consensus_manual_seal::InstantSealParams {
                block_import,
                env: proposer_factory,
                client,
                pool: transaction_pool,
                select_chain,
                consensus_data_provider: None,
                create_inherent_data_providers,
            },
        )),
    };

    // we spawn the future on a background thread managed by service.
    task_manager.spawn_essential_handle().spawn_blocking("manual-seal", None, manual_seal);
    Ok(())
}

type ChainOpsResult = Result<
    (
        Arc<FullClient>,
        Arc<FullBackend>,
        BasicQueue<Block, PrefixedMemoryDB<BlakeTwo256>>,
        TaskManager,
        Arc<MadaraBackend>,
    ),
    ServiceError,
>;

pub fn new_chain_ops(mut config: &mut Configuration) -> ChainOpsResult {
    config.keystore = sc_service::config::KeystoreConfig::InMemory;
    let sc_service::PartialComponents { client, backend, import_queue, task_manager, other, .. } =
        new_partial::<_>(config, build_aura_grandpa_import_queue)?;
    Ok((client, backend, import_queue, task_manager, other.3))
}
