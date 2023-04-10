use std::path::PathBuf;

use madara_runtime::opaque::Block;
use sc_service::{BasePath, Configuration};

pub type MadaraBackend = mc_db::Backend<Block>;

pub fn db_config_dir(config: &Configuration) -> PathBuf {
    let application = &config.impl_name;
    config
        .base_path
        .as_ref()
        .map(|base_path| base_path.config_dir(config.chain_spec.id()))
        .unwrap_or_else(|| BasePath::from_project("", "", application).config_dir(config.chain_spec.id()))
}
