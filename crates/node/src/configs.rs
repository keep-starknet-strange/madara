use serde::Deserialize;

#[derive(Deserialize)]
pub struct Configs {
    pub remote_base_path: String,
    pub genesis_assets: Vec<FileInfos>,
}

#[derive(Deserialize)]
pub struct FileInfos {
    pub name: String,
    pub sha3_256: Option<String>,
    pub url: Option<String>,
}
