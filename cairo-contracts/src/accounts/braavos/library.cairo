%lang starknet

from starkware.cairo.common.cairo_builtins import HashBuiltin, SignatureBuiltin
from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.hash import hash2
from starkware.cairo.common.memcpy import memcpy
from starkware.starknet.common.syscalls import call_contract, get_tx_info, library_call, TxInfo
from starkware.cairo.common.math import assert_not_equal
from starkware.cairo.common.math_cmp import is_not_zero
from starkware.cairo.common.bool import TRUE, FALSE

from src.proxy.library import Proxy
from src.accounts.braavos.migrations.library import Migrations
from src.accounts.braavos.signers.library import (
    Account_public_key,
    Account_signers,
    Account_signers_max_index,
    Signers,
    SignerModel,
)
from src.accounts.braavos.constants import (
    ACCOUNT_DEFAULT_EXECUTION_TIME_DELAY_SEC,
    ACCOUNT_IMPL_VERSION,
    ADD_SIGNER_SELECTOR,
    CANCEL_DEFERRED_DISABLE_MULTISIG_REQ_SELECTOR,
    CANCEL_DEFERRED_REMOVE_SIGNER_REQ_SELECTOR,
    DISABLE_MULTISIG_SELECTOR,
    DISABLE_MULTISIG_WITH_ETD_SELECTOR,
    IACCOUNT_ID,
    IACCOUNT_ID_v0x1010102,
    IERC165_ID,
    MIGRATE_STORAGE_SELECTOR,
    SET_MULTISIG_SELECTOR,
    REMOVE_SIGNER_SELECTOR,
    REMOVE_SIGNER_WITH_ETD_SELECTOR,
    SIGNER_TYPE_STARK,
    SUPPORTS_INTERFACE_SELECTOR,
)

// Structs
struct Call {
    to: felt,
    selector: felt,
    calldata_len: felt,
    calldata: felt*,
}

// Support passing `[AccountCall]` to __execute__
struct AccountCallArray {
    to: felt,
    selector: felt,
    data_offset: felt,
    data_len: felt,
}

// Events
@event
func AccountInitialized(public_key: felt) {
}

// Storage
@storage_var
func Account_execution_time_delay_sec() -> (etd: felt) {
}

@storage_var
func Account_storage_migration_version() -> (res: felt) {
}

namespace Account {
    func initializer{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
        public_key: felt
    ) -> () {
        alloc_locals;
        let signer = SignerModel(
            signer_0=public_key,
            signer_1=0,
            signer_2=0,
            signer_3=0,
            type=SIGNER_TYPE_STARK,
            reserved_0=0,
            reserved_1=0,
        );

        Account_signers.write(0, signer);
        Account_signers_max_index.write(0);
        Account_execution_time_delay_sec.write(ACCOUNT_DEFAULT_EXECUTION_TIME_DELAY_SEC);

        let (tx_info) = get_tx_info();
        let (_: felt, additional_signer: SignerModel) = parse_initializer_signature_aux_data(
            tx_info.signature_len, tx_info.signature
        );

        // additional signer provided, so set it up
        let have_signer = is_not_zero(additional_signer.type);
        if (have_signer == TRUE) {
            Signers.add_signer(additional_signer);
            tempvar syscall_ptr = syscall_ptr;
            tempvar pedersen_ptr = pedersen_ptr;
            tempvar range_check_ptr = range_check_ptr;
        } else {
            tempvar syscall_ptr = syscall_ptr;
            tempvar pedersen_ptr = pedersen_ptr;
            tempvar range_check_ptr = range_check_ptr;
        }

        Account_storage_migration_version.write(ACCOUNT_IMPL_VERSION);
        AccountInitialized.emit(public_key);
        return ();
    }

