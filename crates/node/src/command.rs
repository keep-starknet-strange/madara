use std::path::PathBuf;

use frame_benchmarking_cli::{BenchmarkCmd, ExtrinsicFactory, SUBSTRATE_REFERENCE_HARDWARE};
use madara_runtime::Block;
use mc_data_availability::DaLayer;
use pallet_starknet::utils;
use sc_cli::{ChainSpec, RpcMethods, RuntimeVersion, SubstrateCli};

use crate::benchmarking::{inherent_benchmark_data, RemarkBuilder};
use crate::cli::{Cli, Subcommand, Testnet};
use crate::{chain_spec, constants, service};
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
                Box::new(chain_spec::development_config(
                    enable_manual_seal,
                    self.run.madara_path.clone().expect("Failed retrieving madara_path"),
                )?)
            }
            "" | "local" | "madara-local" => Box::new(chain_spec::local_testnet_config(
                self.run.madara_path.clone().expect("Failed retrieving madara_path"),
            )?),
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

    // alias madara_path <> base_path
    // TODO also alias tmp (tmp generates random base_paths that are not specified within
    // the command)
    let madara_path = match (cli.run.madara_path.clone(), cli.run.run_cmd.shared_params.base_path.clone()) {
        (Some(madara_path), _) => {
            cli.run.run_cmd.shared_params.base_path = Some(madara_path.clone());
            madara_path.to_str().unwrap().to_string()
        }
        (_, Some(base_path)) => {
            cli.run.madara_path = Some(base_path.clone());
            base_path.to_str().unwrap().to_string()
        }
        _ => {
            let home_path = std::env::var("HOME").unwrap_or(std::env::var("USERPROFILE").unwrap_or(".".into()));
            let path = format!("{}/.madara", home_path);
            cli.run.run_cmd.shared_params.base_path = Some((path.clone()).into());
            cli.run.madara_path = Some((path.clone()).into());
            path
        }
    };

    if let Some(genesis_url) = cli.run.genesis_url.clone() {
        // can't copy extra genesis-assets atm
        // we can reuse #982 to create the standard to fetch relevant files
        utils::fetch_from_url(genesis_url, madara_path.clone() + "/configs/genesis-assets")?;
    } else {
        // TODO confirm with the CI that we are fetching all and fetch dynamically
        // Issue #982
        for file in constants::GENESIS_ASSETS_FILES {
            let src_path = utils::get_project_path();
            if let Ok(src_path) = src_path {
                let src_path = src_path + "/configs/genesis-assets/" + file;
                utils::copy_from_filesystem(src_path, madara_path.clone() + "/genesis-assets")?;
            } else {
                utils::fetch_from_url(
                    constants::GENESIS_ASSETS_URL.to_string() + file,
                    madara_path.clone() + "/genesis-assets",
                )?;
            }
        }
    }

    // TODO confirm with the CI that we are fetching all and fetch dynamically
    // Issue #982
    for file in constants::CAIRO_CONTRACTS_FILES {
        let src_path = utils::get_project_path();
        if let Ok(src_path) = src_path {
            let src_path = src_path + "/configs/cairo-contracts/" + file;
            utils::copy_from_filesystem(src_path, madara_path.clone() + "/cairo-contracts")?;
        } else {
            utils::fetch_from_url(
                constants::CAIRO_CONTRACTS_URL.to_string() + file,
                madara_path.clone() + "/cairo-contracts",
            )?;
        }
    }

    if let (Some(chain_spec_url), None) = (cli.run.chain_spec_url.clone(), cli.run.testnet) {
        utils::fetch_from_url(chain_spec_url, madara_path.clone() + "/chain-specs")?;
    }

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

            cli.run.run_cmd.network_params.node_key_params.node_key_file =
                Some((madara_path.clone() + "/p2p-key.ed25519").into());

            if let Some(Testnet::Sharingan) = cli.run.testnet {
                let src_path = utils::get_project_path();
                if let Ok(src_path) = src_path {
                    let src_path = src_path + "/configs/chain-specs/testnet-sharingan-raw.json";
                    utils::copy_from_filesystem(src_path, madara_path.clone() + "/chain-specs")?;
                } else {
                    utils::fetch_from_url(
                        constants::SHARINGAN_CHAIN_SPEC_URL.to_string(),
                        madara_path.clone() + "/chain-specs",
                    )?;
                }

                cli.run.run_cmd.shared_params.chain =
                    Some(madara_path.clone() + "/chain-specs/testnet-sharingan-raw.json");

                // This should go apply to all testnets when applying a match pattern
                cli.run.run_cmd.rpc_external = true;
                cli.run.run_cmd.rpc_methods = RpcMethods::Unsafe;
            }

            let mut da_config: Option<(DaLayer, PathBuf)> = None;
            if let Some(da_layer) = cli.run.da_layer.clone() {
                let da_path = std::path::PathBuf::from(madara_path.clone() + "/da-config.json");
                if !da_path.exists() {
                    log::info!("{} does not contain DA config", madara_path.clone());
                    return Err("DA config not available".into());
                }

                da_config = Some((da_layer, da_path));
            }

            let runner = cli.create_runner(&cli.run.run_cmd)?;
            runner.run_node_until_exit(|config| async move {
                service::new_full(config, cli.sealing, da_config).map_err(sc_cli::Error::Service)
            })
        }
    }
}
