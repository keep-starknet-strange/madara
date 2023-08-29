use std::path::PathBuf;

use sc_cli::RunCmd;

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

    #[clap(flatten)]
    pub run: ExtendedRunCmd,

    /// Choose sealing method.
    #[arg(long, value_enum, ignore_case = true)]
    pub sealing: Option<Sealing>,
}

#[derive(Debug, clap::Args)]
pub struct ExtendedRunCmd {
    #[clap(flatten)]
    pub run_cmd: RunCmd,

    /// Load a custom chain-spec from an url
    #[clap(long)]
    pub chain_spec_url: Option<String>,

    /// Load a custom index.json file for downloading custom assets
    /// Check documentation in
    /// https://github.com/keep-starknet-strange/madara/blob/main/docs/configs.md
    #[clap(long)]
    pub configs_url: Option<String>,

    /// Disable madara default configs:
    /// - Fetching index.json, genesis.json and genesis assets
    /// - Fetching default chain specs
    #[clap(long)]
    pub disable_madara_configs: bool,

    /// Disable automatic url fetching for madara config files
    #[clap(long)]
    pub disable_url_fetch: bool,

    /// Path to the folder where all configuration files and data are stored
    /// base_path will always be overwritten by madara_path
    /// in the case you use the --tmp, the base_path will be changed during the runtime
    #[clap(long, default_value = get_default_madara_path())]
    pub madara_path: Option<PathBuf>,

    /// Choose a supported testnet chain which will load some default values
    /// current supported testnets: sharingan
    #[clap(long)]
    pub testnet: Option<Testnet>,

    /// If the files currently exist in your madara_path, the default behaviour will skip the file
    /// fetching, it's possible to force an update with this flag
    #[clap(long)]
    pub force_update_config: bool,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Key management cli utilities
    #[command(subcommand)]
    Key(sc_cli::KeySubcommand),

    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Remove the whole chain.
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),

    /// Sub-commands concerned with benchmarking.
    #[command(subcommand)]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    /// Try some command against runtime state.
    #[cfg(feature = "try-runtime")]
    TryRuntime(try_runtime_cli::TryRuntimeCmd),

    /// Try some command against runtime state. Note: `try-runtime` feature must be enabled.
    #[cfg(not(feature = "try-runtime"))]
    TryRuntime,

    /// Db meta columns information.
    ChainInfo(sc_cli::ChainInfoCmd),
}