    func upgrade{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
        new_implementation: felt
    ) -> () {
        Proxy.assert_only_admin();
        let (calldata) = alloc();

        // Verify new_implementation contract is an account contract
        assert [calldata] = IACCOUNT_ID;
        let (retdata_size: felt, retdata: felt*) = library_call(
            class_hash=new_implementation,
            function_selector=SUPPORTS_INTERFACE_SELECTOR,
            calldata_size=1,
            calldata=calldata,
        );

        with_attr error_message("Account: Implementation does not support IACCOUNT_ID") {
            assert retdata[0] = TRUE;
        }

        Proxy._set_implementation(new_implementation);

        // Migrate data model (if necessary)
        assert [calldata + 1] = ACCOUNT_IMPL_VERSION;
        let (retdata_size: felt, retdata: felt*) = library_call(
            class_hash=new_implementation,
            function_selector=MIGRATE_STORAGE_SELECTOR,
            calldata_size=1,
            calldata=calldata + 1,
        );
        return ();
    }

    func migrate_storage{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
        from_version: felt
    ) -> () {
        alloc_locals;
        // Update storage version
        Account_storage_migration_version.write(ACCOUNT_IMPL_VERSION);

        // Data model migration comes here,
        // first version that calls this is b'000.000.006'

        // b'000.000.007', b'000.000.008', b'000.000.009' - no migrations
        with_attr error_message("Account: upgrade data migration failed") {
            if (from_version == '000.000.009') {
                let (res) = Migrations.migrate_000_000_009();
                assert res = TRUE;
            }
        }

        return ();
    }

    func get_execution_time_delay{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
        ) -> (etd_sec: felt) {
        let (etd_sec) = Account_execution_time_delay_sec.read();

        return (etd_sec=etd_sec);
    }

    func assert_multicall_valid(
        self: felt, call_array_len: felt, call_array: AccountCallArray*
    ) -> () {
        // A single call is allowed to anywhere
        if (call_array_len == 1) {
            return ();
        }

        with_attr error_message("Account: multicall with subsequent call to self") {
            // Allowed "call-to-self" multicall combinations
            if ((1 - is_not_zero(call_array_len - 2)) *
                (1 - is_not_zero(call_array[0].to - self)) *
                (1 - is_not_zero(call_array[1].to - self)) == TRUE) {
                // add_signer -> set_multisig
                tempvar as_sm = (1 - is_not_zero(call_array[0].selector - ADD_SIGNER_SELECTOR)) * (
                    1 - is_not_zero(call_array[1].selector - SET_MULTISIG_SELECTOR)
                );
                // disable_multisig -> remove_signer
                tempvar dm_rs = (
                    1 - is_not_zero(call_array[0].selector - DISABLE_MULTISIG_SELECTOR)
                ) * (1 - is_not_zero(call_array[1].selector - REMOVE_SIGNER_SELECTOR));
                // disable_multisig_with_etd -> remove_signer_with_etd
                tempvar dmwe_rswe = (
                    1 - is_not_zero(call_array[0].selector - DISABLE_MULTISIG_WITH_ETD_SELECTOR)
                ) * (1 - is_not_zero(call_array[1].selector - REMOVE_SIGNER_WITH_ETD_SELECTOR));
                // cancel_deferred_disable_multisig_req -> cancel_deferred_remove_signer_req
                tempvar cdrsr_cddmr = (
                    1 -
                    is_not_zero(call_array[0].selector - CANCEL_DEFERRED_REMOVE_SIGNER_REQ_SELECTOR)
                ) * (
                    1 -
                    is_not_zero(
                        call_array[1].selector - CANCEL_DEFERRED_DISABLE_MULTISIG_REQ_SELECTOR
                    )
                );
                // disable_multisig -> cancel_deferred_remove_signer_req
                tempvar dm_cdrsr = (
                    1 - is_not_zero(call_array[0].selector - DISABLE_MULTISIG_SELECTOR)
                ) * (
                    1 -
                    is_not_zero(call_array[1].selector - CANCEL_DEFERRED_REMOVE_SIGNER_REQ_SELECTOR)
                );
                // cancel_deferred_remove_signer_req -> set_multisig
                tempvar cdrsr_sm = (
                    1 -
                    is_not_zero(call_array[0].selector - CANCEL_DEFERRED_REMOVE_SIGNER_REQ_SELECTOR)
                ) * (1 - is_not_zero(call_array[1].selector - SET_MULTISIG_SELECTOR));

                // OR between allowed combinations
                // specific combination == TRUE iff selectors in combination match call array
                assert as_sm + dm_rs + dmwe_rswe + cdrsr_cddmr + dm_cdrsr + cdrsr_sm = 1;
            } else {
                _assert_multicall_valid_inner(self, call_array_len, call_array);
            }
        }

        return ();
    }

