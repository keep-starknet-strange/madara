use starknet_api::hash::StarkFelt;
use starknet_api::transaction::ContractAddressSalt;

use super::mock::AccountType;
use crate::tests::mock::{get_account_address, AccountTypeV0Inner};

#[test]
fn given_salt_should_calculate_new_contract_addr() {
    let salt = ContractAddressSalt(
        StarkFelt::try_from("0x000000000000000000000000000000000000000000000000000000000000BEEF").unwrap(),
    );
    let addr_0 = get_account_address(Some(salt), AccountType::V0(AccountTypeV0Inner::Argent));
    let salt = ContractAddressSalt(
        StarkFelt::try_from("0x000000000000000000000000000000000000000000000000000000000000DEAD").unwrap(),
    );
    let addr_1 = get_account_address(Some(salt), AccountType::V0(AccountTypeV0Inner::Argent));
    assert_ne!(addr_0, addr_1);
}
