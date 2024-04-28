use mp_felt::Felt252Wrapper;
use mp_hashers::pedersen::PedersenHasher;
use mp_hashers::poseidon::PoseidonHasher;
use mp_hashers::HasherT;
use starknet_api::core::calculate_contract_address;
use starknet_api::data_availability::DataAvailabilityMode;
use starknet_api::transaction::{
    Calldata, DeclareTransaction, DeclareTransactionV0V1, DeclareTransactionV2, DeclareTransactionV3,
    DeployAccountTransaction, DeployAccountTransactionV1, DeployAccountTransactionV3, InvokeTransaction,
    InvokeTransactionV0, InvokeTransactionV1, InvokeTransactionV3, L1HandlerTransaction, PaymasterData, Resource,
    ResourceBoundsMapping, Tip, TransactionHash,
};
use starknet_core::crypto::compute_hash_on_elements;
use starknet_crypto::FieldElement;

use super::SIMULATE_TX_VERSION_OFFSET;

const DECLARE_PREFIX: &[u8] = b"declare";
const DEPLOY_ACCOUNT_PREFIX: &[u8] = b"deploy_account";
const INVOKE_PREFIX: &[u8] = b"invoke";
const L1_HANDLER_PREFIX: &[u8] = b"l1_handler";
const L1_GAS: &[u8] = b"L1_GAS";
const L2_GAS: &[u8] = b"L2_GAS";

pub trait ComputeTransactionHash {
    fn compute_hash(&self, chain_id: Felt252Wrapper, offset_version: bool) -> TransactionHash;
}

fn convert_calldata(calldata: Calldata) -> Vec<FieldElement> {
    calldata.0.iter().map(|f| Felt252Wrapper::from(*f).into()).collect()
}

fn prepare_resource_bound_value(resource_bounds_mapping: &ResourceBoundsMapping, resource: Resource) -> FieldElement {
    let mut buffer = [0u8; 32];
    buffer[2..8].copy_from_slice(match resource {
        Resource::L1Gas => L1_GAS,
        Resource::L2Gas => L2_GAS,
    });
    if let Some(resource_bounds) = resource_bounds_mapping.0.get(&resource) {
        buffer[8..16].copy_from_slice(&resource_bounds.max_amount.to_be_bytes());
        buffer[16..].copy_from_slice(&resource_bounds.max_price_per_unit.to_be_bytes());
    };

    // Safe to unwrap because we left most significant bit of the buffer empty
    FieldElement::from_bytes_be(&buffer).unwrap()
}

fn prepare_data_availability_modes(
    nonce_data_availability_mode: DataAvailabilityMode,
    fee_data_availability_mode: DataAvailabilityMode,
) -> FieldElement {
    let mut buffer = [0u8; 32];
    buffer[24..28].copy_from_slice(&(nonce_data_availability_mode as u32).to_be_bytes());
    buffer[28..].copy_from_slice(&(fee_data_availability_mode as u32).to_be_bytes());

    // Safe to unwrap because we left most significant bit of the buffer empty
    FieldElement::from_bytes_be(&buffer).unwrap()
}

impl ComputeTransactionHash for InvokeTransactionV0 {
    fn compute_hash(&self, chain_id: Felt252Wrapper, offset_version: bool) -> TransactionHash {
        let prefix = FieldElement::from_byte_slice_be(INVOKE_PREFIX).unwrap();
        let version = if offset_version { SIMULATE_TX_VERSION_OFFSET } else { FieldElement::ZERO };
        let contract_address = Felt252Wrapper::from(self.contract_address).into();
        let entrypoint_selector = Felt252Wrapper::from(self.entry_point_selector).into();
        let calldata_hash = compute_hash_on_elements(&convert_calldata(self.calldata.clone()));
        let max_fee = FieldElement::from(self.max_fee.0);

        Felt252Wrapper(PedersenHasher::compute_hash_on_elements(&[
            prefix,
            version,
            contract_address,
            entrypoint_selector,
            calldata_hash,
            max_fee,
            chain_id.into(),
        ]))
        .into()
    }
}

