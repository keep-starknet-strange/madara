use mp_starknet::execution::types::Felt252Wrapper;

use super::mock::{account_helper, AccountType};
use crate::tests::mock::AccountTypeV0Inner;

#[test]
fn given_salt_should_calculate_new_contract_addr() {
    let salt =
        Felt252Wrapper::from_hex_be("0x000000000000000000000000000000000000000000000000000000000000BEEF").unwrap();
    let (addr_0, _, _) = account_helper(salt, AccountType::V0(AccountTypeV0Inner::Argent));
    let salt =
        Felt252Wrapper::from_hex_be("0x000000000000000000000000000000000000000000000000000000000000DEAD").unwrap();
    let (addr_1, _, _) = account_helper(salt, AccountType::V0(AccountTypeV0Inner::Argent));
    assert_ne!(addr_0, addr_1);
}
