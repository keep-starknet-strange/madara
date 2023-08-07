

pub fn copy_chain_spec(madara_path: String) {
    let mut src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    src.push("chain-specs");
    let mut dst = std::path::PathBuf::from(madara_path);
    dst.push("chain-specs");
    std::fs::create_dir_all(&dst).unwrap();
    for file in std::fs::read_dir(src).unwrap() {
        let file = file.unwrap();
        let mut dst = dst.clone();
        dst.push(file.file_name());
        std::fs::copy(file.path(), dst).unwrap();
    }
}

pub fn read_file_to_string(path: &str) -> String {
    let workspace = std::process::Command::new(env!("CARGO"))
        .args(["locate-project", "--workspace", "--message-format=plain"])
        .output()
        .expect("Failed to execute cargo locate-project command")
        .stdout;
    let mut dir = std::path::PathBuf::from(std::str::from_utf8(&workspace).unwrap().trim());
    dir.pop();
    dir.push(path);
    std::fs::read_to_string(dir).unwrap()
}

pub fn copy_from_filesystem(src_path: String, dest_path: String) {
    let mut src = std::path::PathBuf::from(src_path);
    let mut dst = std::path::PathBuf::from(dest_path);
    std::fs::create_dir_all(&dst).unwrap();
    for file in std::fs::read_dir(src).unwrap() {
        let file = file.unwrap();
        let mut dst = dst.clone();
        dst.push(file.file_name());
        std::fs::copy(file.path(), dst).unwrap();
    }
}

pub fn fetch_from_url(target: String, dest_path: String) -> Result<(), Box<dyn std::error::Error>> {
    let dst = std::path::PathBuf::from(dest_path);
    std::fs::create_dir_all(&dst).unwrap();

    let response = reqwest::blocking::get(target.clone())?;
    let mut file = std::fs::File::create(dst.join(target.split('/').last().unwrap()))?;
    let mut content = std::io::Cursor::new(response.bytes()?);
    std::io::copy(&mut content, &mut file)?;

    Ok(())
}
