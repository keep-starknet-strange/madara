pub fn get_project_path() -> Result<String, Box<dyn std::error::Error>> {
    let workspace = std::process::Command::new(env!("CARGO"))
        .args(["locate-project", "--workspace", "--message-format=plain"])
        .output();

    if workspace.is_err() {
        return Err("Failed to get project path".into());
    }

    let mut dir = std::path::PathBuf::from(std::str::from_utf8(&workspace?.stdout)?.trim());
    dir.pop();
    Ok(dir.to_str().ok_or("Failed to get project path")?.to_string())
}

pub fn copy_from_filesystem(src_path: String, dest_path: String) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Trying to copy {} to {} from filesystem", src_path, dest_path);
    let src = std::path::PathBuf::from(src_path.clone());
    if !src.exists() {
        log::info!("{} does not exist", src_path);
        return Err("File does not exist".into());
    }

    let mut dst = std::path::PathBuf::from(dest_path.clone());
    std::fs::create_dir_all(&dst)?;
    dst.push(src.file_name().ok_or("File name not found")?);
    std::fs::copy(src, dst)?;

    log::info!("Copied {} to {} from filesystem", src_path, dest_path);
    Ok(())
}

pub fn fetch_from_url(target: String, dest_path: String) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Trying to fetch {} to {} from url", target, dest_path);
    let dst = std::path::PathBuf::from(dest_path);
    std::fs::create_dir_all(&dst)?;

    let response = reqwest::blocking::get(target.clone());
    if response.is_err() {
        log::info!("Failed to fetch {} from url", target);
        return Err("Failed to fetch from url".into());
    }

    let file = std::fs::File::create(dst.join(target.split('/').last().ok_or("File name not found")?));
    if file.is_err() {
        log::info!("Failed to create file {} from url", target);
        return Err("Failed to create file".into());
    }

    let bytes = response?.bytes();
    if bytes.is_err() {
        log::info!("Failed to get bytes from {} from url", target);
        return Err("Failed to get bytes from url".into());
    }

    let mut content = std::io::Cursor::new(bytes?);
    std::io::copy(&mut content, &mut file?)?;

    Ok(())
}

pub fn read_file_to_string(path: String) -> Result<String, Box<dyn std::error::Error>> {
    Ok(std::fs::read_to_string(path)?)
}
