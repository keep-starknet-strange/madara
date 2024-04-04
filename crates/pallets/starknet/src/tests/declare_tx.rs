use assert_matches::assert_matches;
use blockifier::execution::contract_class::ClassInfo;
use blockifier::transaction::transactions::DeclareTransaction as BlockifierDeclareTransaction;
use frame_support::{assert_err, assert_ok};
use mp_felt::Felt252Wrapper;
use mp_transactions::compute_hash::ComputeTransactionHash;
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::transaction_validity::{
    InvalidTransaction, TransactionSource, TransactionValidityError, ValidTransaction,
};
use starknet_api::core::{ClassHash, CompiledClassHash, ContractAddress, Nonce, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{
    DeclareTransaction as StarknetApiDeclareTransaction, DeclareTransactionV0V1, DeclareTransactionV2, Fee,
    TransactionSignature,
};
use starknet_crypto::FieldElement;

use super::mock::default_mock::*;
use super::mock::*;
use super::utils::{get_contract_class, sign_message_hash};
use crate::tests::set_nonce;
use crate::Error;

fn create_declare_erc20_v0_transaction(
    chain_id: Felt252Wrapper,
    account_type: AccountType,
    sender_address: Option<ContractAddress>,
    signature: Option<TransactionSignature>,
) -> BlockifierDeclareTransaction {
    let sender_address = sender_address.unwrap_or_else(|| get_account_address(None, account_type));

    let erc20_class = get_contract_class("ERC20.json", 0);
    let erc20_class_hash =
        ClassHash(StarkFelt::try_from("0x057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap());

    let mut tx = StarknetApiDeclareTransaction::V0(DeclareTransactionV0V1 {
        max_fee: Fee(u128::MAX),
        signature: Default::default(),
        nonce: Nonce(StarkFelt::ZERO),
        class_hash: erc20_class_hash,
        sender_address,
    });

    let tx_hash = tx.compute_hash(chain_id, false);
    // Force to do that because ComputeTransactionHash cannot be implemented on DeclareTransactionV0V1
    // directly...
    if let StarknetApiDeclareTransaction::V0(tx) = &mut tx {
        tx.signature = signature.unwrap_or_else(|| sign_message_hash(tx_hash));
    }

    BlockifierDeclareTransaction::new(tx, tx_hash, ClassInfo::new(&erc20_class, 0, 1).unwrap()).unwrap()
}

#[test]
fn given_contract_declare_tx_works_once_not_twice() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();
        let chain_id = Starknet::chain_id();

        let transaction =
            create_declare_erc20_v0_transaction(chain_id, AccountType::V0(AccountTypeV0Inner::NoValidate), None, None);

        assert_ok!(Starknet::declare(none_origin.clone(), transaction.clone().into()));
        // TODO: Uncomment once we have ABI support
        // assert_eq!(Starknet::contract_class_by_class_hash(erc20_class_hash), erc20_class);
        assert_err!(Starknet::declare(none_origin, transaction.into()), Error::<MockRuntime>::ClassHashAlreadyDeclared);
    });
}

#[test]
fn given_contract_declare_tx_fails_sender_not_deployed() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();
        let chain_id = Starknet::chain_id();

        // Wrong address (not deployed)
        let contract_address = ContractAddress(PatriciaKey(
            StarkFelt::try_from("0x03e437FB56Bb213f5708Fcd6966502070e276c093ec271aA33433b89E21fd31f").unwrap(),
        ));

        let transaction = create_declare_erc20_v0_transaction(
            chain_id,
            AccountType::V0(AccountTypeV0Inner::NoValidate),
            Some(contract_address),
            None,
        );
        assert_err!(Starknet::declare(none_origin, transaction), Error::<MockRuntime>::AccountNotDeployed);
    })
}

#[test]
fn given_contract_declare_on_all_account_types_then_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        for account_type in [AccountTypeV0Inner::Openzeppelin, AccountTypeV0Inner::Argent, AccountTypeV0Inner::Braavos]
        {
            let transaction =
                create_declare_erc20_v0_transaction(Starknet::chain_id(), AccountType::V0(account_type), None, None);
            let contract_class = transaction.class_info.contract_class();
            let class_hash = transaction.tx.class_hash();
            assert_ok!(Starknet::validate_unsigned(
                TransactionSource::InBlock,
                &crate::Call::declare { transaction: transaction.clone() },
            ));

            assert_ok!(Starknet::declare(RuntimeOrigin::none(), transaction));
            assert_eq!(Starknet::contract_class_by_class_hash(class_hash.0).unwrap(), contract_class);
        }
    });
}

