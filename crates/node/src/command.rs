use frame_benchmarking_cli::{BenchmarkCmd, ExtrinsicFactory, SUBSTRATE_REFERENCE_HARDWARE};
use madara_runtime::Block;
use sc_cli::{ChainSpec, RpcMethods, RuntimeVersion, SubstrateCli};

use crate::benchmarking::{inherent_benchmark_data, RemarkBuilder};
use crate::cli::{Cli, Subcommand, Testnet};
use crate::{chain_spec, service};

fn copy_chain_spec() {
    let mut src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    src.push("chain-specs");
    let home_path = std::env::var("HOME").unwrap_or(std::env::var("USERPROFILE").unwrap_or(".".into()));
    let mut dst = std::path::PathBuf::from(home_path);
    dst.push(".madara");
    dst.push("chain-specs");
    std::fs::create_dir_all(&dst).unwrap();
    for file in std::fs::read_dir(src).unwrap() {
        let file = file.unwrap();
        let mut dst = dst.clone();
        dst.push(file.file_name());
        std::fs::copy(file.path(), dst).unwrap();
    }
}

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
        None => {
            if cli.run.testnet.is_some() {
                copy_chain_spec();

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
