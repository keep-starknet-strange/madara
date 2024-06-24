// Test for both get_class and get_class_at

// Important: those routes won't work for contracts declared during genesis as the node can't
// collect the additional data needed to return a fully fleshed ContractClass.
// The declaration have to go through the RPC for it to work.

use std::io::Read;

use assert_matches::assert_matches;
use flate2::read::GzDecoder;
use rstest::rstest;
use starknet_accounts::Execution;
use starknet_contract::ContractFactory;
use starknet_core::types::contract::legacy::{LegacyContractClass, LegacyDebugInfo, LegacyProgram};
use starknet_core::types::contract::SierraClass;
use starknet_core::types::{BlockId, ContractClass, FlattenedSierraClass, StarknetError};
use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, SIGNER_PRIVATE, TEST_CONTRACT_CLASS_HASH};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{
    build_single_owner_account, get_contract_address_from_deploy_tx, get_transaction_receipt,
};
use starknet_rpc_test::{Transaction, TransactionResult};
use starknet_test_utils::constants::MAX_FEE_OVERRIDE;
use starknet_test_utils::utils::AccountActions;
use starknet_types_core::felt::Felt;

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let test_contract_class_hash =
        FieldElement::from_hex_be(TEST_CONTRACT_CLASS_HASH).expect("Invalid Contract Address");

    assert_matches!(
        rpc.get_class(BlockId::Number(100), test_contract_class_hash,).await,
        Err(StarknetProviderError(StarknetError::BlockNotFound))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_non_existing_class_hash(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let unknown_contract_class_hash =
        FieldElement::from_hex_be("0x4269DEADBEEF").expect("Invalid Contract classh hash");

    assert_matches!(
        rpc.get_class(BlockId::Number(0), unknown_contract_class_hash,).await,
        Err(StarknetProviderError(StarknetError::ClassHashNotFound))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_non_existing_address(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let unknown_contract_address = FieldElement::from_hex_be("0x4269DEADBEEF").expect("Invalid Contract classh hash");

    assert_matches!(
        rpc.get_class_at(BlockId::Number(0), unknown_contract_address,).await,
        Err(StarknetProviderError(StarknetError::ContractNotFound))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn work_ok_retrieving_class_for_contract_version_0(madara: &ThreadSafeMadaraClient) {
    // Test that the contract class received from the rpc call get_class and get_class_at is the same as
    // the one that was used to declare
    fn assert_eq_contract_class(received: ContractClass, mut expected: LegacyContractClass) {
        assert_matches!(
            received,
            ContractClass::Legacy(c) => {

                assert_eq!(
                    serde_json::to_value(expected.abi).unwrap(),
                    serde_json::to_value(c.abi).unwrap(),
                );
                assert_eq!(
                    serde_json::to_value(expected.entry_points_by_type).unwrap(),
                    serde_json::to_value(c.entry_points_by_type).unwrap(),
                );

                // decompress program
                let mut gz = GzDecoder::new(&c.program[..]);
                let mut decompressed_bytes = Vec::new();
                gz.read_to_end(&mut decompressed_bytes).unwrap();
                let mut program: LegacyProgram = serde_json::from_slice(decompressed_bytes.as_slice()).unwrap();

                // Because of some obscure pathfinder bug, starknet-rs deserialize debug_info as an empty struct rather than none
                // It has been fixed, so when https://github.com/xJonathanLEI/starknet-rs/pull/599 is megerd, this will become useless
                // In the meantime we take this field out and make sure it is the empty struct
                expected.program.debug_info.take();
                let debug_infos = program.debug_info.take();
                assert_eq!(
                    serde_json::to_value(LegacyDebugInfo { file_contents: Default::default(), instruction_locations: Default::default()}).unwrap(),
                    serde_json::to_value(debug_infos).unwrap(),
                );
                // Because fucking program.identifiers.values are raw string, they can be either a positive or a negative number when deserialized
                // In order to make sure those values are trully equal, we need to convert them back to Felt so they can wrap arround
                // We take them out so we can latter call `==` on the whole `program.identifiers`
                for (a, b) in expected.program.identifiers.iter_mut().zip(program.identifiers.iter_mut()) {
                    let a =
                       a.1.value.take().map(|v| Felt::from_dec_str(v.get()).unwrap());
                    let b =
                       b.1.value.take().map(|v| Felt::from_dec_str(v.get()).unwrap());
                     assert_eq!(
                         a, b
                    );
                }
                // Finally compar the fixed program
                assert_eq!(
                    serde_json::to_value(expected.program).unwrap(),
                    serde_json::to_value(program).unwrap(),
                );
            }
        )
    }

    let rpc = madara.get_starknet_client().await;

    let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let (declare_tx, expected_class_hash) =
        account.declare_legacy_contract("../cairo-contracts/build/UnauthorizedInnerCallAccount.json");
    let contract_class: LegacyContractClass = serde_json::from_reader(
        std::fs::File::open("../cairo-contracts/build/UnauthorizedInnerCallAccount.json").unwrap(),
    )
    .unwrap();

    // Declare the class
    let (txs, block_number) = {
        let mut madara_write_lock = madara.write().await;
        let txs =
            madara_write_lock.create_block_with_txs(vec![Transaction::LegacyDeclaration(declare_tx)]).await.unwrap();
        let block_number = rpc.block_number().await.unwrap();
        (txs, block_number)
    };
    assert!(txs[0].is_ok(), "add declare tx failed");

    // Wait for the tx to be synced
    let tx_hash = match &txs[0] {
        Ok(TransactionResult::Declaration(rpc_response)) => rpc_response.transaction_hash,
        _ => panic!("expected declaration result"),
    };
    let _ = get_transaction_receipt(&rpc, tx_hash).await.unwrap();

    // Check that get_class works
    let received_contract_class = rpc.get_class(BlockId::Number(block_number), expected_class_hash).await.unwrap();
    assert_eq_contract_class(received_contract_class, contract_class.clone());

    // Now deploy the contract
    let contract_factory = ContractFactory::new(expected_class_hash, account.clone());
    let max_fee = FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap();
    let deploy_tx = Execution::from(&contract_factory.deploy(vec![], FieldElement::ZERO, true).max_fee(max_fee));

    let (mut txs, block_number) = {
        let mut madara_write_lock = madara.write().await;

        let txs = madara_write_lock.create_block_with_txs(vec![Transaction::Execution(deploy_tx)]).await.unwrap();
        let block_number = rpc.block_number().await.unwrap();

        (txs, block_number)
    };
    let deploy_tx_res = txs.pop().unwrap();
    assert!(deploy_tx_res.is_ok(), "deploy tx failed");

    // And make sure get_class_at also work
    let contract_address = get_contract_address_from_deploy_tx(&rpc, deploy_tx_res).await.unwrap();
    let received_contract_class = rpc.get_class_at(BlockId::Number(block_number), contract_address).await.unwrap();
    assert_eq_contract_class(received_contract_class, contract_class);
}

#[rstest]
#[tokio::test]
async fn work_ok_retrieving_class_for_contract_version_1(madara: &ThreadSafeMadaraClient) {
    let rpc = madara.get_starknet_client().await;

    let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let (declare_tx, class_hash, _) = account.declare_contract(
        "../starknet-rpc-test/contracts/counter10/counter10.contract_class.json",
        "../starknet-rpc-test/contracts/counter10/counter10.compiled_contract_class.json",
        None,
    );

    let (txs, block_number) = {
        let mut madara_write_lock = madara.write().await;

        let txs = madara_write_lock.create_block_with_txs(vec![Transaction::Declaration(declare_tx)]).await.unwrap();
        let block_number = rpc.block_number().await.unwrap();

        (txs, block_number)
    };
    assert!(txs[0].is_ok(), "declare tx failed");

    let test_contract_class_bytes =
        include_bytes!("../starknet-rpc-test/contracts/counter10/counter10.contract_class.json");
    let test_contract_class: SierraClass = serde_json::from_slice(test_contract_class_bytes).unwrap();
    let flattened_test_contract_class: FlattenedSierraClass = test_contract_class.flatten().unwrap();

    // Wait for the tx to be synced
    let tx_hash = match &txs[0] {
        Ok(TransactionResult::Declaration(rpc_response)) => rpc_response.transaction_hash,
        _ => panic!("expected declaration result"),
    };
    let _ = get_transaction_receipt(&rpc, tx_hash).await.unwrap();
    // Check get class works
    assert_matches!(
        rpc
        .get_class(
           BlockId::Number(block_number),
            class_hash
        ).await.unwrap(),
        ContractClass::Sierra(c) => {
            assert_eq!(
                c.abi,
                flattened_test_contract_class.abi,
            );
            assert_eq!(
                c.sierra_program,
                flattened_test_contract_class.sierra_program,
            );
        }
    );

    // Now deploy the contract
    let contract_factory = ContractFactory::new(class_hash, account.clone());
    let max_fee = FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap();
    let deploy_tx = Execution::from(&contract_factory.deploy(vec![], FieldElement::ZERO, true).max_fee(max_fee));

    let (mut txs, block_number) = {
        let mut madara_write_lock = madara.write().await;

        let txs = madara_write_lock.create_block_with_txs(vec![Transaction::Execution(deploy_tx)]).await.unwrap();
        let block_number = rpc.block_number().await.unwrap();

        (txs, block_number)
    };
    let deploy_tx_res = txs.pop().unwrap();
    assert!(deploy_tx_res.is_ok(), "deploy tx failed");

    // And make sure get_class_at also work
    let contract_address = get_contract_address_from_deploy_tx(&rpc, deploy_tx_res).await.unwrap();
    assert_matches!(
    rpc.get_class_at(
        BlockId::Number(block_number), contract_address).await.unwrap(),
        ContractClass::Sierra(c) => {
                assert_eq!(
                    c.abi,
                    flattened_test_contract_class.abi,
                );
                assert_eq!(
                    c.sierra_program,
                    flattened_test_contract_class.sierra_program,
                );
            }
        );
}