    func _assert_multicall_valid_inner(
        self: felt, call_array_len: felt, call_array: AccountCallArray*
    ) -> () {
        if (call_array_len == 0) {
            return ();
        }
        assert_not_equal(call_array[0].to, self);
        _assert_multicall_valid_inner(self, call_array_len - 1, call_array + AccountCallArray.SIZE);
        return ();
    }

    func supports_interface{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
        interface_id: felt
    ) -> (success: felt) {
        if (interface_id == IERC165_ID) {
            return (success=TRUE);
        }
        if (interface_id == IACCOUNT_ID) {
            return (success=TRUE);
        }
        if (interface_id == IACCOUNT_ID_v0x1010102) {
            return (success=TRUE);
        }

        return (success=FALSE);
    }

    func _migrate_storage_if_needed{
        syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr
    }() -> () {
        let (latest_migration) = Account_storage_migration_version.read();
        if (latest_migration != 0) {
            with_attr error_message("Account: account upgraded without migration") {
                assert latest_migration = ACCOUNT_IMPL_VERSION;
            }

            return ();
        }

        // latest_migration == 0, meaning we need to "bootstrap" our storage from an old account
        // We can't use migrate_storage directly as it asserts on proxy admin
        Account_storage_migration_version.write(ACCOUNT_IMPL_VERSION);
        Account_execution_time_delay_sec.write(ACCOUNT_DEFAULT_EXECUTION_TIME_DELAY_SEC);
        let (public_key) = Account_public_key.read();
        if (public_key != 0) {
            // We come from a pre v2.32.2 contract..
            let signer_0 = SignerModel(
                signer_0=public_key,
                signer_1=0,
                signer_2=0,
                signer_3=0,
                type=SIGNER_TYPE_STARK,
                reserved_0=0,
                reserved_1=0,
            );
            Account_signers.write(0, signer_0);
            Account_public_key.write(0);
            return ();  // Avoid revoked refs
        }

        return ();
    }

    // Extract auxiliary data out of txn signature
    // signature[2] -> actual_impl: for no actual_impl, send 0
    // signature[3:10] -> hw_signer: for no hw_signer, send 0's
    func parse_initializer_signature_aux_data(signature_len: felt, signature: felt*) -> (
        actual_impl: felt, hw_signer: SignerModel
    ) {
        with_attr error_message("Account: missing parameters in initializer signature") {
            assert signature_len = 10;
        }
        return (
            actual_impl=signature[2],
            hw_signer=SignerModel(
                signer_0=signature[3],
                signer_1=signature[4],
                signer_2=signature[5],
                signer_3=signature[6],
                type=signature[7],
                reserved_0=signature[8],
                reserved_1=signature[9],
            ),
        );
    }

