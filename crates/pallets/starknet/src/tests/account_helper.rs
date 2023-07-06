use super::mock::{account_helper, AccountType};
use crate::tests::mock::AccountTypeV0Inner;

#[test]
fn given_salt_should_calculate_new_contract_addr() {
    let (addr_0, _, _) = account_helper(
        "0x000000000000000000000000000000000000000000000000000000000000BEEF",
        AccountType::V0(AccountTypeV0Inner::Argent),
    );
    let (addr_1, _, _) = account_helper(
        "0x000000000000000000000000000000000000000000000000000000000000DEAD",
        AccountType::V0(AccountTypeV0Inner::Argent),
    );
    assert_ne!(addr_0, addr_1);
}
