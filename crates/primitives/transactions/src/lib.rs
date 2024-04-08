//! Starknet transaction related functionality.

#![feature(trait_upcasting)]
#[doc(hidden)]
pub extern crate alloc;

pub mod compute_hash;
pub mod execution;
// pub mod conversions;
#[cfg(feature = "client")]
pub mod from_broadcasted_transactions;
// pub mod getters;
#[cfg(feature = "client")]
pub mod to_starknet_core_transaction;
// #[cfg(feature = "client")]
// pub mod utils;

use alloc::vec::Vec;

use blockifier::execution::contract_class::ContractClass;
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::transaction_execution::Transaction;
use blockifier::transaction::transaction_types::TransactionType;
use derive_more::From;
use sp_core::H256;
use starknet_api::transaction::{Fee, TransactionHash};
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

pub fn get_transaction_hash(tx: &Transaction) -> &TransactionHash {
    match tx {
        Transaction::AccountTransaction(tx) => get_account_transaction_hash(tx),
        Transaction::L1HandlerTransaction(tx) => &tx.tx_hash,
    }
}

pub fn get_account_transaction_hash(tx: &AccountTransaction) -> &TransactionHash {
    match tx {
        AccountTransaction::Invoke(tx) => &tx.tx_hash,
        AccountTransaction::Declare(tx) => &tx.tx_hash,
        AccountTransaction::DeployAccount(tx) => &tx.tx_hash,
    }
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

// Adapted from pathfinder
pub fn compute_message_hash(tx: &starknet_api::transaction::L1HandlerTransaction) -> H256 {
    use sha3::{Digest, Keccak256};

    let Some((from_address, payload)) = tx.calldata.0.split_first() else {
        // This would indicate a pretty severe error in the L1 transaction.
        // But since we haven't encoded this during serialization, this could in
        // theory mess us up here.
        //
        // We should incorporate this into the deserialization instead. Returning an
        // error here is unergonomic and far too late.
        return H256::zero();
    };

    let mut hash = Keccak256::new();

    // In the folowing lines we are abusing the fact that the internal representation of a StarkFelt is
    // an big endian array of bytes [u8; 32] This is an ethereum address
    // Should this internal representation change (and it will!!!) this would break
    // TODO: add a test so it fails when the inner repr changes
    hash.update(from_address.0);
    hash.update(tx.contract_address.0.0.0);
    hash.update(tx.nonce.0.0);
    hash.update(tx.entry_point_selector.0.0);

    // Pad the u64 to 32 bytes to match a felt.
    hash.update([0u8; 24]);
    hash.update((payload.len() as u64).to_be_bytes());

    for elem in payload {
        hash.update(elem.0);
    }

    let hash = <[u8; 32]>::from(hash.finalize());

    hash.into()
}

// impl From<TxType> for TransactionType {
//     fn from(value: TxType) -> Self {
//         match value {
//             TxType::Invoke => TransactionType::InvokeFunction,
//             TxType::Declare => TransactionType::Declare,
//             TxType::DeployAccount => TransactionType::DeployAccount,
//             TxType::L1Handler => TransactionType::L1Handler,
//         }
//     }
// }

impl From<&AccountTransaction> for TxType {
    fn from(value: &AccountTransaction) -> Self {
        match value {
            AccountTransaction::Declare(_) => TxType::Declare,
            AccountTransaction::DeployAccount(_) => TxType::DeployAccount,
            AccountTransaction::Invoke(_) => TxType::Invoke,
        }
    }
}

impl From<&Transaction> for TxType {
    fn from(value: &Transaction) -> Self {
        match value {
            Transaction::AccountTransaction(tx) => tx.into(),
            Transaction::L1HandlerTransaction(_) => TxType::L1Handler,
        }
    }
}

// #[derive(Clone, Debug, Eq, PartialEq, From)]
// #[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode,
// parity_scale_codec::Decode))] #[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
// pub enum UserTransaction {
//     Declare(DeclareTransaction, ContractClass),
//     DeployAccount(DeployAccountTransaction),
//     Invoke(InvokeTransaction),
// }

// #[derive(Clone, Debug, Eq, PartialEq, From)]
// #[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode,
// parity_scale_codec::Decode))] #[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
// pub enum Transaction {
//     Declare(DeclareTransaction, ContractClass),
//     DeployAccount(DeployAccountTransaction),
//     Invoke(InvokeTransaction),
//     L1Handler(HandleL1MessageTransaction),
// }

// #[derive(Clone, Debug, Eq, PartialEq, From)]
// #[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode,
// parity_scale_codec::Decode))] #[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
// pub enum UserOrL1HandlerTransaction {
//     User(UserTransaction),
//     L1Handler(HandleL1MessageTransaction, Fee),
// }

// #[derive(Debug, Clone, Eq, PartialEq, From)]
// #[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode,
// parity_scale_codec::Decode))] #[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
// pub enum InvokeTransaction {
//     V1(InvokeTransactionV1),
// }

// #[derive(Debug, Clone, Eq, PartialEq)]
// #[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode,
// parity_scale_codec::Decode))] #[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
// pub struct InvokeTransactionV1 {
//     pub max_fee: u128,
//     pub signature: Vec<Felt252Wrapper>,
//     pub nonce: Felt252Wrapper,
//     pub sender_address: Felt252Wrapper,
//     pub calldata: Vec<Felt252Wrapper>,
//     pub offset_version: bool,
// }

// #[derive(Debug, Clone, Eq, PartialEq, From)]
// #[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode,
// parity_scale_codec::Decode))] #[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
// pub enum DeclareTransaction {
//     V0(DeclareTransactionV0),
//     V1(DeclareTransactionV1),
//     V2(DeclareTransactionV2),
// }

// #[derive(Debug, Clone, Eq, PartialEq)]
// #[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode,
// parity_scale_codec::Decode))] #[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
// pub struct DeclareTransactionV0 {
//     pub max_fee: u128,
//     pub signature: Vec<Felt252Wrapper>,
//     pub nonce: Felt252Wrapper,
//     pub class_hash: Felt252Wrapper,
//     pub sender_address: Felt252Wrapper,
// }

// #[derive(Debug, Clone, Eq, PartialEq)]
// #[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode,
// parity_scale_codec::Decode))] #[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
// pub struct DeclareTransactionV1 {
//     pub max_fee: u128,
//     pub signature: Vec<Felt252Wrapper>,
//     pub nonce: Felt252Wrapper,
//     pub class_hash: Felt252Wrapper,
//     pub sender_address: Felt252Wrapper,
//     pub offset_version: bool,
// }

// #[derive(Debug, Clone, Eq, PartialEq)]
// #[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode,
// parity_scale_codec::Decode))] #[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
// pub struct DeclareTransactionV2 {
//     pub max_fee: u128,
//     pub signature: Vec<Felt252Wrapper>,
//     pub nonce: Felt252Wrapper,
//     pub class_hash: Felt252Wrapper,
//     pub sender_address: Felt252Wrapper,
//     pub compiled_class_hash: Felt252Wrapper,
//     pub offset_version: bool,
// }

// #[derive(Debug, Clone, Eq, PartialEq)]
// #[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode,
// parity_scale_codec::Decode))] #[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
// pub struct DeployAccountTransaction {
//     pub max_fee: u128,
//     pub signature: Vec<Felt252Wrapper>,
//     pub nonce: Felt252Wrapper,
//     pub contract_address_salt: Felt252Wrapper,
//     pub constructor_calldata: Vec<Felt252Wrapper>,
//     pub class_hash: Felt252Wrapper,
//     pub offset_version: bool,
// }

// #[derive(Debug, Clone, Eq, PartialEq)]
// #[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode,
// parity_scale_codec::Decode))] #[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
// pub struct HandleL1MessageTransaction {
//     pub nonce: u64,
//     pub contract_address: Felt252Wrapper,
//     pub entry_point_selector: Felt252Wrapper,
//     pub calldata: Vec<Felt252Wrapper>,
// }

// impl From<MsgFromL1> for HandleL1MessageTransaction {
//     fn from(msg: MsgFromL1) -> Self {
//         let calldata =
//             std::iter::once(msg.from_address.into()).chain(msg.payload.into_iter().map(|felt|
// felt.into())).collect();

//         Self {
//             contract_address: msg.to_address.into(),
//             nonce: 0u32.into(),
//             entry_point_selector: msg.entry_point_selector.into(),
//             calldata,
//         }
//     }
// }
