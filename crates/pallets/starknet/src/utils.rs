pub fn get_project_path() -> String {
    let workspace = std::process::Command::new(env!("CARGO"))
        .args(["locate-project", "--workspace", "--message-format=plain"])
        .output();

    if workspace.is_err() {
        return "".to_string();
    }

    let mut dir = std::path::PathBuf::from(std::str::from_utf8(&workspace.unwrap().stdout).unwrap().trim());
    dir.pop();
    dir.to_str().unwrap().to_string()
}

pub fn copy_from_filesystem(src_path: String, dest_path: String) -> bool {
    log::info!("Trying to copy {} to {} from filesystem", src_path.clone(), dest_path.clone());
    let src = std::path::PathBuf::from(src_path.clone());
    if !src.exists() {
        log::info!("{} does not exist", src_path.clone());
        return false;
    }

    let mut dst = std::path::PathBuf::from(dest_path.clone());
    std::fs::create_dir_all(&dst).unwrap();
    dst.push(src.file_name().unwrap());
    std::fs::copy(src, dst).unwrap();

    log::info!("Copied {} to {} from filesystem", src_path, dest_path);
    return true;
}

pub fn fetch_from_url(target: String, dest_path: String) -> bool {
    log::info!("Trying to fetch {} to {} from url", target.clone(), dest_path.clone());
    let dst = std::path::PathBuf::from(dest_path);
    std::fs::create_dir_all(&dst).unwrap();

    let response = reqwest::blocking::get(target.clone());
    if response.is_err() {
        log::info!("Failed to fetch {} from url", target.clone());
        return false;
    }

    let file = std::fs::File::create(dst.join(target.split('/').last().unwrap()));
    if file.is_err() {
        log::info!("Failed to create file {} from url", target.clone());
        return false;
    }

    let bytes = response.unwrap().bytes();
    if bytes.is_err() {
        log::info!("Failed to get bytes from {} from url", target);
        return false;
    }

    let mut content = std::io::Cursor::new(bytes.unwrap());
    std::io::copy(&mut content, &mut file.unwrap()).unwrap();

    return true;
}

pub fn read_file_to_string(path: String) -> String {
    std::fs::read_to_string(path).unwrap()
}
