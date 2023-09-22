use mp_felt::Felt252Wrapper;

use super::mock::AccountType;
use crate::tests::mock::{get_account_address, AccountTypeV0Inner};

#[test]
fn given_salt_should_calculate_new_contract_addr() {
    let salt = Felt252Wrapper::from_hex_be("0x000000000000000000000000000000000000000000000000000000000000BEEF")
        .unwrap()
        .into();
    let addr_0 = get_account_address(salt, AccountType::V0(AccountTypeV0Inner::Argent));
    let salt = Felt252Wrapper::from_hex_be("0x000000000000000000000000000000000000000000000000000000000000DEAD")
        .unwrap()
        .into();
    let addr_1 = get_account_address(salt, AccountType::V0(AccountTypeV0Inner::Argent));
    assert_ne!(addr_0, addr_1);
}
