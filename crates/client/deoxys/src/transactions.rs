use mp_transactions::{DeclareTransactionV0, DeclareTransactionV1, DeclareTransactionV2};
use starknet_client::reader::objects::transaction::{IntermediateDeclareTransaction, IntermediateInvokeTransaction};
use starknet_client::reader::ReaderClientError;

pub fn declare_tx_to_starknet_tx(
    declare_transaction: IntermediateDeclareTransaction,
) -> Result<mp_transactions::Transaction, ReaderClientError> {
    // Convert `IntermediateDeclareTransaction` to `starknet_api::transaction::DeclareTransaction`
    let starknet_declare_tx = starknet_api::transaction::DeclareTransaction::try_from(declare_transaction)?;

    // Convert `starknet_api::transaction::DeclareTransaction` to `mp_transactions::DeclareTransaction`
    let mp_declare_tx = match starknet_declare_tx {
        starknet_api::transaction::DeclareTransaction::V0(inner) => {
            mp_transactions::DeclareTransaction::V0(DeclareTransactionV0::from_starknet(inner))
        }
        starknet_api::transaction::DeclareTransaction::V1(inner) => {
            mp_transactions::DeclareTransaction::V1(DeclareTransactionV1::from_starknet(inner))
        }
        starknet_api::transaction::DeclareTransaction::V2(inner) => {
            mp_transactions::DeclareTransaction::V2(DeclareTransactionV2::from_starknet(inner))
        }
    };

    Ok(mp_transactions::Transaction::Declare(mp_declare_tx))
}

pub fn invoke_tx_to_starknet_tx(
    invoke_transaction: IntermediateInvokeTransaction,
) -> Result<mp_transactions::Transaction, ReaderClientError> {
    // Try to convert the intermediate representation to the starknet_api representation
    let starknet_invoke_tx = starknet_api::transaction::InvokeTransaction::try_from(invoke_transaction)?;

    // Convert `starknet_api::transaction::InvokeTransaction` to `mp_transactions::InvokeTransaction`
    let mp_invoke_tx = match starknet_invoke_tx {
        starknet_api::transaction::InvokeTransaction::V0(inner) => {
            mp_transactions::InvokeTransaction::V0(mp_transactions::InvokeTransactionV0::from_starknet(inner))
        }
        starknet_api::transaction::InvokeTransaction::V1(inner) => {
            mp_transactions::InvokeTransaction::V1(mp_transactions::InvokeTransactionV1::from_starknet(inner))
        }
    };

    Ok(mp_transactions::Transaction::Invoke(mp_invoke_tx))
}

pub async fn deploy_tx_to_starknet_tx(
    deploy_transaction: starknet_api::transaction::DeployTransaction,
) -> Result<mp_transactions::Transaction, ReaderClientError> {
    let mp_deploy_tx = mp_transactions::DeployTransaction::from_starknet(deploy_transaction);
    Ok(mp_transactions::Transaction::Deploy(mp_deploy_tx))
}

pub fn deploy_account_tx_to_starknet_tx(
    deploy_account_transaction: starknet_api::transaction::DeployAccountTransaction,
) -> Result<mp_transactions::Transaction, ReaderClientError> {
    let mp_deploy_account_tx = mp_transactions::DeployAccountTransaction::from_starknet(deploy_account_transaction);
    Ok(mp_transactions::Transaction::DeployAccount(mp_deploy_account_tx))
}

pub fn l1handler_tx_to_starknet_tx(
    l1handler_transaction: starknet_api::transaction::L1HandlerTransaction,
) -> Result<mp_transactions::Transaction, ReaderClientError> {
    let mp_l1handler_tx = mp_transactions::HandleL1MessageTransaction::from_starknet(l1handler_transaction);
    Ok(mp_transactions::Transaction::L1Handler(mp_l1handler_tx))
}