#[test]
fn given_contract_declare_on_all_account_types_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        for account_type in [AccountTypeV0Inner::Openzeppelin, AccountTypeV0Inner::Argent, AccountTypeV0Inner::Braavos]
        {
            let transaction = create_declare_erc20_v0_transaction(
                Starknet::chain_id(),
                AccountType::V0(account_type),
                None,
                Some(TransactionSignature(vec![StarkFelt::ZERO, StarkFelt::ONE])),
            );

            assert_matches!(
                Starknet::validate_unsigned(
                    TransactionSource::InBlock,
                    &crate::Call::declare { transaction: transaction.clone() },
                ),
                Err(TransactionValidityError::Invalid(_))
            );

            assert_err!(
                Starknet::declare(RuntimeOrigin::none(), transaction.into()),
                Error::<MockRuntime>::TransactionExecutionFailed
            );
        }
    });
}

#[test]
fn given_contract_declare_on_cairo_1_no_validate_account_then_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let account_addr = get_account_address(None, AccountType::V1(AccountTypeV1Inner::NoValidate));

        let hello_starknet_class = get_contract_class("HelloStarknet.casm.json", 1);
        let hello_starknet_class_hash = ClassHash(
            StarkFelt::try_from("0x05518b17fb5c84683ba37eba8a682b7a6f330911c2216c52c6badff69cc2ec13").unwrap(),
        );
        let hello_starknet_compiled_class_hash = CompiledClassHash(
            StarkFelt::try_from("0x00df4d3042eec107abe704619f13d92bbe01a58029311b7a1886b23dcbb4ea87").unwrap(),
        );

        let mut tx = DeclareTransactionV2 {
            sender_address: account_addr.into(),
            class_hash: hello_starknet_class_hash,
            compiled_class_hash: hello_starknet_compiled_class_hash,
            nonce: Nonce(StarkFelt::ZERO),
            max_fee: Fee(u128::MAX),
            signature: TransactionSignature::default(),
        };
        let tx_hash = tx.compute_hash(Starknet::chain_id(), false);
        tx.signature = sign_message_hash(tx_hash);

        let transaction = BlockifierDeclareTransaction::new(
            StarknetApiDeclareTransaction::V2(tx),
            tx_hash,
            ClassInfo::new(&hello_starknet_class, 1, 1).unwrap(),
        )
        .unwrap();

        assert_ok!(Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::declare { transaction: transaction.clone() },
        ));

        assert_ok!(Starknet::declare(none_origin, transaction.into()));
        assert_eq!(Starknet::contract_class_by_class_hash(hello_starknet_class_hash.0).unwrap(), hello_starknet_class);
    });
}

#[test]
fn test_verify_tx_longevity() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let transaction = create_declare_erc20_v0_transaction(
            Starknet::chain_id(),
            AccountType::V0(AccountTypeV0Inner::NoValidate),
            None,
            None,
        );

        let validate_result =
            Starknet::validate_unsigned(TransactionSource::InBlock, &crate::Call::declare { transaction }).unwrap();

        assert_eq!(validate_result.longevity, TransactionLongevity::get());
    });
}

#[test]
fn test_verify_require_tag() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let transaction = create_declare_erc20_v0_transaction(
            Starknet::chain_id(),
            AccountType::V0(AccountTypeV0Inner::NoValidate),
            None,
            None,
        );

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::declare { transaction: transaction.clone() },
        )
        .unwrap();

        let valid_transaction_expected = ValidTransaction::with_tag_prefix("starknet")
            .priority(u64::MAX)
            .and_provides((*transaction.tx.sender_address(), *transaction.tx.nonce()))
            .longevity(TransactionLongevity::get())
            .propagate(true)
            .and_requires((
                *transaction.tx.sender_address(),
                Felt252Wrapper::from(
                    FieldElement::from(Felt252Wrapper::from(transaction.tx.nonce())) - FieldElement::ONE,
                ),
            ))
            .build()
            .unwrap();

        assert_eq!(validate_result, valid_transaction_expected)
    });
}

#[test]
fn test_verify_nonce_in_unsigned_tx() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let transaction = create_declare_erc20_v0_transaction(
            Starknet::chain_id(),
            AccountType::V0(AccountTypeV0Inner::NoValidate),
            None,
            None,
        );

        let tx_sender = transaction.tx.sender_address();
        let tx_source = TransactionSource::InBlock;
        let call = crate::Call::declare { transaction };

        assert!(Starknet::validate_unsigned(tx_source, &call).is_ok());

        set_nonce::<MockRuntime>(&tx_sender, &Nonce(StarkFelt::from(2u64)));

        assert_eq!(
            Starknet::validate_unsigned(tx_source, &call),
            Err(TransactionValidityError::Invalid(InvalidTransaction::Stale))
        );
    });
}
