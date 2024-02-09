use alloc::vec::Vec;

use mp_felt::Felt252Wrapper;
use mp_hashers::HasherT;
use starknet_core::crypto::compute_hash_on_elements;
use starknet_crypto::FieldElement;

use super::{
    DeclareTransaction, DeclareTransactionV0, DeclareTransactionV1, DeclareTransactionV2, DeployAccountTransaction,
    HandleL1MessageTransaction, InvokeTransaction, InvokeTransactionV0, InvokeTransactionV1, Transaction,
    UserTransaction, SIMULATE_TX_VERSION_OFFSET,
};
use crate::UserOrL1HandlerTransaction;

const DECLARE_PREFIX: &[u8] = b"declare";
const DEPLOY_ACCOUNT_PREFIX: &[u8] = b"deploy_account";
const INVOKE_PREFIX: &[u8] = b"invoke";
const L1_HANDLER_PREFIX: &[u8] = b"l1_handler";

pub trait ComputeTransactionHash {
    fn compute_hash<H: HasherT>(&self, chain_id: Felt252Wrapper, offset_version: bool) -> Felt252Wrapper;
}

fn convert_calldata(data: &[Felt252Wrapper]) -> &[FieldElement] {
    // Non-copy but less dangerous than transmute
    // https://doc.rust-lang.org/std/mem/fn.transmute.html#alternatives
    unsafe { core::slice::from_raw_parts(data.as_ptr() as *const FieldElement, data.len()) }
}

impl ComputeTransactionHash for InvokeTransactionV0 {
    fn compute_hash<H: HasherT>(&self, chain_id: Felt252Wrapper, offset_version: bool) -> Felt252Wrapper {
        let prefix = FieldElement::from_byte_slice_be(INVOKE_PREFIX).unwrap();
        let version = if offset_version { SIMULATE_TX_VERSION_OFFSET } else { FieldElement::ZERO };
        let contract_address = self.contract_address.into();
        let entrypoint_selector = self.entry_point_selector.into();
        let calldata_hash = compute_hash_on_elements(convert_calldata(&self.calldata));
        let max_fee = FieldElement::from(self.max_fee);
        let chain_id = chain_id.into();

        H::compute_hash_on_elements(&[
            prefix,
            version,
            contract_address,
            entrypoint_selector,
            calldata_hash,
            max_fee,
            chain_id,
        ])
        .into()
    }
}

impl ComputeTransactionHash for InvokeTransactionV1 {
    fn compute_hash<H: HasherT>(&self, chain_id: Felt252Wrapper, offset_version: bool) -> Felt252Wrapper {
        let prefix = FieldElement::from_byte_slice_be(INVOKE_PREFIX).unwrap();
        let version = if offset_version { SIMULATE_TX_VERSION_OFFSET + FieldElement::ONE } else { FieldElement::ONE };
        let sender_address = self.sender_address.into();
        let entrypoint_selector = FieldElement::ZERO;
        let calldata_hash = compute_hash_on_elements(convert_calldata(&self.calldata));
        let max_fee = FieldElement::from(self.max_fee);
        let chain_id = chain_id.into();
        let nonce = FieldElement::from(self.nonce);

        H::compute_hash_on_elements(&[
            prefix,
            version,
            sender_address,
            entrypoint_selector,
            calldata_hash,
            max_fee,
            chain_id,
            nonce,
        ])
        .into()
    }
}

impl ComputeTransactionHash for InvokeTransaction {
    fn compute_hash<H: HasherT>(&self, chain_id: Felt252Wrapper, offset_version: bool) -> Felt252Wrapper {
        match self {
            InvokeTransaction::V0(tx) => tx.compute_hash::<H>(chain_id, offset_version),
            InvokeTransaction::V1(tx) => tx.compute_hash::<H>(chain_id, offset_version),
        }
    }
}

