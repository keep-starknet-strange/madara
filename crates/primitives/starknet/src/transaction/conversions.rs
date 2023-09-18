use alloc::sync::Arc;

use blockifier::execution::contract_class::ContractClass;
use blockifier::transaction::objects::TransactionExecutionResult;
use blockifier::transaction::transactions as btx;
use starknet_api::api_core::Nonce;
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;
use starknet_api::transaction::{EventKey, Fee, TransactionVersion};
use starknet_api::{api_core as stcore, transaction as sttx};

use super::compute_hash::ComputeTransactionHash;
use super::{
    DeclareTransaction, DeclareTransactionV0, DeclareTransactionV1, DeclareTransactionV2, DeployAccountTransaction,
    HandleL1MessageTransaction, InvokeTransaction, InvokeTransactionV0, InvokeTransactionV1,
};
use crate::execution::felt252_wrapper::Felt252Wrapper;
use crate::traits::hash::HasherT;

impl DeclareTransactionV0 {
    fn try_into_executable<H: HasherT>(
        &self,
        chain_id: Felt252Wrapper,
        contract_class: ContractClass,
        is_query: bool,
    ) -> TransactionExecutionResult<btx::DeclareTransaction> {
        let transaction_hash = self.compute_hash::<H>(chain_id, is_query);

        btx::DeclareTransaction::new(
            sttx::DeclareTransaction::V0(sttx::DeclareTransactionV0V1 {
                max_fee: sttx::Fee(self.max_fee),
                signature: vec_of_felt_to_signature(&self.signature),
                nonce: self.nonce.into(),
                class_hash: self.class_hash.into(),
                sender_address: self.sender_address.into(),
            }),
            transaction_hash.into(),
            contract_class,
        )
    }
}

impl DeclareTransactionV1 {
    fn try_into_executable<H: HasherT>(
        &self,
        chain_id: Felt252Wrapper,
        contract_class: ContractClass,
        is_query: bool,
    ) -> TransactionExecutionResult<btx::DeclareTransaction> {
        let transaction_hash = self.compute_hash::<H>(chain_id, is_query);

        btx::DeclareTransaction::new(
            sttx::DeclareTransaction::V1(sttx::DeclareTransactionV0V1 {
                max_fee: sttx::Fee(self.max_fee),
                signature: vec_of_felt_to_signature(&self.signature),
                nonce: self.nonce.into(),
                class_hash: self.class_hash.into(),
                sender_address: self.sender_address.into(),
            }),
            transaction_hash.into(),
            contract_class,
        )
    }
}

impl DeclareTransactionV2 {
    fn try_into_executable<H: HasherT>(
        &self,
        chain_id: Felt252Wrapper,
        contract_class: ContractClass,
        is_query: bool,
    ) -> TransactionExecutionResult<btx::DeclareTransaction> {
        let transaction_hash = self.compute_hash::<H>(chain_id, is_query);

        btx::DeclareTransaction::new(
            sttx::DeclareTransaction::V2(sttx::DeclareTransactionV2 {
                max_fee: sttx::Fee(self.max_fee),
                signature: vec_of_felt_to_signature(&self.signature),
                nonce: self.nonce.into(),
                class_hash: self.class_hash.into(),
                compiled_class_hash: self.compiled_class_hash.into(),
                sender_address: self.sender_address.into(),
            }),
            transaction_hash.into(),
            contract_class,
        )
    }
}

impl DeclareTransaction {
    pub fn try_into_executable<H: HasherT>(
        &self,
        chain_id: Felt252Wrapper,
        contract_class: ContractClass,
        is_query: bool,
    ) -> TransactionExecutionResult<btx::DeclareTransaction> {
        match self {
            DeclareTransaction::V0(tx) => tx.try_into_executable::<H>(chain_id, contract_class, is_query),
            DeclareTransaction::V1(tx) => tx.try_into_executable::<H>(chain_id, contract_class, is_query),
            DeclareTransaction::V2(tx) => tx.try_into_executable::<H>(chain_id, contract_class, is_query),
        }
    }
}

impl InvokeTransactionV0 {
    pub fn into_executable<H: HasherT>(&self, chain_id: Felt252Wrapper, is_query: bool) -> btx::InvokeTransaction {
        let transaction_hash = self.compute_hash::<H>(chain_id, is_query);

        btx::InvokeTransaction {
            tx: sttx::InvokeTransaction::V0(sttx::InvokeTransactionV0 {
                max_fee: sttx::Fee(self.max_fee),
                signature: vec_of_felt_to_signature(&self.signature),
                contract_address: self.contract_address.into(),
                entry_point_selector: self.entry_point_selector.into(),
                calldata: vec_of_felt_to_calldata(&self.calldata),
            }),
            tx_hash: transaction_hash.into(),
        }
    }
}

impl InvokeTransactionV1 {
    pub fn into_executable<H: HasherT>(&self, chain_id: Felt252Wrapper, is_query: bool) -> btx::InvokeTransaction {
        let transaction_hash = self.compute_hash::<H>(chain_id, is_query);

        btx::InvokeTransaction {
            tx: sttx::InvokeTransaction::V1(sttx::InvokeTransactionV1 {
                max_fee: sttx::Fee(self.max_fee),
                signature: vec_of_felt_to_signature(&self.signature),
                nonce: self.nonce.into(),
                calldata: vec_of_felt_to_calldata(&self.calldata),
                sender_address: self.sender_address.into(),
            }),
            tx_hash: transaction_hash.into(),
        }
    }
}

