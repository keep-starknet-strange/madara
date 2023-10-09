use std::path::PathBuf;

use madara_runtime::SealingMode;
use mc_data_availability::DaLayer;
use mc_l1_messages::config::{L1MessagesWorkerConfig, L1MessagesWorkerConfigError};
use sc_cli::{Result, RpcMethods, RunCmd, SubstrateCli};
use sc_service::BasePath;
use serde::{Deserialize, Serialize};

use crate::cli::Cli;
use crate::service;

#[derive(Debug, Clone, clap::Args)]
pub struct L1MessagesParams {
    /// Ethereum Provider (Node) Url
    #[clap(
        long,
        value_hint=clap::ValueHint::Url,
    )]
    pub provider_url: String,

    /// L1 Contract Address
    #[clap(
        long,
        value_hint=clap::ValueHint::Other,
    )]
    pub l1_contract_address: String,
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
    pub config_params: Option<L1MessagesParams>,
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
    #[clap(long, requires = "da_conf")]
    pub da_layer: Option<DaLayer>,

    #[clap(long, requires = "da_layer")]
    pub da_conf: Option<String>,

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
    pub fn base_path(&self) -> Result<BasePath> {
        Ok(self
            .base
            .shared_params
            .base_path()?
            .unwrap_or_else(|| BasePath::from_project("", "", &<Cli as SubstrateCli>::executable_name())))
    }
}

pub fn run_node(mut cli: Cli) -> Result<()> {
    if cli.run.base.shared_params.dev {
        override_dev_environment(&mut cli.run);
    }
    let runner = cli.create_runner(&cli.run.base)?;

    let da_config: Option<(DaLayer, PathBuf)> = match cli.run.da_layer {
        Some(da_layer) => {
            let da_conf = PathBuf::from(cli.run.da_conf.expect("clap requires da_conf when da_layer is present"));
            if !da_conf.exists() {
                log::info!("{} does not contain DA config", da_conf.display());
                return Err("DA config not available".into());
            }

            Some((da_layer, da_conf))
        }
        None => {
            log::info!("Madara initialized w/o DA layer");
            None
        }
    };

    let l1_messages_worker_config = extract_l1_messages_worker_config(&cli.run.l1_messages_worker)
        .map_err(|e| sc_cli::Error::Input(e.to_string()))?;

    runner.run_node_until_exit(|config| async move {
        let sealing = cli.run.sealing.map(Into::into).unwrap_or_default();
        let cache = cli.run.cache;
        service::new_full(config, sealing, da_config, cache, l1_messages_worker_config).map_err(sc_cli::Error::Service)
    })
}

fn extract_l1_messages_worker_config(
    run_cmd: &L1Messages,
) -> std::result::Result<Option<L1MessagesWorkerConfig>, L1MessagesWorkerConfigError> {
    if let Some(ref config_path) = run_cmd.l1_messages_config {
        let config = L1MessagesWorkerConfig::new_from_file(config_path)?;
        Ok(Some(config))
    } else if let Some(ref config_params) = run_cmd.config_params {
        let config =
            L1MessagesWorkerConfig::new_from_params(&config_params.provider_url, &config_params.l1_contract_address)?;
        Ok(Some(config))
    } else {
        Ok(None)
    }
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
