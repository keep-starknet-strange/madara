use test_utils::validate_load_and_dump;

use super::{DeclareResponse, DeployAccountResponse, InvokeResponse};

#[test]
fn load_and_dump_deploy_account_same_string() {
    validate_load_and_dump::<DeployAccountResponse>("writer/deploy_account_response.json");
}

#[test]
fn load_and_dump_invoke_same_string() {
    validate_load_and_dump::<InvokeResponse>("writer/invoke_response.json");
}

#[test]
fn load_and_dump_declare_same_string() {
    validate_load_and_dump::<DeclareResponse>("writer/declare_response.json");
}
