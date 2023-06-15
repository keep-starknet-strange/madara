use frame_benchmarking_cli::{BenchmarkCmd, ExtrinsicFactory, SUBSTRATE_REFERENCE_HARDWARE};
use madara_runtime::Block;
use mc_storage::overrides_handle;
use sc_cli::{ChainSpec, RpcMethods, RuntimeVersion, SubstrateCli};
use sc_executor_common::wasm_runtime::{HeapAllocStrategy, DEFAULT_HEAP_ALLOC_STRATEGY};
use sp_blockchain::HeaderBackend;
use sp_runtime::generic::Era;

use crate::benchmarking::{inherent_benchmark_data, RemarkBuilder};
use crate::cli::{Cli, Subcommand, Testnet};
use crate::{chain_spec, rpc, service};

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "Substrate Node".into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "support.anonymous.an".into()
    }

    fn copyright_start_year() -> i32 {
        2017
    }

    fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(match id {
            "dev" => {
                let enable_manual_seal = self.sealing.map(|_| true);
                Box::new(chain_spec::development_config(enable_manual_seal)?)
            }
            "" | "local" | "madara-local" => Box::new(chain_spec::local_testnet_config()?),
            path => Box::new(chain_spec::ChainSpec::from_json_file(std::path::PathBuf::from(path))?),
        })
    }

    fn native_runtime_version(_: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        &madara_runtime::VERSION
    }
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
    let mut cli = Cli::from_args();

    match &cli.subcommand {
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, import_queue, task_manager, _) = service::new_chain_ops(&mut config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, _, task_manager, _) = service::new_chain_ops(&mut config)?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, _, task_manager, _) = service::new_chain_ops(&mut config)?;
                Ok((cmd.run(client, config.chain_spec), task_manager))
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, import_queue, task_manager, _) = service::new_chain_ops(&mut config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, backend, _, task_manager, _) = service::new_chain_ops(&mut config)?;
                let aux_revert = Box::new(|client, _, blocks| {
                    sc_consensus_grandpa::revert(client, blocks)?;
                    Ok(())
                });
                Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
            })
        }
        Some(Subcommand::Benchmark(cmd)) => {
            let runner = cli.create_runner(cmd)?;

            runner.sync_run(|mut config| {
                // This switch needs to be in the client, since the client decides
                // which sub-commands it wants to support.
                match cmd {
                    BenchmarkCmd::Pallet(cmd) => {
                        if !cfg!(feature = "runtime-benchmarks") {
                            return Err("Runtime benchmarking wasn't enabled when building the node. You can enable \
                                        it with `--features runtime-benchmarks`."
                                .into());
                        }

                        cmd.run::<Block, service::ExecutorDispatch>(config)
                    }
                    BenchmarkCmd::Block(cmd) => {
                        let (client, _, _, _, _) = service::new_chain_ops(&mut config)?;
                        cmd.run(client)
                    }
                    #[cfg(not(feature = "runtime-benchmarks"))]
                    BenchmarkCmd::Storage(_) => {
                        Err("Storage benchmarking can be enabled with `--features runtime-benchmarks`.".into())
                    }
                    #[cfg(feature = "runtime-benchmarks")]
                    BenchmarkCmd::Storage(cmd) => {
                        let (client, backend, _, _, _) = service::new_chain_ops(&mut config)?;
                        let db = backend.expose_db();
                        let storage = backend.expose_storage();

                        cmd.run(config, client, db, storage)
                    }
                    BenchmarkCmd::Overhead(cmd) => {
                        let (client, _, _, _, _) = service::new_chain_ops(&mut config)?;
                        let ext_builder = RemarkBuilder::new(client.clone());

                        cmd.run(config, client, inherent_benchmark_data()?, Vec::new(), &ext_builder)
                    }
                    BenchmarkCmd::Extrinsic(cmd) => {
                        let (client, _, _, _, _) = service::new_chain_ops(&mut config)?;
                        // Register the *Remark* builder.
                        let ext_factory = ExtrinsicFactory(vec![Box::new(RemarkBuilder::new(client.clone()))]);

                        cmd.run(client, inherent_benchmark_data()?, Vec::new(), &ext_factory)
                    }
                    BenchmarkCmd::Machine(cmd) => cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone()),
                }
            })
        }
        #[cfg(feature = "try-runtime")]
        Some(Subcommand::TryRuntime(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                // we don't need any of the components of new_partial, just a runtime, or a task
                // manager to do `async_run`.
                let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
                let task_manager = sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
                    .map_err(|e| sc_cli::Error::Service(sc_service::Error::Prometheus(e)))?;
                Ok((cmd.run::<Block, service::ExecutorDispatch>(config), task_manager))
            })
        }
        #[cfg(not(feature = "try-runtime"))]
        Some(Subcommand::TryRuntime) => Err("TryRuntime wasn't enabled when building the node. You can enable it \
                                             with `--features try-runtime`."
            .into()),
        Some(Subcommand::ChainInfo(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run::<Block>(&config))
        }
        Some(Subcommand::Simnode(cmd)) => {
            let runner = cli.create_runner(&cmd.run.normalize())?;
            let config = runner.config();

            let heap_pages = config
                .default_heap_pages
                .map_or(DEFAULT_HEAP_ALLOC_STRATEGY, |h| HeapAllocStrategy::Static { extra_pages: h as _ });

            let executor = sc_simnode::Executor::builder()
                .with_execution_method(config.wasm_method)
                .with_onchain_heap_alloc_strategy(heap_pages)
                .with_offchain_heap_alloc_strategy(heap_pages)
                .with_max_runtime_instances(config.max_runtime_instances)
                .with_runtime_cache_size(config.runtime_cache_size)
                .build();

            runner.run_node_until_exit(move |config| async move {
                // pass the custom executor along
                let sc_service::PartialComponents {
                    client,
                    backend,
                    task_manager,
                    keystore_container,
                    select_chain,
                    import_queue,
                    transaction_pool,
                    other: (block_import, grandpa_link, telemetry, madara_backend),
                } = service::new_partial::<_, _>(&config, service::build_aura_grandpa_import_queue, executor)?;

                let overrides = overrides_handle(client.clone());
                let starting_block = client.info().best_number;

                let starknet_rpc_params = rpc::StarknetDeps {
                    client: client.clone(),
                    madara_backend: madara_backend.clone(),
                    overrides,
                    sync_service: None,
                    starting_block,
                };

                let rpc_extensions_builder = {
                    let client = client.clone();
                    let pool = transaction_pool.clone();

                    Box::new(move |deny_unsafe, _| {
                        let deps = rpc::FullDeps {
                            client: client.clone(),
                            pool: pool.clone(),
                            deny_unsafe,
                            starknet: starknet_rpc_params.clone(),
                            command_sink: None,
                        };
                        crate::rpc::create_full(deps).map_err(Into::into)
                    })
                };

                let sim_components = sc_service::PartialComponents {
                    client,
                    backend,
                    task_manager,
                    import_queue,
                    keystore_container,
                    select_chain,
                    transaction_pool,
                    other: (block_import, telemetry, grandpa_link),
                };

                // start simnode's subsystems
                let task_manager =
                    sc_simnode::aura::start_simnode::<RuntimeInfo, _, _, _, _, _>(sc_simnode::SimnodeParams {
                        components: sim_components,
                        config,
                        // you'll want this to be set to true so simnode creates
                        // blocks for every transaction that enters the tx pool.
                        // For special cases where you want to manually send
                        // RPC requests before blocks are created, set this to false.
                        instant: true,
                        rpc_builder: rpc_extensions_builder,
                    })
                    .await?;
                Ok(task_manager)
            })
        }
        None => {
            if cli.run.testnet.is_some() {
                let home_path = std::env::var("HOME").unwrap_or(std::env::var("USERPROFILE").unwrap_or(".".into()));
                cli.run.run_cmd.network_params.node_key_params.node_key_file =
                    Some((home_path.clone() + "/.madara/p2p-key.ed25519").into());
                cli.run.run_cmd.shared_params.base_path = Some((home_path.clone() + "/.madara").into());
                if let Some(Testnet::Sharingan) = cli.run.testnet {
                    cli.run.run_cmd.shared_params.chain =
                        Some(home_path + "/.madara/chain-specs/testnet-sharingan-raw.json");
                }

                cli.run.run_cmd.shared_params.dev = true;
                cli.run.run_cmd.rpc_external = true;
                cli.run.run_cmd.rpc_methods = RpcMethods::Unsafe;
            }
            let runner = cli.create_runner(&cli.run.run_cmd)?;
            runner.run_node_until_exit(|config| async move {
                service::new_full(config, cli.sealing).map_err(sc_cli::Error::Service)
            })
        }
    }
}