impl InvokeTransaction {
    pub fn into_executable<H: HasherT>(&self, chain_id: Felt252Wrapper, is_query: bool) -> btx::InvokeTransaction {
        match self {
            InvokeTransaction::V0(tx) => tx.into_executable::<H>(chain_id, is_query),
            InvokeTransaction::V1(tx) => tx.into_executable::<H>(chain_id, is_query),
        }
    }
}

impl DeployAccountTransaction {
    pub fn into_executable<H: HasherT>(
        &self,
        chain_id: Felt252Wrapper,
        is_query: bool,
    ) -> btx::DeployAccountTransaction {
        let account_address = self.get_account_address();
        let transaction_hash: Felt252Wrapper =
            self.compute_hash_given_contract_address::<H>(chain_id.into(), account_address, is_query).into();
        let contract_address: Felt252Wrapper = account_address.into();

        btx::DeployAccountTransaction {
            tx: sttx::DeployAccountTransaction {
                max_fee: sttx::Fee(self.max_fee),
                version: sttx::TransactionVersion(StarkFelt::from(1u128)),
                signature: vec_of_felt_to_signature(&self.signature),
                nonce: self.nonce.into(),
                class_hash: self.class_hash.into(),
                contract_address_salt: self.contract_address_salt.into(),
                constructor_calldata: vec_of_felt_to_calldata(&self.constructor_calldata),
            },
            tx_hash: transaction_hash.into(),
            contract_address: contract_address.into(),
        }
    }
}

impl HandleL1MessageTransaction {
    pub fn into_executable<H: HasherT>(
        &self,
        chain_id: Felt252Wrapper,
        paid_fee_on_l1: Fee,
        is_query: bool,
    ) -> btx::L1HandlerTransaction {
        let transaction_hash = self.compute_hash::<H>(chain_id, is_query);

        let tx = sttx::L1HandlerTransaction {
            version: TransactionVersion(StarkFelt::from(0u8)),
            nonce: Nonce(StarkFelt::from(self.nonce)),
            contract_address: self.contract_address.into(),
            entry_point_selector: self.entry_point_selector.into(),
            calldata: vec_of_felt_to_calldata(&self.calldata),
        };

        btx::L1HandlerTransaction { tx, paid_fee_on_l1, tx_hash: transaction_hash.into() }
    }
}

impl From<Felt252Wrapper> for sttx::TransactionHash {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<sttx::TransactionHash> for Felt252Wrapper {
    fn from(value: sttx::TransactionHash) -> Self {
        value.0.into()
    }
}

impl From<Felt252Wrapper> for stcore::Nonce {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<stcore::Nonce> for Felt252Wrapper {
    fn from(value: stcore::Nonce) -> Self {
        value.0.into()
    }
}

impl From<Felt252Wrapper> for stcore::ClassHash {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<stcore::ClassHash> for Felt252Wrapper {
    fn from(value: stcore::ClassHash) -> Self {
        value.0.into()
    }
}

impl From<Felt252Wrapper> for stcore::CompiledClassHash {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<stcore::CompiledClassHash> for Felt252Wrapper {
    fn from(value: stcore::CompiledClassHash) -> Self {
        value.0.into()
    }
}

impl From<Felt252Wrapper> for stcore::PatriciaKey {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<stcore::PatriciaKey> for Felt252Wrapper {
    fn from(value: stcore::PatriciaKey) -> Self {
        value.0.into()
    }
}

impl From<Felt252Wrapper> for stcore::ContractAddress {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<stcore::ContractAddress> for Felt252Wrapper {
    fn from(value: stcore::ContractAddress) -> Self {
        value.0.into()
    }
}

impl From<Felt252Wrapper> for stcore::EntryPointSelector {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<stcore::EntryPointSelector> for Felt252Wrapper {
    fn from(value: stcore::EntryPointSelector) -> Self {
        value.0.into()
    }
}

impl From<Felt252Wrapper> for sttx::ContractAddressSalt {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<sttx::ContractAddressSalt> for Felt252Wrapper {
    fn from(value: sttx::ContractAddressSalt) -> Self {
        value.0.into()
    }
}

impl From<Felt252Wrapper> for StorageKey {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<StorageKey> for Felt252Wrapper {
    fn from(value: StorageKey) -> Self {
        value.0.0.into()
    }
}

impl From<Felt252Wrapper> for TransactionVersion {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<TransactionVersion> for Felt252Wrapper {
    fn from(value: TransactionVersion) -> Self {
        value.0.into()
    }
}

impl From<Felt252Wrapper> for EventKey {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<EventKey> for Felt252Wrapper {
    fn from(value: EventKey) -> Self {
        value.0.into()
    }
}

fn vec_of_felt_to_signature(felts: &[Felt252Wrapper]) -> sttx::TransactionSignature {
    sttx::TransactionSignature(felts.iter().map(|&f| f.into()).collect())
}

fn vec_of_felt_to_calldata(felts: &[Felt252Wrapper]) -> sttx::Calldata {
    sttx::Calldata(Arc::new(felts.iter().map(|&f| f.into()).collect()))
}
