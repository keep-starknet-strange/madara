use celestia_types::nmt::Namespace;
use eyre::Result;

pub fn string_to_namespace(input: &str) -> Result<Namespace> {
    // Convert the input string to bytes
    let bytes = input.as_bytes();

    // Create a new Namespace from these bytes
    let namespace = Namespace::new_v0(bytes)?;

    Ok(namespace)
}
