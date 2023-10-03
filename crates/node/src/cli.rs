use std::path::PathBuf;

use mc_data_availability::DaLayer;
use sc_cli::RunCmd;

use crate::constants;

/// Returns the `madara_path` default value ($HOME/.madara) as a string
fn get_default_madara_path() -> String {
    let home_path = std::env::var("HOME").unwrap_or(std::env::var("USERPROFILE").unwrap_or(".".into()));
    format!("{}/.madara", home_path)
}

/// Available Sealing methods.
#[derive(Debug, Copy, Clone, clap::ValueEnum, Default)]
pub enum Sealing {
    // Seal using rpc method.
    #[default]
    Manual,
    // Seal when transaction is executed.
    Instant,
}

/// Available testnets.
#[derive(Debug, Copy, Clone, PartialEq, clap::ValueEnum)]
pub enum Testnet {
    Sharingan,
}

#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    /// Path to the folder where all configuration files and data are stored
    /// base_path will always be overwritten by madara_path
    /// in the case you use the --tmp, the base_path will be changed during the runtime
    #[clap(global = true, long, default_value = get_default_madara_path())]
    pub madara_path: Option<PathBuf>,

    /// Choose sealing method.
    #[clap(global = true, long, value_enum, ignore_case = true)]
    pub sealing: Option<Sealing>,
}

#[derive(Clone, Debug, clap::Args)]
pub struct ExtendedRunCmd {
    #[clap(flatten)]
    pub run_cmd: RunCmd,

    /// Choose a supported DA Layer
    #[clap(long)]
    pub da_layer: Option<DaLayer>,

    /// Load a custom chain-spec from an url
    /// If you want to load a chain spec that is present in your filesystem, use `--chain=<PATH>`
    #[clap(long, conflicts_with = "testnet")]
    pub fetch_chain_spec: Option<String>,

    /// Choose a supported testnet chain which will load some default values
    /// The testnets will allways be fetched when this flag is passed to search for updates
    #[clap(long, conflicts_with = "fetch_chain_spec", conflicts_with = "chain")]
    pub testnet: Option<Testnet>,
}

#[derive(Debug, clap::Args)]
pub struct SetupCmd {
    /// Load a index.json file for downloading assets
    /// The index.json must follow the format of the official index.json
    /// (https://github.com/keep-starknet-strange/madara/blob/main/configs/index.json)
    /// Where the `md5` and `url` fields are optional
    #[clap(long, default_value = constants::DEFAULT_CONFIGS_URL)]
    pub fetch_madara_configs: Option<String>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Sub-commands concerned with benchmarking.
    #[command(subcommand)]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Db meta columns information.
    ChainInfo(sc_cli::ChainInfoCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Key management cli utilities
    #[command(subcommand)]
    Key(sc_cli::KeySubcommand),

    /// Remove the whole chain.
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),

    // Run madara node
    Run(ExtendedRunCmd),

    // Setup madara node
    Setup(SetupCmd),

    /// Try some command against runtime state.
    #[cfg(feature = "try-runtime")]
    TryRuntime(try_runtime_cli::TryRuntimeCmd),

    /// Try some command against runtime state. Note: `try-runtime` feature must be enabled.
    #[cfg(not(feature = "try-runtime"))]
    TryRuntime,
}
