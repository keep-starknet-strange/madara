use std::fs::File;
use std::path::{Path, PathBuf};

use clap::ValueHint::FilePath;
use madara_runtime::SealingMode;
#[cfg(feature = "avail")]
use mc_data_availability::avail::{config::AvailConfig, AvailClient};
#[cfg(feature = "celestia")]
use mc_data_availability::celestia::{config::CelestiaConfig, CelestiaClient};
use mc_data_availability::ethereum::config::EthereumConfig;
use mc_data_availability::ethereum::EthereumClient;
use mc_data_availability::{DaClient, DaLayer};
use mc_l1_messages::config::{L1MessagesWorkerConfig, L1MessagesWorkerConfigError};
use mc_settlement::SettlementLayer;
use sc_cli::{Result, RpcMethods, RunCmd, SubstrateCli};
use sc_service::BasePath;
use serde::{Deserialize, Serialize};

use crate::cli::Cli;
use crate::service;

#[derive(Debug, Clone, clap::Args)]
#[group(multiple = true)]
pub struct L1MessagesParams {
    /// Ethereum Provider (Node) Url
    #[clap(
        long,
        value_hint=clap::ValueHint::Url,
        conflicts_with="l1_messages_config",
        requires="l1_contract_address",
    )]
    pub provider_url: Option<String>,

    /// L1 Contract Address
    #[clap(
        long,
        value_hint=clap::ValueHint::Other,
        conflicts_with="l1_messages_config",
        requires="provider_url",
    )]
    pub l1_contract_address: Option<String>,
}

#[derive(Debug, Clone, clap::Args)]
pub struct L1Messages {
    /// Path to configuration file for Ethereum Core Contract Events Listener
    #[clap(
        long,
        conflicts_with_all=["provider_url", "l1_contract_address"],
        value_hint=clap::ValueHint::FilePath,
    )]
    pub l1_messages_config: Option<PathBuf>,

    #[clap(flatten)]
    pub config_params: L1MessagesParams,
}

/// Available Sealing methods.
#[derive(Debug, Copy, Clone, clap::ValueEnum, Default, Serialize, Deserialize)]
pub enum Sealing {
    /// Seal using rpc method.
    #[default]
    Manual,
    /// Seal when transaction is executed. This mode does not finalize blocks, if you want to
    /// finalize blocks use `--sealing=instant-finality`.
    Instant,
    /// Seal when transaction is executed with finalization.
    InstantFinality,
}

impl From<Sealing> for SealingMode {
    fn from(value: Sealing) -> Self {
        match value {
            Sealing::Manual => SealingMode::Manual,
            Sealing::Instant => SealingMode::Instant { finalize: false },
            Sealing::InstantFinality => SealingMode::Instant { finalize: true },
        }
    }
}

#[derive(Clone, Debug, clap::Args)]
pub struct ExtendedRunCmd {
    #[clap(flatten)]
    pub base: RunCmd,

    /// Choose sealing method.
    #[clap(long, value_enum, ignore_case = true)]
    pub sealing: Option<Sealing>,

    /// Choose a supported DA Layer
    #[clap(long, ignore_case = true)]
    pub da_layer: Option<DaLayer>,

    /// Path to a file containing the DA configuration
    ///
    /// If `da_layer` is `Some` and `da_conf` is `None` we will try to read one at
    /// `<chain_config_directory>/<da_layer_name>.json`. If it's not there, an error will be
    /// returned.
    #[clap(long, value_hint = FilePath, requires = "da_layer")]
    pub da_conf: Option<PathBuf>,

    /// Choose a supported settlement layer
    #[clap(long, ignore_case = true)]
    pub settlement: Option<SettlementLayer>,

    /// Path to a file containing the settlement configuration
    ///
    /// If `settlement` is `Some` and `settlement_conf` is `None` we will try to read one at
    /// `<chain_config_directory>/settlement_conf.json`. If it's not there, an error will be
    /// returned.
    #[clap(long, value_hint = FilePath, requires = "settlement")]
    pub settlement_conf: Option<PathBuf>,

    /// When enabled, more information about the blocks and their transaction is cached and stored
    /// in the database.
    ///
    /// This may improve response times for RPCs that require that information, but it also
    /// increases the memory footprint of the node.
    #[clap(long)]
    pub cache: bool,

    /// Configuration for L1 Messages (Syncing) Worker
    #[clap(flatten)]
    pub l1_messages_worker: L1Messages,
}

impl ExtendedRunCmd {
    /// The substrate base directory on your machine
    ///
    /// Will be different depending on your OS
    pub fn base_path(&self) -> Result<BasePath> {
        Ok(self
            .base
            .shared_params
            .base_path()?
            .unwrap_or_else(|| BasePath::from_project("", "", &<Cli as SubstrateCli>::executable_name())))
    }

    /// The chain name
    ///
    /// Will use `""` (empty sting) if none is provided
    fn chain_id(&self) -> &str {
        match &self.base.shared_params.chain {
            Some(s) => s,
            None => "",
        }
    }

    /// The path of the configuration folder of your chain
    ///
    /// "<base_path>/chains/<my_chain_id>"
    fn chain_config_dir(&self) -> Result<PathBuf> {
        let chain_id = self.chain_id();
        let chain_config_dir = self.base_path()?.config_dir(chain_id);
        Ok(chain_config_dir)
    }
}