impl ComputeTransactionHash for InvokeTransactionV1 {
    fn compute_hash(&self, chain_id: Felt252Wrapper, offset_version: bool) -> TransactionHash {
        let prefix = FieldElement::from_byte_slice_be(INVOKE_PREFIX).unwrap();
        let version = if offset_version { SIMULATE_TX_VERSION_OFFSET + FieldElement::ONE } else { FieldElement::ONE };
        let sender_address = Felt252Wrapper::from(self.sender_address).into();
        let entrypoint_selector = FieldElement::ZERO;
        let calldata_hash = compute_hash_on_elements(&convert_calldata(self.calldata.clone()));
        let max_fee = FieldElement::from(self.max_fee.0);
        let nonce = Felt252Wrapper::from(self.nonce.0).into();

        Felt252Wrapper(PedersenHasher::compute_hash_on_elements(&[
            prefix,
            version,
            sender_address,
            entrypoint_selector,
            calldata_hash,
            max_fee,
            chain_id.into(),
            nonce,
        ]))
        .into()
    }
}

impl ComputeTransactionHash for InvokeTransactionV3 {
    fn compute_hash(&self, chain_id: Felt252Wrapper, offset_version: bool) -> TransactionHash {
        let prefix = FieldElement::from_byte_slice_be(INVOKE_PREFIX).unwrap();
        let version =
            if offset_version { SIMULATE_TX_VERSION_OFFSET + FieldElement::THREE } else { FieldElement::THREE };
        let sender_address = Felt252Wrapper::from(self.sender_address).into();
        let nonce = Felt252Wrapper::from(self.nonce.0).into();
        let account_deployment_data_hash = PoseidonHasher::compute_hash_on_elements(
            &self.account_deployment_data.0.iter().map(|f| Felt252Wrapper::from(*f).into()).collect::<Vec<_>>(),
        );
        let calldata_hash = PoseidonHasher::compute_hash_on_elements(
            &self.calldata.0.iter().map(|f| Felt252Wrapper::from(*f).into()).collect::<Vec<_>>(),
        );

        compute_transaction_hash_common_v3(
            prefix,
            version,
            sender_address,
            chain_id.into(),
            nonce,
            self.tip,
            &self.paymaster_data,
            self.nonce_data_availability_mode,
            self.fee_data_availability_mode,
            &self.resource_bounds,
            vec![account_deployment_data_hash, calldata_hash],
        )
    }
}

impl ComputeTransactionHash for InvokeTransaction {
    fn compute_hash(&self, chain_id: Felt252Wrapper, offset_version: bool) -> TransactionHash {
        match self {
            InvokeTransaction::V0(tx) => tx.compute_hash(chain_id, offset_version),
            InvokeTransaction::V1(tx) => tx.compute_hash(chain_id, offset_version),
            InvokeTransaction::V3(tx) => tx.compute_hash(chain_id, offset_version),
        }
    }
}

fn compute_hash_declare_v0_or_v1(
    chain_id: Felt252Wrapper,
    offset_version: bool,
    tx: &DeclareTransactionV0V1,
    version: u8,
) -> TransactionHash {
    let prefix = FieldElement::from_byte_slice_be(DECLARE_PREFIX).unwrap();
    let sender_address = Felt252Wrapper::from(tx.sender_address).into();
    let zero = FieldElement::ZERO;
    let class_or_nothing_hash = if version == 0 {
        compute_hash_on_elements(&[])
    } else {
        compute_hash_on_elements(&[Felt252Wrapper::from(tx.class_hash).into()])
    };
    let max_fee = FieldElement::from(tx.max_fee.0);
    let nonce_or_class_hash =
        if version == 0 { Felt252Wrapper::from(tx.class_hash).into() } else { Felt252Wrapper::from(tx.nonce).into() };
    let version = if offset_version {
        SIMULATE_TX_VERSION_OFFSET + FieldElement::from(version)
    } else {
        FieldElement::from(version)
    };

    Felt252Wrapper(PedersenHasher::compute_hash_on_elements(&[
        prefix,
        version,
        sender_address,
        zero,
        class_or_nothing_hash,
        max_fee,
        chain_id.into(),
        nonce_or_class_hash,
    ]))
    .into()
}

