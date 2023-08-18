use std::collections::HashMap;
use std::io::BufRead;

use frame_benchmarking_cli::{BenchmarkCmd, ExtrinsicFactory, SUBSTRATE_REFERENCE_HARDWARE};
use madara_runtime::Block;
use sc_cli::{ChainSpec, RpcMethods, RuntimeVersion, SubstrateCli};

use crate::benchmarking::{inherent_benchmark_data, RemarkBuilder};
use crate::cli::{Cli, Subcommand, Testnet, DA_CONFIG_NAME};
use crate::{chain_spec, service};

fn copy_chain_spec(madara_path: String) {
    let mut src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    src.push("chain-specs");
    let mut dst = std::path::PathBuf::from(madara_path);
    dst.push("chain-specs");
    std::fs::create_dir_all(&dst).unwrap();
    for file in std::fs::read_dir(src).unwrap() {
        let file = file.unwrap();
        let mut dst = dst.clone();
        dst.push(file.file_name());
        std::fs::copy(file.path(), dst).unwrap();
    }
}

fn copy_da_config(da_config_path: String, madara_path: String) {
    let src = std::path::PathBuf::from(da_config_path);
    let mut dst = std::path::PathBuf::from(madara_path.clone());
    dst.push(DA_CONFIG_NAME);

    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::copy(src, dst).unwrap();
    println!("Copied da_config file to madara path: {}", madara_path + "/" + DA_CONFIG_NAME);
}

fn get_da_config(madara_path: String) -> Option<HashMap<String, String>> {
    let src = madara_path + "/" + DA_CONFIG_NAME;
    let path = std::path::Path::new(&src);
    if path.exists() {
        println!("DA config file loaded from: {src}");
        let file = std::fs::File::open(&src).unwrap();
        let reader = std::io::BufReader::new(file);
        let mut map = HashMap::new();

        for line in reader.lines().flatten() {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() > 1 {
                map.insert(parts[0].to_string(), parts[1].to_string());
            }
        }
        Some(map)
    } else {
        None
    }
}

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "Madara Node".into()
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
        "madara.zone".into()
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
            // create a reproducible dev environment
            if cli.run.run_cmd.shared_params.dev {
                cli.run.run_cmd.shared_params.dev = false;
                cli.run.run_cmd.shared_params.chain = Some("dev".to_string());

                cli.run.run_cmd.force_authoring = true;
                cli.run.run_cmd.alice = true;

                // we can't set `--rpc-cors=all`, so it needs to be set manually if we want to connect with external
                // hosts
                cli.run.run_cmd.rpc_external = true;
                cli.run.run_cmd.rpc_methods = RpcMethods::Unsafe;
            }

            // alias madara_path <> base_path
            let madara_path = if cli.run.madara_path.is_some() {
                let path = cli.run.madara_path.clone().unwrap().to_str().unwrap().to_string();
                cli.run.run_cmd.shared_params.base_path = Some((path.clone()).into());
                path
            } else if cli.run.run_cmd.shared_params.base_path.is_some() {
                let path = cli.run.run_cmd.shared_params.base_path.clone().unwrap().to_str().unwrap().to_string();
                cli.run.madara_path = Some((path.clone()).into());
                path
            } else {
                let home_path = std::env::var("HOME").unwrap_or(std::env::var("USERPROFILE").unwrap_or(".".into()));
                let path = format!("{}/.madara", home_path);
                cli.run.run_cmd.shared_params.base_path = Some((path.clone()).into());
                cli.run.madara_path = Some((path.clone()).into());
                path
            };

            cli.run.run_cmd.network_params.node_key_params.node_key_file =
                Some((madara_path.clone() + "/p2p-key.ed25519").into());

            if cli.run.testnet.is_some() {
                if let Some(Testnet::Sharingan) = cli.run.testnet {
                    copy_chain_spec(madara_path.clone());
                    cli.run.run_cmd.shared_params.chain =
                        Some(madara_path.clone() + "/chain-specs/testnet-sharingan-raw.json");
                }

                cli.run.run_cmd.rpc_external = true;
                cli.run.run_cmd.rpc_methods = RpcMethods::Unsafe;
            }

            // Copy the DA config to the madara path if passed in cli
            if let Some(da_config_path) = cli.run.da_config_path.clone() {
                copy_da_config(da_config_path, madara_path.clone())
            }
            // Get the DA Config
            let da_config = get_da_config(madara_path);

            let runner = cli.create_runner(&cli.run.run_cmd)?;
            runner.run_node_until_exit(|config| async move {
                service::new_full(config, cli.sealing, da_config).map_err(sc_cli::Error::Service)
            })
        }
    }
}
