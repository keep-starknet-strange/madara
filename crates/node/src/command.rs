use std::path::PathBuf;

use frame_benchmarking_cli::{BenchmarkCmd, ExtrinsicFactory, SUBSTRATE_REFERENCE_HARDWARE};
use madara_runtime::Block;
use mc_data_availability::DaLayer;
use pallet_starknet::utils;
use sc_cli::{ChainSpec, RpcMethods, RuntimeVersion, SubstrateCli};

use crate::benchmarking::{inherent_benchmark_data, RemarkBuilder};
use crate::cli::{Cli, Subcommand, Testnet};
use crate::{chain_spec, configs, constants, service};
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
                let enable_manual_seal = self.run.sealing.map(|_| true);
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

fn set_dev_environment(cli: &mut Cli) {
    println!("Setting up dev environment");
    // create a reproducible dev environment
    cli.run.run_cmd.shared_params.dev = false;
    cli.run.run_cmd.shared_params.chain = Some("dev".to_string());

    cli.run.run_cmd.force_authoring = true;
    cli.run.run_cmd.alice = true;

    // we can't set `--rpc-cors=all`, so it needs to be set manually if we want to connect with external
    // hosts
    cli.run.run_cmd.rpc_external = true;
    cli.run.run_cmd.rpc_methods = RpcMethods::Unsafe;
}

fn set_testnet(cli: &mut Cli, testnet: Testnet) -> Result<(), String> {
    // checks if it should retrieve and enable a specific chain-spec
    let madara_path = cli.run.madara_path.clone().expect("Failed retrieving madara_path").to_str().unwrap().to_string();
    let local_path = utils::get_project_path();

    if testnet == Testnet::Sharingan {
        if let Ok(ref src_path) = local_path {
            let src_path = src_path.clone() + "/configs/chain-specs/testnet-sharingan-raw.json";
            utils::copy_from_filesystem(src_path, madara_path.clone() + "/chain-specs")?;
            cli.run.run_cmd.shared_params.chain = Some(madara_path + "/chain-specs/testnet-sharingan-raw.json");
        } else {
            utils::fetch_from_url(
                constants::SHARINGAN_CHAIN_SPEC_URL.to_string(),
                madara_path.clone() + "/configs/chain-specs/",
            )?;
            cli.run.run_cmd.shared_params.chain = Some(madara_path + "/chain-specs/testnet-sharingan-raw.json");
        }
    }

    if cli.run.run_cmd.shared_params.chain.is_some() {
        cli.run.run_cmd.rpc_external = true;
        cli.run.run_cmd.rpc_methods = RpcMethods::Unsafe;
    }

    Ok(())
}

fn set_chain_spec(cli: &mut Cli, chain_spec_url: String) -> Result<(), String> {
    let madara_path = cli.run.madara_path.clone().expect("Failed retrieving madara_path").to_str().unwrap().to_string();

    utils::fetch_from_url(chain_spec_url.clone(), madara_path.clone() + "/chain-specs")?;
    let chain_spec = chain_spec_url.split('/').last().expect("Chain spec file name not found");
    cli.run.run_cmd.shared_params.chain = Some(madara_path + "/chain-specs/" + chain_spec);

    Ok(())
}

fn fetch_madara_configs(cli: &Cli) -> Result<(), String> {
    let madara_path = cli.run.madara_path.clone().expect("Failed retrieving madara_path").to_str().unwrap().to_string();
    let local_path = utils::get_project_path();

    if let Ok(ref src_path) = local_path {
        let index_path = src_path.clone() + "/configs/index.json";
        utils::copy_from_filesystem(index_path, madara_path.clone() + "/configs")?;

        let madara_configs: configs::Configs =
            serde_json::from_str(&utils::read_file_to_string(madara_path.clone() + "/configs/index.json")?)
                .expect("Failed to load madara configs");
        for asset in madara_configs.genesis_assets {
            let src_path = src_path.clone() + "/configs/genesis-assets/" + &asset.name;
            utils::copy_from_filesystem(src_path, madara_path.clone() + "/configs/genesis-assets")?;
        }
    } else if let Some(configs_url) = &cli.setup.fetch_madara_configs {
        utils::fetch_from_url(configs_url.to_string(), madara_path.clone() + "/configs")?;

        let madara_configs: configs::Configs =
            serde_json::from_str(&utils::read_file_to_string(madara_path.clone() + "/configs/index.json")?)
                .expect("Failed to load madara configs");

        for asset in madara_configs.genesis_assets {
            configs::fetch_and_validate_file(
                madara_configs.remote_base_path.clone(),
                asset,
                madara_path.clone() + "/configs/genesis-assets/",
            )?;
        }
    }

    Ok(())
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
    let mut cli = Cli::from_args();

    cli.run.run_cmd.shared_params.base_path = cli.run.madara_path.clone();

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
        Some(Subcommand::Run(cmd)) => {
            let madara_path =
                cli.run.madara_path.clone().expect("Failed retrieving madara_path").to_str().unwrap().to_string();

            // Set the node_key_file for substrate in the case that it was not manually setted
            if cmd.run_cmd.network_params.node_key_params.node_key_file.is_none() {
                cli.run.run_cmd.network_params.node_key_params.node_key_file =
                    Some((madara_path.clone() + "/p2p-key.ed25519").into());
            }

            if cmd.run_cmd.shared_params.dev {
                set_dev_environment(&mut cli);
            }

            if let Some(chain_spec_url) = cli.run.fetch_chain_spec.clone() {
                set_chain_spec(&mut cli, chain_spec_url)?;
            }

            if let Some(testnet) = cli.run.testnet {
                set_testnet(&mut cli, testnet)?;
            }

            let da_config: Option<(DaLayer, PathBuf)> = match cli.run.da_layer {
                Some(da_layer) => {
                    let da_path = std::path::PathBuf::from(madara_path.clone() + "/da-config.json");
                    if !da_path.exists() {
                        log::info!("{} does not contain DA config", madara_path);
                        return Err("DA config not available".into());
                    }

                    Some((da_layer, da_path))
                }
                None => {
                    log::info!("madara initialized w/o da layer");
                    None
                }
            };

            let runner = cli.create_runner(&cli.run.run_cmd)?;
            runner.run_node_until_exit(|config| async move {
                service::new_full(config, cli.run.sealing, da_config).map_err(sc_cli::Error::Service)
            })
        }
        Some(Subcommand::Setup(_)) => {
            fetch_madara_configs(&cli)?;
            Ok(())
        }
        _ => Err("You need to specify some subcommand. E.g. `madara run`".into()),
    }
}
