#[derive(Debug)]
pub enum Error {
    Cli(sc_cli::Error),
}

impl From<Error> for sc_cli::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Cli(err) => err,
        }
    }
}

impl From<Error> for String {
    fn from(err: Error) -> Self {
        match err {
            Error::Cli(err) => err.to_string(),
        }
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        Error::Cli(sc_cli::Error::Input(err.to_string()))
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Cli(sc_cli::Error::Io(err))
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Cli(sc_cli::Error::Input(err.to_string()))
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::Cli(sc_cli::Error::Input(err.to_string()))
    }
}

impl From<core::str::Utf8Error> for Error {
    fn from(err: core::str::Utf8Error) -> Self {
        Error::Cli(sc_cli::Error::Input(err.to_string()))
    }
}

pub fn get_project_path() -> Result<String, Error> {
    let workspace = std::process::Command::new(env!("CARGO"))
        .args(["locate-project", "--workspace", "--message-format=plain"])
        .output()?;

    let mut dir = std::path::PathBuf::from(std::str::from_utf8(&workspace.stdout)?.trim());
    dir.pop();
    Ok(dir.to_str().ok_or("Failed to get project path")?.to_string())
}

pub fn copy_from_filesystem(src_path: String, dest_path: String) -> Result<(), Error> {
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

pub fn fetch_from_url(target: String, dest_path: String) -> Result<(), Error> {
    log::info!("Trying to fetch {} to {} from url", target, dest_path);
    let mut dst = std::path::PathBuf::from(dest_path);
    std::fs::create_dir_all(&dst)?;
    dst.push(target.split('/').last().expect("Failed to get file name from `target` while fetching url"));

    let response = reqwest::blocking::get(target.clone())?;

    let mut file = std::fs::File::create(dst)?;
    let bytes = response.bytes()?;

    let mut content = std::io::Cursor::new(bytes);
    std::io::copy(&mut content, &mut file)?;

    Ok(())
}

pub fn read_file_to_string(path: String) -> Result<String, Error> {
    Ok(std::fs::read_to_string(path)?)
}
