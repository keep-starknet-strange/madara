use mp_felt::Felt252Wrapper;
use starknet_crypto::FieldElement;

pub fn to_starknet_core_tx(
    tx: blockifier::transaction::transaction_execution::Transaction,
) -> starknet_core::types::Transaction {
    match tx {
        blockifier::transaction::transaction_execution::Transaction::AccountTransaction(acc_tx) => match acc_tx {
            blockifier::transaction::account_transaction::AccountTransaction::Declare(dec_tx) => match dec_tx.tx {
                starknet_api::transaction::DeclareTransaction::V0(tx) => starknet_core::types::Transaction::Declare(
                    starknet_core::types::DeclareTransaction::V0(starknet_core::types::DeclareTransactionV0 {
                        transaction_hash: Felt252Wrapper::from(dec_tx.tx_hash).into(),
                        sender_address: Felt252Wrapper::from(tx.sender_address).into(),
                        max_fee: FieldElement::from(tx.max_fee.0),
                        signature: tx.signature.0.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                        class_hash: Felt252Wrapper::from(tx.class_hash).into(),
                    }),
                ),
                starknet_api::transaction::DeclareTransaction::V1(tx) => starknet_core::types::Transaction::Declare(
                    starknet_core::types::DeclareTransaction::V1(starknet_core::types::DeclareTransactionV1 {
                        transaction_hash: Felt252Wrapper::from(dec_tx.tx_hash).into(),
                        sender_address: Felt252Wrapper::from(tx.sender_address).into(),
                        max_fee: FieldElement::from(tx.max_fee.0),
                        signature: tx.signature.0.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                        nonce: Felt252Wrapper::from(tx.nonce).into(),
                        class_hash: Felt252Wrapper::from(tx.class_hash).into(),
                    }),
                ),
                starknet_api::transaction::DeclareTransaction::V2(tx) => starknet_core::types::Transaction::Declare(
                    starknet_core::types::DeclareTransaction::V2(starknet_core::types::DeclareTransactionV2 {
                        transaction_hash: Felt252Wrapper::from(dec_tx.tx_hash).into(),
                        sender_address: Felt252Wrapper::from(tx.sender_address).into(),
                        compiled_class_hash: Felt252Wrapper::from(tx.compiled_class_hash).into(),
                        max_fee: FieldElement::from(tx.max_fee.0),
                        signature: tx.signature.0.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                        nonce: Felt252Wrapper::from(tx.nonce).into(),
                        class_hash: Felt252Wrapper::from(tx.class_hash).into(),
                    }),
                ),
                starknet_api::transaction::DeclareTransaction::V3(tx) => starknet_core::types::Transaction::Declare(
                    starknet_core::types::DeclareTransaction::V3(starknet_core::types::DeclareTransactionV3 {
                        transaction_hash: Felt252Wrapper::from(dec_tx.tx_hash).into(),
                        sender_address: Felt252Wrapper::from(tx.sender_address).into(),
                        compiled_class_hash: Felt252Wrapper::from(tx.compiled_class_hash).into(),
                        signature: tx.signature.0.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                        nonce: Felt252Wrapper::from(tx.nonce).into(),
                        class_hash: Felt252Wrapper::from(tx.class_hash).into(),
                        resource_bounds: resource_bounds_mapping_conversion(tx.resource_bounds),
                        tip: tx.tip.0,
                        paymaster_data: tx
                            .paymaster_data
                            .0
                            .into_iter()
                            .map(|v| Felt252Wrapper::from(v).into())
                            .collect(),
                        account_deployment_data: tx
                            .account_deployment_data
                            .0
                            .into_iter()
                            .map(|v| Felt252Wrapper::from(v).into())
                            .collect(),
                        nonce_data_availability_mode: data_availability_mode_conversion(
                            tx.nonce_data_availability_mode,
                        ),
                        fee_data_availability_mode: data_availability_mode_conversion(tx.fee_data_availability_mode),
                    }),
                ),
            },
            blockifier::transaction::account_transaction::AccountTransaction::DeployAccount(da_tx) => match da_tx.tx {
                starknet_api::transaction::DeployAccountTransaction::V1(tx) => {
                    starknet_core::types::Transaction::DeployAccount(
                        starknet_core::types::DeployAccountTransaction::V1(
                            starknet_core::types::DeployAccountTransactionV1 {
                                transaction_hash: Felt252Wrapper::from(da_tx.tx_hash).into(),
                                max_fee: FieldElement::from(tx.max_fee.0),
                                signature: tx.signature.0.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                                nonce: Felt252Wrapper::from(tx.nonce).into(),
                                contract_address_salt: Felt252Wrapper::from(tx.contract_address_salt).into(),
                                constructor_calldata: tx
                                    .constructor_calldata
                                    .0
                                    .iter()
                                    .map(|&v| Felt252Wrapper::from(v).into())
                                    .collect(),
                                class_hash: Felt252Wrapper::from(tx.class_hash).into(),
                            },
                        ),
                    )
                }
                starknet_api::transaction::DeployAccountTransaction::V3(tx) => {
                    starknet_core::types::Transaction::DeployAccount(
                        starknet_core::types::DeployAccountTransaction::V3(
                            starknet_core::types::DeployAccountTransactionV3 {
                                transaction_hash: Felt252Wrapper::from(da_tx.tx_hash).into(),
                                signature: tx.signature.0.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                                nonce: Felt252Wrapper::from(tx.nonce).into(),
                                contract_address_salt: Felt252Wrapper::from(tx.contract_address_salt).into(),
                                constructor_calldata: tx
                                    .constructor_calldata
                                    .0
                                    .iter()
                                    .map(|&v| Felt252Wrapper::from(v).into())
                                    .collect(),
                                class_hash: Felt252Wrapper::from(tx.class_hash).into(),
                                resource_bounds: resource_bounds_mapping_conversion(tx.resource_bounds),
                                tip: tx.tip.0,
                                paymaster_data: tx
                                    .paymaster_data
                                    .0
                                    .into_iter()
                                    .map(|v| Felt252Wrapper::from(v).into())
                                    .collect(),
                                nonce_data_availability_mode: data_availability_mode_conversion(
                                    tx.nonce_data_availability_mode,
                                ),
                                fee_data_availability_mode: data_availability_mode_conversion(
                                    tx.fee_data_availability_mode,
                                ),
                            },
                        ),
                    )
                }
            },
            blockifier::transaction::account_transaction::AccountTransaction::Invoke(inv_tx) => match inv_tx.tx {
                starknet_api::transaction::InvokeTransaction::V0(tx) => starknet_core::types::Transaction::Invoke(
                    starknet_core::types::InvokeTransaction::V0(starknet_core::types::InvokeTransactionV0 {
                        transaction_hash: Felt252Wrapper::from(inv_tx.tx_hash).into(),
                        max_fee: FieldElement::from(tx.max_fee.0),
                        signature: tx.signature.0.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                        contract_address: Felt252Wrapper::from(tx.contract_address).into(),
                        entry_point_selector: Felt252Wrapper::from(tx.entry_point_selector).into(),
                        calldata: tx.calldata.0.iter().map(|&v| Felt252Wrapper::from(v).into()).collect(),
                    }),
                ),
                starknet_api::transaction::InvokeTransaction::V1(tx) => starknet_core::types::Transaction::Invoke(
                    starknet_core::types::InvokeTransaction::V1(starknet_core::types::InvokeTransactionV1 {
                        transaction_hash: Felt252Wrapper::from(inv_tx.tx_hash).into(),
                        sender_address: Felt252Wrapper::from(tx.sender_address).into(),
                        calldata: tx.calldata.0.iter().map(|&v| Felt252Wrapper::from(v).into()).collect(),
                        max_fee: FieldElement::from(tx.max_fee.0),
                        signature: tx.signature.0.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                        nonce: Felt252Wrapper::from(tx.nonce).into(),
                    }),
                ),
                starknet_api::transaction::InvokeTransaction::V3(tx) => starknet_core::types::Transaction::Invoke(
                    starknet_core::types::InvokeTransaction::V3(starknet_core::types::InvokeTransactionV3 {
                        transaction_hash: Felt252Wrapper::from(inv_tx.tx_hash).into(),
                        sender_address: Felt252Wrapper::from(tx.sender_address).into(),
                        calldata: tx.calldata.0.iter().map(|&v| Felt252Wrapper::from(v).into()).collect(),
                        signature: tx.signature.0.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                        nonce: Felt252Wrapper::from(tx.nonce).into(),
                        resource_bounds: resource_bounds_mapping_conversion(tx.resource_bounds),
                        tip: tx.tip.0,
                        paymaster_data: tx
                            .paymaster_data
                            .0
                            .into_iter()
                            .map(|v| Felt252Wrapper::from(v).into())
                            .collect(),
                        account_deployment_data: tx
                            .account_deployment_data
                            .0
                            .into_iter()
                            .map(|v| Felt252Wrapper::from(v).into())
                            .collect(),
                        nonce_data_availability_mode: data_availability_mode_conversion(
                            tx.nonce_data_availability_mode,
                        ),
                        fee_data_availability_mode: data_availability_mode_conversion(tx.fee_data_availability_mode),
                    }),
                ),
            },
        },
        blockifier::transaction::transaction_execution::Transaction::L1HandlerTransaction(l1h_tx) => {
            starknet_core::types::Transaction::L1Handler(starknet_core::types::L1HandlerTransaction {
                transaction_hash: Felt252Wrapper::from(l1h_tx.tx_hash).into(),
                version: FieldElement::ZERO,
                // Safe to unwrap as long as there is less than u64::MAX messages sent from l1 to l1.
                // We have some margin here.
                nonce: u64::try_from(Felt252Wrapper::from(l1h_tx.tx.nonce)).unwrap(),
                contract_address: Felt252Wrapper::from(l1h_tx.tx.contract_address).into(),
                entry_point_selector: Felt252Wrapper::from(l1h_tx.tx.entry_point_selector).into(),
                calldata: l1h_tx.tx.calldata.0.iter().map(|&v| Felt252Wrapper::from(v).into()).collect(),
            })
        }
    }
}