pub struct RuntimeInfo;

impl sc_simnode::ChainInfo for RuntimeInfo {
    // make sure you pass the opaque::Block here
    type Block = madara_runtime::opaque::Block;
    // the runtime type
    type Runtime = madara_runtime::Runtime;
    // the runtime api
    type RuntimeApi = madara_runtime::RuntimeApi;
    // [`SignedExtra`] for your runtime
    type SignedExtras = madara_runtime::SignedExtra;

    // Initialize the [`SignedExtra`] for your runtime, you'll notice I'm calling a pallet method here
    // in order to read from the runtime storage. This is possible because this method is called in
    // an externalities provided environment. So feel free to read your runtime storage.
    fn signed_extras(from: <Self::Runtime as frame_system::pallet::Config>::AccountId) -> Self::SignedExtras {
        let nonce = frame_system::Pallet::<Self::Runtime>::account_nonce(from);
        (
            frame_system::CheckNonZeroSender::<Self::Runtime>::new(),
            frame_system::CheckSpecVersion::<Self::Runtime>::new(),
            frame_system::CheckTxVersion::<Self::Runtime>::new(),
            frame_system::CheckGenesis::<Self::Runtime>::new(),
            frame_system::CheckEra::<Self::Runtime>::from(Era::Immortal),
            frame_system::CheckNonce::<Self::Runtime>::from(nonce),
            frame_system::CheckWeight::<Self::Runtime>::new(),
        )
    }
}
