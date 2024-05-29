use std::path::PathBuf;

use clap::ValueHint::FilePath;
use madara_runtime::SealingMode;
use sc_cli::{Result, RpcMethods, RunCmd, SubstrateCli};
use sc_service::BasePath;
use serde::{Deserialize, Serialize};

use crate::cli::Cli;
use crate::service;

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

#[derive(Debug, Copy, Clone, PartialEq, clap::ValueEnum)]
pub enum SettlementLayer {
    /// Use Ethereum core contract
    Ethereum,
}

#[derive(Clone, Debug, clap::Args)]
pub struct ExtendedRunCmd {
    #[clap(flatten)]
    pub base: RunCmd,

    /// Choose sealing method.
    #[clap(long, value_enum, ignore_case = true)]
    pub sealing: Option<Sealing>,

    /// Choose a supported settlement layer
    #[clap(long, ignore_case = true, requires = "settlement_conf")]
    pub settlement: Option<SettlementLayer>,

    /// Path to a file containing the settlement configuration
    ///
    /// If `settlement` is `Some` and `settlement_conf` is `None` we will try to read one at
    /// `<chain_config_directory>/settlement_conf.json`. If it's not there, an error will be
    /// returned.
    #[clap(long, value_hint = FilePath, requires = "settlement")]
    pub settlement_conf: Option<PathBuf>,
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
}

pub fn run_node(mut cli: Cli) -> Result<()> {
    if cli.run.base.shared_params.dev {
        override_dev_environment(&mut cli.run);
    }
    let runner = cli.create_runner(&cli.run.base)?;

    let settlement_config: Option<(SettlementLayer, PathBuf)> = match cli.run.settlement {
        Some(SettlementLayer::Ethereum) => {
            let settlement_conf = match cli.run.clone().settlement_conf {
                Some(settlement_conf) => settlement_conf,
                None => panic!("Settlement layer Ethereum requires a settlement configuration"),
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
        service::new_full(config, sealing, settlement_config).map_err(sc_cli::Error::Service)
    })
}

fn override_dev_environment(cmd: &mut ExtendedRunCmd) {
    // create a reproducible dev environment
    // by disabling the default substrate `dev` behaviour
    cmd.base.shared_params.dev = false;
    cmd.base.shared_params.chain = Some("dev".to_string());

    cmd.base.force_authoring = true;
    cmd.base.alice = true;

    if cmd.base.shared_params.base_path.is_none() {
        cmd.base.tmp = true;
    }

    // we can't set `--rpc-cors=all`, so it needs to be set manually if we want to connect with external
    // hosts
    cmd.base.rpc_external = true;
    cmd.base.rpc_methods = RpcMethods::Unsafe;
}