impl ComputeTransactionHash for DeclareTransactionV2 {
    fn compute_hash(&self, chain_id: Felt252Wrapper, offset_version: bool) -> TransactionHash {
        let prefix = FieldElement::from_byte_slice_be(DECLARE_PREFIX).unwrap();
        let version = if offset_version { SIMULATE_TX_VERSION_OFFSET + FieldElement::TWO } else { FieldElement::TWO };
        let sender_address = Felt252Wrapper::from(self.sender_address).into();
        let entrypoint_selector = FieldElement::ZERO;
        let calldata = compute_hash_on_elements(&[Felt252Wrapper::from(self.class_hash).into()]);
        let max_fee = FieldElement::from(self.max_fee.0);
        let nonce = Felt252Wrapper::from(self.nonce).into();
        let compiled_class_hash = Felt252Wrapper::from(self.compiled_class_hash).into();

        Felt252Wrapper(PedersenHasher::compute_hash_on_elements(&[
            prefix,
            version,
            sender_address,
            entrypoint_selector,
            calldata,
            max_fee,
            chain_id.into(),
            nonce,
            compiled_class_hash,
        ]))
        .into()
    }
}

impl ComputeTransactionHash for DeclareTransactionV3 {
    fn compute_hash(&self, chain_id: Felt252Wrapper, offset_version: bool) -> TransactionHash {
        let prefix = FieldElement::from_byte_slice_be(DECLARE_PREFIX).unwrap();
        let version =
            if offset_version { SIMULATE_TX_VERSION_OFFSET + FieldElement::THREE } else { FieldElement::THREE };
        let sender_address = Felt252Wrapper::from(self.sender_address).into();
        let nonce = Felt252Wrapper::from(self.nonce.0).into();
        let account_deployment_data_hash = PoseidonHasher::compute_hash_on_elements(
            &self.account_deployment_data.0.iter().map(|f| Felt252Wrapper::from(*f).into()).collect::<Vec<_>>(),
        );

        compute_transaction_hash_common_v3(
            prefix,
            version,
            sender_address,
            chain_id.into(),
            nonce,
            self.tip,
            &self.paymaster_data,
            self.nonce_data_availability_mode,
            self.fee_data_availability_mode,
            &self.resource_bounds,
            vec![
                account_deployment_data_hash,
                Felt252Wrapper::from(self.class_hash).into(),
                Felt252Wrapper::from(self.compiled_class_hash).into(),
            ],
        )
    }
}
impl ComputeTransactionHash for DeclareTransaction {
    fn compute_hash(&self, chain_id: Felt252Wrapper, offset_version: bool) -> TransactionHash {
        match self {
            DeclareTransaction::V0(tx) => compute_hash_declare_v0_or_v1(chain_id, offset_version, tx, 0),
            DeclareTransaction::V1(tx) => compute_hash_declare_v0_or_v1(chain_id, offset_version, tx, 1),
            DeclareTransaction::V2(tx) => tx.compute_hash(chain_id, offset_version),
            DeclareTransaction::V3(tx) => tx.compute_hash(chain_id, offset_version),
        }
    }
}

impl ComputeTransactionHash for DeployAccountTransaction {
    fn compute_hash(&self, chain_id: Felt252Wrapper, offset_version: bool) -> TransactionHash {
        match self {
            DeployAccountTransaction::V1(tx) => tx.compute_hash(chain_id, offset_version),
            DeployAccountTransaction::V3(tx) => tx.compute_hash(chain_id, offset_version),
        }
    }
}

impl ComputeTransactionHash for DeployAccountTransactionV1 {
    fn compute_hash(&self, chain_id: Felt252Wrapper, offset_version: bool) -> TransactionHash {
        let constructor_calldata = convert_calldata(self.constructor_calldata.clone());

        let contract_address = Felt252Wrapper::from(
            calculate_contract_address(
                self.contract_address_salt,
                self.class_hash,
                &self.constructor_calldata,
                Default::default(),
            )
            .unwrap(),
        )
        .into();
        let prefix = FieldElement::from_byte_slice_be(DEPLOY_ACCOUNT_PREFIX).unwrap();
        let version = if offset_version { SIMULATE_TX_VERSION_OFFSET + FieldElement::ONE } else { FieldElement::ONE };
        let entrypoint_selector = FieldElement::ZERO;
        let mut calldata: Vec<FieldElement> = Vec::with_capacity(constructor_calldata.len() + 2);
        calldata.push(Felt252Wrapper::from(self.class_hash).into());
        calldata.push(Felt252Wrapper::from(self.contract_address_salt).into());
        calldata.extend_from_slice(&constructor_calldata);
        let calldata_hash = compute_hash_on_elements(&calldata);
        let max_fee = FieldElement::from(self.max_fee.0);
        let nonce = Felt252Wrapper::from(self.nonce).into();

        Felt252Wrapper(PedersenHasher::compute_hash_on_elements(&[
            prefix,
            version,
            contract_address,
            entrypoint_selector,
            calldata_hash,
            max_fee,
            chain_id.into(),
            nonce,
        ]))
        .into()
    }
}

