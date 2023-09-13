use std::env;
use std::fs::read_to_string;
use std::path::Path;
use std::string::String;

pub fn read_resource_file(path_in_resource_dir: &str) -> String {
    let path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("resources")
        .join(path_in_resource_dir);
    return read_to_string(path.to_str().unwrap()).unwrap();
}
