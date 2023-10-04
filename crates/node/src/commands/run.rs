use std::path::PathBuf;

use mc_data_availability::DaLayer;
use sc_cli::{Result, RpcMethods, RunCmd, SubstrateCli};

use crate::cli::Cli;
use crate::service;

/// Available Sealing methods.
#[derive(Debug, Copy, Clone, clap::ValueEnum, Default)]
pub enum Sealing {
    // Seal using rpc method.
    #[default]
    Manual,
    // Seal when transaction is executed.
    Instant,
}

#[derive(Clone, Debug, clap::Args)]
pub struct ExtendedRunCmd {
    #[clap(flatten)]
    pub base: RunCmd,

    /// Choose sealing method.
    #[clap(long, value_enum, ignore_case = true)]
    pub sealing: Option<Sealing>,

    /// Choose a supported DA Layer
    #[clap(long)]
    pub da_layer: Option<DaLayer>,
}

pub fn run_node(mut cli: Cli) -> Result<()> {
    if cli.run.base.shared_params.dev {
        override_dev_environment(&mut cli.run);
    }
    let runner = cli.create_runner(&cli.run.base)?;
    let data_path = &runner.config().data_path;

    let da_config: Option<(DaLayer, PathBuf)> = match cli.run.da_layer {
        Some(da_layer) => {
            let da_path = data_path.join("da-config.json");
            if !da_path.exists() {
                log::info!("{} does not contain DA config", da_path.display());
                return Err("DA config not available".into());
            }

            Some((da_layer, da_path))
        }
        None => {
            log::info!("Madara initialized w/o DA layer");
            None
        }
    };
    let sealing = cli.run.sealing;

    runner.run_node_until_exit(|config| async move {
        service::new_full(config, sealing, da_config).map_err(sc_cli::Error::Service)
    })
}

fn override_dev_environment(cmd: &mut ExtendedRunCmd) {
    // create a reproducible dev environment
    cmd.base.shared_params.dev = false;
    cmd.base.shared_params.chain = Some("dev".to_string());

    cmd.base.force_authoring = true;
    cmd.base.alice = true;

    // we can't set `--rpc-cors=all`, so it needs to be set manually if we want to connect with external
    // hosts
    cmd.base.rpc_external = true;
    cmd.base.rpc_methods = RpcMethods::Unsafe;
}
