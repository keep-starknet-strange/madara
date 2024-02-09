//! Starknet transaction related functionality.
#![cfg_attr(not(feature = "std"), no_std)]

#[doc(hidden)]
pub extern crate alloc;

pub mod compute_hash;
pub mod conversions;
pub mod execution;
#[cfg(feature = "client")]
pub mod from_broadcasted_transactions;
pub mod getters;
#[cfg(feature = "client")]
pub mod to_starknet_core_transaction;
#[cfg(feature = "client")]
pub mod utils;

use alloc::vec::Vec;

use blockifier::execution::contract_class::ContractClass;
use blockifier::transaction::transaction_types::TransactionType;
use derive_more::From;
use starknet_api::transaction::Fee;
use starknet_core::types::{MsgFromL1, TransactionExecutionStatus, TransactionFinalityStatus};
use starknet_ff::FieldElement;

const SIMULATE_TX_VERSION_OFFSET: FieldElement =
    FieldElement::from_mont([18446744073700081665, 17407, 18446744073709551584, 576460752142434320]);

/// Functions related to transaction conversions
// pub mod utils;
use mp_felt::Felt252Wrapper;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransactionStatus {
    pub finality_status: TransactionFinalityStatus,
    pub execution_status: TransactionExecutionStatus,
}

/// Wrapper type for transaction execution error.
/// Different tx types.
/// See `https://docs.starknet.io/documentation/architecture_and_concepts/Blocks/transactions/` for more details.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub enum TxType {
    /// Regular invoke transaction.
    Invoke,
    /// Declare transaction.
    Declare,
    /// Deploy account transaction.
    DeployAccount,
    /// Message sent from ethereum.
    L1Handler,
}

impl From<TxType> for TransactionType {
    fn from(value: TxType) -> Self {
        match value {
            TxType::Invoke => TransactionType::InvokeFunction,
            TxType::Declare => TransactionType::Declare,
            TxType::DeployAccount => TransactionType::DeployAccount,
            TxType::L1Handler => TransactionType::L1Handler,
        }
    }
}

impl From<&UserTransaction> for TxType {
    fn from(value: &UserTransaction) -> Self {
        match value {
            UserTransaction::Declare(_, _) => TxType::Declare,
            UserTransaction::DeployAccount(_) => TxType::DeployAccount,
            UserTransaction::Invoke(_) => TxType::Invoke,
        }
    }
}

impl From<&UserOrL1HandlerTransaction> for TxType {
    fn from(value: &UserOrL1HandlerTransaction) -> Self {
        match value {
            UserOrL1HandlerTransaction::User(tx) => tx.into(),
            UserOrL1HandlerTransaction::L1Handler(_, _) => TxType::L1Handler,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, From)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub enum UserTransaction {
    Declare(DeclareTransaction, ContractClass),
    DeployAccount(DeployAccountTransaction),
    Invoke(InvokeTransaction),
}

#[derive(Clone, Debug, Eq, PartialEq, From)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub enum Transaction {
    Declare(DeclareTransaction),
    DeployAccount(DeployAccountTransaction),
    Invoke(InvokeTransaction),
    L1Handler(HandleL1MessageTransaction),
}

#[derive(Clone, Debug, Eq, PartialEq, From)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub enum UserOrL1HandlerTransaction {
    User(UserTransaction),
    L1Handler(HandleL1MessageTransaction, Fee),
}

#[derive(Debug, Clone, Eq, PartialEq, From)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub enum InvokeTransaction {
    V0(InvokeTransactionV0),
    V1(InvokeTransactionV1),
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct InvokeTransactionV0 {
    pub max_fee: u128,
    pub signature: Vec<Felt252Wrapper>,
    pub contract_address: Felt252Wrapper,
    pub entry_point_selector: Felt252Wrapper,
    pub calldata: Vec<Felt252Wrapper>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct InvokeTransactionV1 {
    pub max_fee: u128,
    pub signature: Vec<Felt252Wrapper>,
    pub nonce: Felt252Wrapper,
    pub sender_address: Felt252Wrapper,
    pub calldata: Vec<Felt252Wrapper>,
    pub offset_version: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, From)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub enum DeclareTransaction {
    V0(DeclareTransactionV0),
    V1(DeclareTransactionV1),
    V2(DeclareTransactionV2),
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct DeclareTransactionV0 {
    pub max_fee: u128,
    pub signature: Vec<Felt252Wrapper>,
    pub nonce: Felt252Wrapper,
    pub class_hash: Felt252Wrapper,
    pub sender_address: Felt252Wrapper,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct DeclareTransactionV1 {
    pub max_fee: u128,
    pub signature: Vec<Felt252Wrapper>,
    pub nonce: Felt252Wrapper,
    pub class_hash: Felt252Wrapper,
    pub sender_address: Felt252Wrapper,
    pub offset_version: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct DeclareTransactionV2 {
    pub max_fee: u128,
    pub signature: Vec<Felt252Wrapper>,
    pub nonce: Felt252Wrapper,
    pub class_hash: Felt252Wrapper,
    pub sender_address: Felt252Wrapper,
    pub compiled_class_hash: Felt252Wrapper,
    pub offset_version: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct DeployAccountTransaction {
    pub max_fee: u128,
    pub signature: Vec<Felt252Wrapper>,
    pub nonce: Felt252Wrapper,
    pub contract_address_salt: Felt252Wrapper,
    pub constructor_calldata: Vec<Felt252Wrapper>,
    pub class_hash: Felt252Wrapper,
    pub offset_version: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct HandleL1MessageTransaction {
    pub nonce: u64,
    pub contract_address: Felt252Wrapper,
    pub entry_point_selector: Felt252Wrapper,
    pub calldata: Vec<Felt252Wrapper>,
}

impl From<MsgFromL1> for HandleL1MessageTransaction {
    fn from(msg: MsgFromL1) -> Self {
        let calldata = msg.payload.into_iter().map(|felt| felt.into()).collect();
        Self {
            contract_address: msg.to_address.into(),
            nonce: 0u32.into(),
            entry_point_selector: msg.entry_point_selector.into(),
            calldata,
        }
    }
}