impl ComputeTransactionHash for DeployAccountTransactionV3 {
    fn compute_hash(&self, chain_id: Felt252Wrapper, offset_version: bool) -> TransactionHash {
        let prefix = FieldElement::from_byte_slice_be(DEPLOY_ACCOUNT_PREFIX).unwrap();
        let version =
            if offset_version { SIMULATE_TX_VERSION_OFFSET + FieldElement::THREE } else { FieldElement::THREE };
        let constructor_calldata = convert_calldata(self.constructor_calldata.clone());
        let contract_address = Felt252Wrapper::from(
            calculate_contract_address(
                self.contract_address_salt,
                self.class_hash,
                &self.constructor_calldata,
                Default::default(),
            )
            .unwrap(),
        )
        .into();
        let nonce = Felt252Wrapper::from(self.nonce.0).into();
        let constructor_calldata_hash = PoseidonHasher::compute_hash_on_elements(&constructor_calldata);

        compute_transaction_hash_common_v3(
            prefix,
            version,
            contract_address,
            chain_id.into(),
            nonce,
            self.tip,
            &self.paymaster_data,
            self.nonce_data_availability_mode,
            self.fee_data_availability_mode,
            &self.resource_bounds,
            vec![
                constructor_calldata_hash,
                Felt252Wrapper::from(self.class_hash).into(),
                Felt252Wrapper::from(self.contract_address_salt).into(),
            ],
        )
    }
}

impl ComputeTransactionHash for L1HandlerTransaction {
    fn compute_hash(&self, chain_id: Felt252Wrapper, offset_version: bool) -> TransactionHash {
        let prefix = FieldElement::from_byte_slice_be(L1_HANDLER_PREFIX).unwrap();
        let version = if offset_version { SIMULATE_TX_VERSION_OFFSET } else { FieldElement::ZERO };
        let contract_address = Felt252Wrapper::from(self.contract_address).into();
        let entrypoint_selector = Felt252Wrapper::from(self.entry_point_selector).into();
        let calldata_hash = compute_hash_on_elements(&convert_calldata(self.calldata.clone()));
        let nonce = Felt252Wrapper::from(self.nonce).into();

        Felt252Wrapper(PedersenHasher::compute_hash_on_elements(&[
            prefix,
            version,
            contract_address,
            entrypoint_selector,
            calldata_hash,
            chain_id.into(),
            nonce,
        ]))
        .into()
    }
}

#[allow(clippy::too_many_arguments)]
fn compute_transaction_hash_common_v3(
    tx_hash_prefix: FieldElement,
    version: FieldElement,
    sender_address: FieldElement,
    chain_id: FieldElement,
    nonce: FieldElement,
    tip: Tip,
    paymaster_data: &PaymasterData,
    nonce_data_availability_mode: DataAvailabilityMode,
    fee_data_availability_mode: DataAvailabilityMode,
    resource_bounds: &ResourceBoundsMapping,
    additional_data: Vec<FieldElement>,
) -> TransactionHash {
    let gas_hash = PoseidonHasher::compute_hash_on_elements(&[
        FieldElement::from(tip.0),
        prepare_resource_bound_value(resource_bounds, Resource::L1Gas),
        prepare_resource_bound_value(resource_bounds, Resource::L2Gas),
    ]);
    let paymaster_hash = PoseidonHasher::compute_hash_on_elements(
        &paymaster_data.0.iter().map(|f| Felt252Wrapper::from(*f).into()).collect::<Vec<_>>(),
    );
    let data_availability_modes =
        prepare_data_availability_modes(nonce_data_availability_mode, fee_data_availability_mode);
    let mut data_to_hash = vec![
        tx_hash_prefix,
        version,
        sender_address,
        gas_hash,
        paymaster_hash,
        chain_id,
        nonce,
        data_availability_modes,
    ];
    data_to_hash.extend(additional_data);
    Felt252Wrapper(PoseidonHasher::compute_hash_on_elements(data_to_hash.as_slice())).into()
}

#[cfg(test)]
#[path = "compute_hash_tests.rs"]
mod compute_hash_tests;