fn data_availability_mode_conversion(
    da_mode: starknet_api::data_availability::DataAvailabilityMode,
) -> starknet_core::types::DataAvailabilityMode {
    match da_mode {
        starknet_api::data_availability::DataAvailabilityMode::L1 => starknet_core::types::DataAvailabilityMode::L1,
        starknet_api::data_availability::DataAvailabilityMode::L2 => starknet_core::types::DataAvailabilityMode::L2,
    }
}

fn resource_bounds_mapping_conversion(
    resource_bounds: starknet_api::transaction::ResourceBoundsMapping,
) -> starknet_core::types::ResourceBoundsMapping {
    let l1_gas = resource_bounds.0.get(&starknet_api::transaction::Resource::L1Gas);
    let l2_gas = resource_bounds.0.get(&starknet_api::transaction::Resource::L2Gas);

    starknet_core::types::ResourceBoundsMapping {
        l1_gas: starknet_core::types::ResourceBounds {
            max_amount: l1_gas.map(|v| v.max_amount).unwrap_or_default(),
            max_price_per_unit: l1_gas.map(|v| v.max_price_per_unit).unwrap_or_default(),
        },
        l2_gas: starknet_core::types::ResourceBounds {
            max_amount: l2_gas.map(|v| v.max_amount).unwrap_or_default(),
            max_price_per_unit: l2_gas.map(|v| v.max_price_per_unit).unwrap_or_default(),
        },
    }
}
