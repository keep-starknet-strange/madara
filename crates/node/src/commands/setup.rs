use std::path::{Path, PathBuf};

use sc_cli::{Error, Result, SubstrateCli};
use sc_service::BasePath;
use url::Url;

use crate::chain_spec::GENESIS_ASSETS_DIR;
use crate::cli::Cli;
use crate::configs::FileInfos;
use crate::{configs, constants};

/// Define a way to retrieve an index.json file

/// The index.json must follow the format of the official index.json
/// (https://github.com/keep-starknet-strange/madara/blob/main/configs/index.json)
/// Where the `md5` and `url` fields are optional
#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub struct SetupSource {
    /// Download an index.json file for an url
    #[clap(
        long,
        conflicts_with="from_local",
        value_hint=clap::ValueHint::Url,
        // This combination of properties allow us to use a default value only if the arg is passed without value.
        // If it is not passed at all, no default value is used.
        // See: https://docs.rs/clap/latest/clap/struct.Arg.html#method.default_missing_value
        num_args = 0..=1, require_equals=true, default_missing_value = Some(constants::DEFAULT_CONFIGS_URL)
    )]
    pub from_remote: Option<String>,

    /// Copy an index.json file for an url
    #[clap(long, conflicts_with = "from_remote")]
    pub from_local: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct SetupCmd {
    #[arg(long, value_name = "CHAIN_SPEC")]
    pub chain: Option<String>,

    /// Specify custom base path.
    #[arg(long, short = 'd', value_name = "PATH")]
    pub base_path: Option<PathBuf>,

    #[clap(flatten)]
    pub source: SetupSource,
}

impl SetupCmd {
    fn chain_id(&self) -> String {
        match self.chain {
            Some(ref chain) => chain.clone(),
            None => "".into(),
        }
    }

    fn base_path(&self) -> Option<BasePath> {
        self.base_path.as_ref().map(|bp| BasePath::from(bp.clone()))
    }
}

impl SetupCmd {
    pub fn run(&self) -> Result<()> {
        log::info!("setup cmd: {:?}", self);
        let dest_config_dir_path = {
            let chain_id = self.chain_id();
            let base_path = self.base_path().unwrap_or_else(|| BasePath::from_project("", "", &Cli::executable_name()));
            base_path.config_dir(&chain_id)
        };
        log::info!("Setting up madara config at '{}'", dest_config_dir_path.display());

        // Load config from disk
        if let Some(src_configs_dir_path) = &self.source.from_local {
            // Build the source file path
            let src_configs_dir_path = PathBuf::from(src_configs_dir_path);
            let index_file_path = src_configs_dir_path.join("index.json");

            // Read, deserialize and copy it
            let madara_configs = {
                let src_file_content =
                    std::fs::read_to_string(index_file_path).map_err(|e| Error::Application(Box::new(e)))?;
                // Make sure content is valid before writing it to disk
                let configs_content: configs::Configs = serde_json::from_str(&src_file_content)
                    .map_err(|e| Error::Input(format!("invalid `index.json` content: {}", e)))?;
                write_content_to_disk(src_file_content, dest_config_dir_path.join("index.json").as_path())?;

                configs_content
            };

            // Copy each asset
            for asset in madara_configs.genesis_assets {
                copy_file(
                    &src_configs_dir_path.join(GENESIS_ASSETS_DIR).join(asset.name),
                    &dest_config_dir_path.join(GENESIS_ASSETS_DIR),
                )?;
            }
        // Load config form a remote server
        } else if let Some(configs_url) = &self.source.from_remote {
            // Build the source url
            let configs_url = Url::parse(configs_url)
                .map_err(|e| Error::Input(format!("invalid input for 'fetch_madara_configs': {}", e)))?;
            println!("Fetching chain config from '{}'", &configs_url);

            // Query, deserialize and copy it
            let madara_configs = {
                let response = reqwest::blocking::get(configs_url).map_err(|e| Error::Application(Box::new(e)))?;
                let bytes = response.bytes().map_err(|e| Error::Application(Box::new(e)))?;
                // Make sure content is valid before writing it to disk
                let configs_content: configs::Configs =
                    serde_json::from_slice(&bytes[..]).map_err(|e| Error::Application(Box::new(e)))?;
                write_content_to_disk(bytes, dest_config_dir_path.join("index.json").as_path())?;

                configs_content
            };

            // Query and copy each asset
            let base_url = Url::parse(&madara_configs.remote_base_path).map_err(|e| Error::Application(Box::new(e)))?;
            for asset in madara_configs.genesis_assets {
                fetch_and_validate_genesis_assets(&base_url, asset, &dest_config_dir_path)?;
            }
        } else {
            unreachable!(
                "clap::Args is derived upon `SetupSource` in a way that guarantee that either `from_remote` or \
                 `from_local` is present"
            );
        }

        Ok(())
    }
}

fn write_content_to_disk<T: AsRef<[u8]>>(config_content: T, dest_config_file_path: &Path) -> Result<()> {
    std::fs::create_dir_all(
        dest_config_file_path.parent().expect("dest_config_file_path should be the path to a file, not a dict"),
    )?;
    let mut dest_file = std::fs::File::create(dest_config_file_path)?;
    let mut reader = std::io::Cursor::new(config_content);
    std::io::copy(&mut reader, &mut dest_file)?;

    Ok(())
}

fn copy_file(src_path: &Path, dest_dir_path: &PathBuf) -> Result<()> {
    if !src_path.exists() {
        return Err(format!("Source file '{}' does not exist", src_path.display()).into());
    }

    std::fs::create_dir_all(dest_dir_path)?;
    let dest_file_path = dest_dir_path.join(src_path.file_name().ok_or("File name not found")?);
    std::fs::copy(src_path, dest_file_path)?;

    Ok(())
}

fn fetch_and_validate_genesis_assets(base_remote_url: &Url, file: FileInfos, base_path: &Path) -> Result<()> {
    let full_url = base_remote_url
        .join(GENESIS_ASSETS_DIR)
        .map_err(|e| Error::Application(Box::new(e)))?
        .join(&file.name)
        .map_err(|e| Error::Application(Box::new(e)))?;
    println!("Fetching '{}'", &full_url);
    let dest_path = base_path.join(GENESIS_ASSETS_DIR);

    // Copy
    let file_as_bytes = {
        let response = reqwest::blocking::get(full_url.clone()).map_err(|e| Error::Application(Box::new(e)))?;
        let bytes = response.bytes().map_err(|e| Error::Application(Box::new(e)))?;
        write_content_to_disk(&bytes, &dest_path.join(file.name))?;
        bytes
    };

    if let Some(file_hash) = file.md5 {
        let digest = md5::compute(file_as_bytes);
        let hash = format!("{:x}", digest);
        if hash != file_hash {
            return Err(Error::Input(format!("Hash mismatch for file '{}': {} != {}", full_url, hash, file_hash)));
        }
    }

    Ok(())
}