impl ComputeTransactionHash for DeclareTransactionV0 {
    fn compute_hash<H: HasherT>(&self, chain_id: Felt252Wrapper, offset_version: bool) -> Felt252Wrapper {
        let prefix = FieldElement::from_byte_slice_be(DECLARE_PREFIX).unwrap();
        let version = if offset_version { SIMULATE_TX_VERSION_OFFSET } else { FieldElement::ZERO };
        let sender_address = self.sender_address.into();
        let entrypoint_selector = FieldElement::ZERO;
        let alignment_placeholder = compute_hash_on_elements(&[]);
        let max_fee = FieldElement::from(self.max_fee);
        let chain_id = chain_id.into();
        let class_hash = self.class_hash.into();

        H::compute_hash_on_elements(&[
            prefix,
            version,
            sender_address,
            entrypoint_selector,
            alignment_placeholder,
            max_fee,
            chain_id,
            class_hash,
        ])
        .into()
    }
}

impl ComputeTransactionHash for DeclareTransactionV1 {
    fn compute_hash<H: HasherT>(&self, chain_id: Felt252Wrapper, offset_version: bool) -> Felt252Wrapper {
        let prefix = FieldElement::from_byte_slice_be(DECLARE_PREFIX).unwrap();
        let version = if offset_version { SIMULATE_TX_VERSION_OFFSET + FieldElement::ONE } else { FieldElement::ONE };
        let sender_address = self.sender_address.into();
        let entrypoint_selector = FieldElement::ZERO;
        let calldata = compute_hash_on_elements(&[self.class_hash.into()]);
        let max_fee = FieldElement::from(self.max_fee);
        let chain_id = chain_id.into();
        let nonce = FieldElement::from(self.nonce);

        H::compute_hash_on_elements(&[
            prefix,
            version,
            sender_address,
            entrypoint_selector,
            calldata,
            max_fee,
            chain_id,
            nonce,
        ])
        .into()
    }
}

impl ComputeTransactionHash for DeclareTransactionV2 {
    fn compute_hash<H: HasherT>(&self, chain_id: Felt252Wrapper, offset_version: bool) -> Felt252Wrapper {
        let prefix = FieldElement::from_byte_slice_be(DECLARE_PREFIX).unwrap();
        let version = if offset_version { SIMULATE_TX_VERSION_OFFSET + FieldElement::TWO } else { FieldElement::TWO };
        let sender_address = self.sender_address.into();
        let entrypoint_selector = FieldElement::ZERO;
        let calldata = compute_hash_on_elements(&[self.class_hash.into()]);
        let max_fee = FieldElement::from(self.max_fee);
        let chain_id = chain_id.into();
        let nonce = FieldElement::from(self.nonce);
        let compiled_class_hash = self.compiled_class_hash.into();

        H::compute_hash_on_elements(&[
            prefix,
            version,
            sender_address,
            entrypoint_selector,
            calldata,
            max_fee,
            chain_id,
            nonce,
            compiled_class_hash,
        ])
        .into()
    }
}

impl ComputeTransactionHash for DeclareTransaction {
    fn compute_hash<H: HasherT>(&self, chain_id: Felt252Wrapper, offset_version: bool) -> Felt252Wrapper {
        match self {
            DeclareTransaction::V0(tx) => tx.compute_hash::<H>(chain_id, offset_version),
            DeclareTransaction::V1(tx) => tx.compute_hash::<H>(chain_id, offset_version),
            DeclareTransaction::V2(tx) => tx.compute_hash::<H>(chain_id, offset_version),
        }
    }
}

impl ComputeTransactionHash for DeployAccountTransaction {
    fn compute_hash<H: HasherT>(&self, chain_id: Felt252Wrapper, offset_version: bool) -> Felt252Wrapper {
        let chain_id = chain_id.into();
        let contract_address = self.get_account_address();

        self.compute_hash_given_contract_address::<H>(chain_id, contract_address, offset_version).into()
    }
}

impl DeployAccountTransaction {
    pub fn get_account_address(&self) -> FieldElement {
        Self::calculate_contract_address(
            self.contract_address_salt.into(),
            self.class_hash.into(),
            convert_calldata(&self.constructor_calldata),
        )
    }

    pub fn calculate_contract_address(
        contract_address_salt: FieldElement,
        class_hash: FieldElement,
        constructor_calldata: &[FieldElement],
    ) -> FieldElement {
        /// Cairo string for "STARKNET_CONTRACT_ADDRESS"
        const PREFIX_CONTRACT_ADDRESS: FieldElement = FieldElement::from_mont([
            3829237882463328880,
            17289941567720117366,
            8635008616843941496,
            533439743893157637,
        ]);
        // 2 ** 251 - 256
        const ADDR_BOUND: FieldElement =
            FieldElement::from_mont([18446743986131443745, 160989183, 18446744073709255680, 576459263475590224]);

        starknet_core::crypto::compute_hash_on_elements(&[
            PREFIX_CONTRACT_ADDRESS,
            FieldElement::ZERO,
            contract_address_salt,
            class_hash,
            starknet_core::crypto::compute_hash_on_elements(constructor_calldata),
        ]) % ADDR_BOUND
    }

