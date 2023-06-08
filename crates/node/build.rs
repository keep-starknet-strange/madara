use substrate_build_script_utils::{generate_cargo_keys, rerun_if_git_head_changed};

fn copy_chain_spec() {
    let mut src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    src.push("chain-specs");
    let home_path = std::env::var("HOME").unwrap_or(std::env::var("USERPROFILE").unwrap_or(".".into()));
    let mut dst = std::path::PathBuf::from(home_path);
    dst.push(".madara");
    dst.push("chain-specs");
    std::fs::create_dir_all(&dst).unwrap();
    for file in std::fs::read_dir(src).unwrap() {
        let file = file.unwrap();
        let mut dst = dst.clone();
        dst.push(file.file_name());
        std::fs::copy(file.path(), dst).unwrap();
    }
}

fn main() {
    generate_cargo_keys();

    rerun_if_git_head_changed();

    copy_chain_spec();
}
