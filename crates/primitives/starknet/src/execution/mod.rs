//! Starknet execution functionality.

mod call_entrypoint_wrapper;
mod contract_class_wrapper;
mod entrypoint_wrapper;

/// All the types related to the execution of a transaction.
pub mod types {
    /// Type wrapper for a contract address.
    pub type ContractAddressWrapper = [u8; 32];

    /// Wrapper type for class hash field.
    pub type ClassHashWrapper = [u8; 32];
    pub use super::call_entrypoint_wrapper::*;
    pub use super::contract_class_wrapper::*;
    pub use super::entrypoint_wrapper::*;
}
