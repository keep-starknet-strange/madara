use pallet_starknet::utils;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Configs {
    pub remote_base_path: String,
    pub chain_specs: Vec<File>,
    pub genesis_assets: Vec<File>,
}

#[derive(Deserialize)]
pub struct File {
    pub name: String,
    pub md5: String,
    pub url: Option<String>,
}

pub fn fetch_and_validate_file(remote_base_path: String, file: File, dest_path: String) -> Result<(), String> {
	let force_fetching = true;
    if let Some(url) = file.url {
        utils::fetch_from_url(url, dest_path.clone(), force_fetching)?;
    } else {
        let relative_path =
            dest_path.split("configs/").collect::<Vec<&str>>()[1].split('/').collect::<Vec<&str>>().join("/");
        utils::fetch_from_url(remote_base_path + &relative_path + &file.name, dest_path.clone(), force_fetching)?;
    }

    let file_str = utils::read_file_to_string(dest_path + &file.name)?;
    let digest = md5::compute(file_str.as_bytes());
    let hash = format!("{:x}", digest);
    if hash != file.md5 {
        return Err(format!("File hash mismatch: {} != {}", hash, file.md5));
    }

    Ok(())
}
