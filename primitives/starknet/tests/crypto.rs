use kp_starknet::crypto::commitment::{calculate_transaction_commitment, Transaction};
use sp_core::H256;
use starknet_crypto::FieldElement;

#[test]
fn test_merkle_tree() {
    let txs = vec![
        Transaction {
            tx_hash: H256::from_low_u64_be(6),
            signature: vec![H256::from_low_u64_be(10), H256::from_low_u64_be(20), H256::from_low_u64_be(30)],
        },
        Transaction { tx_hash: H256::from_low_u64_be(28), signature: vec![H256::from_low_u64_be(40)] },
    ];
    let com = calculate_transaction_commitment(&txs);
    assert_eq!(
        FieldElement::from_hex_be("0x054c0fddf3aaf1ca03271712b323822647b66042ccc418ba1d7fb852aebfd2da").unwrap(),
        com
    )
}
