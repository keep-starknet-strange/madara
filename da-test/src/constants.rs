use std::path::PathBuf;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref ETHEREUM_DA_CONFIG: PathBuf = PathBuf::from("../examples/da-confs/ethereum.json");
    pub static ref CELESTIA_DA_CONFIG: PathBuf = PathBuf::from("../examples/da-confs/celestia.json");
    pub static ref AVAIL_DA_CONFIG: PathBuf = PathBuf::from("../examples/da-confs/avail.json");
}
