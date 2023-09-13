use std::collections::HashMap;

use starknet_api::core::{ClassHash, ContractAddress};
use starknet_api::deprecated_contract_class::{
    ContractClassAbiEntry as DeprecatedContractClassAbiEntry,
    EntryPoint as DeprecatedEntryPoint,
    EntryPointType as DeprecatedEntryPointType,
};
use starknet_api::transaction::TransactionHash;
use test_utils::{auto_impl_get_test_instance, get_number_of_variants, GetTestInstance};

use crate::writer::objects::response::{
    DeclareResponse,
    DeployAccountResponse,
    InvokeResponse,
    SuccessfulStarknetErrorCode,
};
use crate::writer::objects::transaction::DeprecatedContractClass;

auto_impl_get_test_instance! {
    pub struct DeprecatedContractClass {
        pub abi: Option<Vec<DeprecatedContractClassAbiEntry>>,
        pub compressed_program: String,
        pub entry_points_by_type: HashMap<DeprecatedEntryPointType, Vec<DeprecatedEntryPoint>>,
    }
    pub struct InvokeResponse {
        pub code: SuccessfulStarknetErrorCode,
        pub transaction_hash: TransactionHash,
    }
    pub struct DeployAccountResponse {
        pub code: SuccessfulStarknetErrorCode,
        pub transaction_hash: TransactionHash,
        pub address: ContractAddress,
    }
    pub struct DeclareResponse {
        pub code: SuccessfulStarknetErrorCode,
        pub transaction_hash: TransactionHash,
        pub class_hash: ClassHash,
    }
    pub enum SuccessfulStarknetErrorCode {
        TransactionReceived = 0,
    }
}
