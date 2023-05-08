use blockifier::abi::abi_utils::selector_from_name;
use lazy_static::lazy_static;
use starknet_api::api_core::EntryPointSelector;

/// validate entry point name
pub const VALIDATE_ENTRY_POINT_NAME: &str = "__validate__";
/// validate declare entry point name
pub const VALIDATE_DECLARE_ENTRY_POINT_NAME: &str = "__validate_declare__";
/// validate deploy entry point name
pub const VALIDATE_DEPLOY_ENTRY_POINT_NAME: &str = "__validate_deploy__";

lazy_static! {
    /// validate entry point selector
    pub static ref VALIDATE_ENTRY_POINT_SELECTOR: EntryPointSelector = selector_from_name(VALIDATE_ENTRY_POINT_NAME);
    /// validate declare entry point selector
    pub static ref VALIDATE_DECLARE_ENTRY_POINT_SELECTOR: EntryPointSelector = selector_from_name(VALIDATE_DECLARE_ENTRY_POINT_NAME);
    /// validate deploy entry point selector
    pub static ref VALIDATE_DEPLOY_ENTRY_POINT_SELECTOR: EntryPointSelector = selector_from_name(VALIDATE_DEPLOY_ENTRY_POINT_NAME);
}
