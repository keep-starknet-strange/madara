
use starknet_ff::FieldElement;

use crate::*;

#[test]
fn test_sn_goerli_chain_id() {
    let sn_goerli_chain_id = FieldElement::from_byte_slice_be(b"SN_GOERLI").unwrap();
    let mont_sn_goerli_chain_id = FieldElement::into_mont(sn_goerli_chain_id);
    let expected_value: Felt252Wrapper = Felt252Wrapper(starknet_ff::FieldElement::from_mont(mont_sn_goerli_chain_id)).into();
    assert_eq!(
        SN_GOERLI_CHAIN_ID, 
        expected_value, 
        "SN_GOERLI_CHAIN_ID does not match the expected value."
    );
}

#[test]
fn test_sn_main_chain_id() {
    let sn_main_chain_id = FieldElement::from_byte_slice_be(b"SN_MAIN").unwrap();
    let mont_sn_main_chain_id = FieldElement::into_mont(sn_main_chain_id);
    let expected_value: Felt252Wrapper = Felt252Wrapper(starknet_ff::FieldElement::from_mont(mont_sn_main_chain_id)).into();
    assert_eq!(
        SN_MAIN_CHAIN_ID, 
        expected_value, 
        "SN_MAIN_CHAIN_ID does not match the expected value."
    );
}