    func validate_deploy{
        syscall_ptr: felt*,
        pedersen_ptr: HashBuiltin*,
        range_check_ptr,
        ecdsa_ptr: SignatureBuiltin*,
    }(
        class_hash: felt,
        contract_address_salt: felt,
        implementation_address: felt,
        initializer_selector: felt,
        calldata_len: felt,
        calldata: felt*,
    ) -> () {
        // Hash signature aux data
        let (tx_info) = get_tx_info();
        let (actual_impl: felt, hw_signer: SignerModel) = parse_initializer_signature_aux_data(
            tx_info.signature_len, tx_info.signature
        );

        let hash_ptr = pedersen_ptr;
        with hash_ptr {
            // Reconstruct compute_hash_on_elements logic
            let (hash_res) = hash2(0, tx_info.transaction_hash);
            let (hash_res) = hash2(hash_res, actual_impl);
            let (hash_res) = hash2(hash_res, hw_signer.signer_0);
            let (hash_res) = hash2(hash_res, hw_signer.signer_1);
            let (hash_res) = hash2(hash_res, hw_signer.signer_2);
            let (hash_res) = hash2(hash_res, hw_signer.signer_3);
            let (hash_res) = hash2(hash_res, hw_signer.type);
            let (hash_res) = hash2(hash_res, hw_signer.reserved_0);
            let (hash_res) = hash2(hash_res, hw_signer.reserved_1);
            let (hash_res) = hash2(hash_res, 9);
        }
        let pedersen_ptr = hash_ptr;

        // We know that initializer assigned signer idx 0 to be seed signer
        tempvar actual_sig: felt* = new (tx_info.signature[0], tx_info.signature[1]);
        Signers._is_valid_stark_signature(calldata[0], hash_res, 2, actual_sig);

        return ();
    }

    func account_validate{
        syscall_ptr: felt*,
        pedersen_ptr: HashBuiltin*,
        range_check_ptr,
        ecdsa_ptr: SignatureBuiltin*,
    }(
        call_array_len: felt,
        call_array: AccountCallArray*,
        calldata_len: felt,
        calldata: felt*,
        tx_info: TxInfo*,
    ) -> (valid: felt) {
        with_attr error_message("Account: no calls provided") {
            let have_calls = is_not_zero(call_array_len);
            assert have_calls = TRUE;
        }

        // Be defensive about dapps trying to trick the user into signing
        // subsequent account related transactions
        assert_multicall_valid(tx_info.account_contract_address, call_array_len, call_array);

        return (valid=TRUE);
    }

    // Execute
    func execute{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
        call_array_len: felt, call_array: AccountCallArray*, calldata_len: felt, calldata: felt*
    ) -> (response_len: felt, response: felt*) {
        alloc_locals;
        // TMP: Convert `AccountCallArray` to 'Call'.
        let (calls: Call*) = alloc();
        _from_call_array_to_call(call_array_len, call_array, calldata, calls);
        let calls_len = call_array_len;

        // execute call
        let (response: felt*) = alloc();
        let (response_len) = _execute_list(calls_len, calls, response);

        return (response_len=response_len, response=response);
    }

    func _execute_list{syscall_ptr: felt*}(calls_len: felt, calls: Call*, response: felt*) -> (
        response_len: felt
    ) {
        alloc_locals;

        // if no more calls
        if (calls_len == 0) {
            return (response_len=0);
        }

        // do the current call
        let this_call: Call = [calls];
        let res = call_contract(
            contract_address=this_call.to,
            function_selector=this_call.selector,
            calldata_size=this_call.calldata_len,
            calldata=this_call.calldata,
        );
        // copy the result in response
        memcpy(response, res.retdata, res.retdata_size);
        // do the next calls recursively
        let (response_len) = _execute_list(
            calls_len - 1, calls + Call.SIZE, response + res.retdata_size
        );
        return (response_len=response_len + res.retdata_size);
    }

    func _from_call_array_to_call{syscall_ptr: felt*}(
        call_array_len: felt, call_array: AccountCallArray*, calldata: felt*, calls: Call*
    ) -> () {
        // if no more calls
        if (call_array_len == 0) {
            return ();
        }

        // parse the current call
        assert [calls] = Call(
            to=[call_array].to,
            selector=[call_array].selector,
            calldata_len=[call_array].data_len,
            calldata=calldata + [call_array].data_offset,
        );
        // parse the remaining calls recursively
        _from_call_array_to_call(
            call_array_len - 1, call_array + AccountCallArray.SIZE, calldata, calls + Call.SIZE
        );
        return ();
    }
}
