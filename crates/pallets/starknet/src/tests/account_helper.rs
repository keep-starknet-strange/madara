use hex::FromHex;

use crate::tests::mock::{account_helper, TEST_ACCOUNT_SALT};

#[test]
fn should_calculate_contract_addr_correct() {
    let (addr, _, _) = account_helper(TEST_ACCOUNT_SALT);
    let exp = <[u8; 32]>::from_hex("00b72536305f9a17ed8c0d9abe80e117164589331c3e9547942a830a99d3a5e9").unwrap();
    assert_eq!(addr, exp);
}

#[test]
fn given_salt_should_calculate_new_contract_addr() {
    let (addr, _, _) = account_helper("0x00000000000000000000000000000000000000000000000000000000DEADBEEF");
    let exp = <[u8; 32]>::from_hex("00b72536305f9a17ed8c0d9abe80e117164589331c3e9547942a830a99d3a5e9").unwrap();
    assert_ne!(addr, exp);
}