    pub(super) fn compute_hash_given_contract_address<H: HasherT>(
        &self,
        chain_id: FieldElement,
        contract_address: FieldElement,
        offset_version: bool,
    ) -> FieldElement {
        let prefix = FieldElement::from_byte_slice_be(DEPLOY_ACCOUNT_PREFIX).unwrap();
        let version = if offset_version { SIMULATE_TX_VERSION_OFFSET + FieldElement::ONE } else { FieldElement::ONE };
        let entrypoint_selector = FieldElement::ZERO;
        let mut calldata: Vec<FieldElement> = Vec::with_capacity(self.constructor_calldata.len() + 2);
        calldata.push(self.class_hash.into());
        calldata.push(self.contract_address_salt.into());
        calldata.extend_from_slice(convert_calldata(&self.constructor_calldata));
        let calldata_hash = compute_hash_on_elements(&calldata);
        let max_fee = FieldElement::from(self.max_fee);
        let nonce = FieldElement::from(self.nonce);
        let elements =
            &[prefix, version, contract_address, entrypoint_selector, calldata_hash, max_fee, chain_id, nonce];

        H::compute_hash_on_elements(elements)
    }
}

impl ComputeTransactionHash for HandleL1MessageTransaction {
    fn compute_hash<H: HasherT>(&self, chain_id: Felt252Wrapper, offset_version: bool) -> Felt252Wrapper {
        let prefix = FieldElement::from_byte_slice_be(L1_HANDLER_PREFIX).unwrap();
        let version = if offset_version { SIMULATE_TX_VERSION_OFFSET } else { FieldElement::ZERO };
        let contract_address = self.contract_address.into();
        let entrypoint_selector = self.entry_point_selector.into();
        let calldata_hash = compute_hash_on_elements(convert_calldata(&self.calldata));
        let chain_id = chain_id.into();
        let nonce = self.nonce.into();

        H::compute_hash_on_elements(&[
            prefix,
            version,
            contract_address,
            entrypoint_selector,
            calldata_hash,
            chain_id,
            nonce,
        ])
        .into()
    }
}

impl ComputeTransactionHash for Transaction {
    fn compute_hash<H: HasherT>(&self, chain_id: Felt252Wrapper, offset_version: bool) -> Felt252Wrapper {
        match self {
            Transaction::Declare(tx) => tx.compute_hash::<H>(chain_id, offset_version),
            Transaction::DeployAccount(tx) => tx.compute_hash::<H>(chain_id, offset_version),
            Transaction::Invoke(tx) => tx.compute_hash::<H>(chain_id, offset_version),
            Transaction::L1Handler(tx) => tx.compute_hash::<H>(chain_id, offset_version),
        }
    }
}

impl ComputeTransactionHash for UserTransaction {
    fn compute_hash<H: HasherT>(&self, chain_id: Felt252Wrapper, offset_version: bool) -> Felt252Wrapper {
        match self {
            UserTransaction::Declare(tx, _) => tx.compute_hash::<H>(chain_id, offset_version),
            UserTransaction::DeployAccount(tx) => tx.compute_hash::<H>(chain_id, offset_version),
            UserTransaction::Invoke(tx) => tx.compute_hash::<H>(chain_id, offset_version),
        }
    }
}

impl ComputeTransactionHash for UserOrL1HandlerTransaction {
    fn compute_hash<H: HasherT>(&self, chain_id: Felt252Wrapper, offset_version: bool) -> Felt252Wrapper {
        match self {
            UserOrL1HandlerTransaction::User(tx) => tx.compute_hash::<H>(chain_id, offset_version),
            UserOrL1HandlerTransaction::L1Handler(tx, _) => tx.compute_hash::<H>(chain_id, offset_version),
        }
    }
}

#[cfg(test)]
#[path = "compute_hash_tests.rs"]
mod compute_hash_tests;
