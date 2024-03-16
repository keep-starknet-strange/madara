//! Starknet transaction related functionality.

#![feature(trait_upcasting)]
#[doc(hidden)]
pub extern crate alloc;

pub mod compute_hash;
pub mod execution;
#[cfg(feature = "client")]
pub mod from_broadcasted_transactions;
#[cfg(feature = "client")]
pub mod to_starknet_core_transaction;

use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::transaction_execution::Transaction;
use sp_core::H256;
use starknet_api::core::{ContractAddress, Nonce, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::TransactionHash;
use starknet_core::types::{TransactionExecutionStatus, TransactionFinalityStatus};
use starknet_ff::FieldElement;

const SIMULATE_TX_VERSION_OFFSET: FieldElement =
    FieldElement::from_mont([18446744073700081665, 17407, 18446744073709551584, 576460752142434320]);

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

pub fn get_transaction_nonce(tx: &Transaction) -> Nonce {
    match tx {
        Transaction::AccountTransaction(tx) => match tx {
            AccountTransaction::Declare(tx) => tx.tx.nonce(),
            AccountTransaction::DeployAccount(tx) => tx.tx.nonce(),
            AccountTransaction::Invoke(tx) => tx.tx.nonce(),
        },
        Transaction::L1HandlerTransaction(tx) => tx.tx.nonce,
    }
}

pub fn get_transaction_sender_address(tx: &Transaction) -> ContractAddress {
    match tx {
        Transaction::AccountTransaction(tx) => match tx {
            AccountTransaction::Declare(tx) => tx.tx.sender_address(),
            AccountTransaction::DeployAccount(tx) => tx.contract_address,
            AccountTransaction::Invoke(tx) => tx.tx.sender_address(),
        },
        Transaction::L1HandlerTransaction(_) => ContractAddress(PatriciaKey(StarkFelt::ZERO)),
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