fn init_da_client(da_layer: DaLayer, da_path: &Path) -> Result<Box<dyn DaClient + Send + Sync>> {
    let file = File::open(da_path)?;

    let da_client: Box<dyn DaClient + Send + Sync> = match da_layer {
        #[cfg(feature = "celestia")]
        DaLayer::Celestia => {
            let celestia_conf: CelestiaConfig =
                serde_json::from_reader(file).map_err(|e| sc_cli::Error::Input(e.to_string()))?;
            Box::new(CelestiaClient::try_from(celestia_conf).map_err(|e| sc_cli::Error::Input(e.to_string()))?)
        }
        DaLayer::Ethereum => {
            let ethereum_conf: EthereumConfig =
                serde_json::from_reader(file).map_err(|e| sc_cli::Error::Input(e.to_string()))?;
            Box::new(EthereumClient::try_from(ethereum_conf).map_err(|e| sc_cli::Error::Input(e.to_string()))?)
        }
        #[cfg(feature = "avail")]
        DaLayer::Avail => {
            let avail_conf: AvailConfig =
                serde_json::from_reader(file).map_err(|e| sc_cli::Error::Input(e.to_string()))?;
            Box::new(AvailClient::try_from(avail_conf).map_err(|e| sc_cli::Error::Input(e.to_string()))?)
        }
    };

    Ok(da_client)
}

pub fn run_node(mut cli: Cli) -> Result<()> {
    if cli.run.base.shared_params.dev {
        override_dev_environment(&mut cli.run);
    }
    let runner = cli.create_runner(&cli.run.base)?;

    let chain_config_dir = cli.run.chain_config_dir()?;

    let da_client = match cli.run.da_layer {
        Some(da_layer) => {
            let da_conf = match cli.run.clone().da_conf {
                Some(da_conf) => da_conf,
                None => {
                    let path_da_conf_json = chain_config_dir.join(format!("{}.json", da_layer));
                    if !path_da_conf_json.exists() {
                        return Err(sc_cli::Error::Input(format!(
                            "no file {} in base_path",
                            path_da_conf_json.to_string_lossy()
                        )));
                    }
                    path_da_conf_json
                }
            };

            log::info!("Initializing DA client with layer: {:?}", da_layer);
            Some(init_da_client(da_layer, &da_conf)?)
        }
        None => {
            log::info!("Madara initialized w/o DA layer");
            None
        }
    };

    let l1_messages_worker_config = extract_l1_messages_worker_config(&cli.run.l1_messages_worker)
        .map_err(|e| sc_cli::Error::Input(e.to_string()))?;

    let settlement_config: Option<(SettlementLayer, PathBuf)> = match cli.run.settlement {
        Some(SettlementLayer::Ethereum) => {
            let settlement_conf = match cli.run.clone().settlement_conf {
                Some(settlement_conf) => settlement_conf,
                None => {
                    let path_settlement_conf_json = chain_config_dir.join("settlement_conf.json");
                    if !path_settlement_conf_json.exists() {
                        return Err(sc_cli::Error::Input(format!(
                            "no file {} in base_path",
                            path_settlement_conf_json.to_string_lossy()
                        )));
                    }
                    path_settlement_conf_json
                }
            };

            log::info!("Initializing settlement client with layer: {:?}", SettlementLayer::Ethereum);
            Some((SettlementLayer::Ethereum, settlement_conf))
        }

        None => {
            log::info!("Madara initialized w/o settlement layer");
            None
        }
    };

    runner.run_node_until_exit(|config| async move {
        let sealing = cli.run.sealing.map(Into::into).unwrap_or_default();
        let cache = cli.run.cache;
        service::new_full(config, sealing, da_client, cache, l1_messages_worker_config, settlement_config)
            .map_err(sc_cli::Error::Service)
    })
}

fn extract_l1_messages_worker_config(
    run_cmd: &L1Messages,
) -> std::result::Result<Option<L1MessagesWorkerConfig>, L1MessagesWorkerConfigError> {
    if let Some(ref config_path) = run_cmd.l1_messages_config {
        let config = L1MessagesWorkerConfig::new_from_file(config_path)?;
        return Ok(Some(config));
    }

    if let L1MessagesParams { provider_url: Some(url), l1_contract_address: Some(address) } = &run_cmd.config_params {
        let config = L1MessagesWorkerConfig::new_from_params(url, address)?;
        return Ok(Some(config));
    }

    Ok(None)
}

fn override_dev_environment(cmd: &mut ExtendedRunCmd) {
    // create a reproducible dev environment
    // by disabling the default substrate `dev` behaviour
    cmd.base.shared_params.dev = false;
    cmd.base.shared_params.chain = Some("dev".to_string());

    cmd.base.force_authoring = true;
    cmd.base.alice = true;
    cmd.base.tmp = true;

    // we can't set `--rpc-cors=all`, so it needs to be set manually if we want to connect with external
    // hosts
    cmd.base.rpc_external = true;
    cmd.base.rpc_methods = RpcMethods::Unsafe;
}
