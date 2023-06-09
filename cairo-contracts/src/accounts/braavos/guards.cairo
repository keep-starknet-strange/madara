%lang starknet
from starkware.cairo.common.math import assert_not_equal
from starkware.starknet.common.syscalls import (
    get_caller_address,
    get_contract_address,
    get_tx_info,
    TxInfo,
)

from src.accounts.braavos.constants import (
    TX_VERSION_0_EST_FEE,
)

namespace Guards {
    func assert_only_self{syscall_ptr: felt*}() {
        let (self) = get_contract_address();
        let (caller) = get_caller_address();
        with_attr error_message("Guards: caller is not this account") {
            assert self = caller;
        }
        return ();
    }

    func assert_no_reentrance{syscall_ptr: felt*}() {
        // validate caller - here since __validate__ is only called on tx
        let (caller) = get_caller_address();
        with_attr error_message("Guards: no reentrant call") {
            assert caller = 0;
        }

        return ();
    }

    func assert_valid_transaction_version{syscall_ptr: felt*}(
        tx_info: TxInfo*) {
        with_attr error_message(
                "Please Upgrade Wallet app. Invalid transaction version.") {
            assert_not_equal(tx_info.version, 0);
            assert_not_equal(tx_info.version, TX_VERSION_0_EST_FEE);
        }

        return ();
    }
}
