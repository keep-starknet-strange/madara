%lang starknet

from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.bool import TRUE
from starkware.cairo.common.cairo_builtins import HashBuiltin, SignatureBuiltin
from starkware.starknet.common.syscalls import (
    get_block_number,
    get_block_timestamp,
    get_contract_address,
    get_tx_info,
    library_call,
)
from starkware.cairo.common.math import assert_not_zero
from starkware.cairo.common.math_cmp import is_not_zero

from src.proxy.library import Proxy
from src.accounts.braavos.library import Account, AccountCallArray, Account_execution_time_delay_sec
from src.accounts.braavos.multisig.library import (
    DeferredMultisigDisableRequest,
    Multisig,
    Multisig_num_signers,
    PendingMultisigTransaction,
)
from src.accounts.braavos.signers.library import (
    Account_signers_num_hw_signers,
    DeferredRemoveSignerRequest,
    IndexedSignerModel,
    Signers,
    SignerModel,
)
from src.accounts.braavos.constants import (
    ACCOUNT_IMPL_VERSION,
    IACCOUNT_ID,
    SUPPORTS_INTERFACE_SELECTOR,
)
from src.accounts.braavos.guards import Guards

// Account specific
@view
func supportsInterface{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    interfaceId: felt
) -> (success: felt) {
    return Account.supports_interface(interfaceId);
}

@view
func get_impl_version{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}() -> (
    res: felt
) {
    return (ACCOUNT_IMPL_VERSION,);
}

// Init & Upgrade
@external
func initializer{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    public_key: felt
) -> () {
    let (proxy_admin) = get_contract_address();
    // NOTE!! Proxy.initializer asserts if account was already initialized
    // DO NOT REMOVE THE Proxy.initializer line below!
    Proxy.initializer(proxy_admin);
    Account.initializer(public_key);

    return ();
}

@external
func upgrade{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    new_implementation: felt
) -> () {
    Proxy.assert_only_admin();

    Account.upgrade(new_implementation);
    return ();
}

@external
func migrate_storage{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    from_version: felt
) -> () {
    Proxy.assert_only_admin();

    Account.migrate_storage(from_version);
    return ();
}

// Signers Entrypoints
@external
func add_signer{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    signer: SignerModel
) -> (signer_id: felt) {
    Guards.assert_only_self();

    return Signers.add_signer(signer);
}

@external
func swap_signers{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    remove_index: felt, added_signer: SignerModel
) -> (signer_id: felt) {
    Guards.assert_only_self();

    let (multisig_num_signers) = Multisig.get_multisig_num_signers();
    return Signers.swap_signers(remove_index, added_signer, is_not_zero(multisig_num_signers));
}

@external
func setPublicKey{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    newPublicKey: felt
) -> () {
    Guards.assert_only_self();

    with_attr error_message("Account: setPublicKey is not supported") {
        assert_not_zero(0);
    }
    return ();
}

@external
func remove_signer{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    index: felt
) -> () {
    Guards.assert_only_self();

    Signers.remove_signer(index);
    // Since we only support 2 signers, successful removal of additional signer
    // necessarily means that we need to disable multisig
    Multisig.disable_multisig();
    return ();
}

@external
func remove_signer_with_etd{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    index: felt
) -> () {
    Guards.assert_only_self();
    let (account_etd) = Account_execution_time_delay_sec.read();

    Signers.remove_signer_with_etd(index, account_etd);
    return ();
}

@external
func cancel_deferred_remove_signer_req{
    syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr
}(removed_signer_id: felt) -> () {
    Guards.assert_only_self();

    Signers.cancel_deferred_remove_signer_req(removed_signer_id);
    return ();
}

@view
func getPublicKey{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}() -> (
    publicKey: felt
) {
    Account._migrate_storage_if_needed();

    let (seed_signer) = Signers.get_signer(0);
    return (publicKey=seed_signer.signer_0);
}

// Backward Compatibility
@view
func get_public_key{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}() -> (
    res: felt
) {
    let (public_key) = getPublicKey();
    return (public_key,);
}

@view
func get_signers{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}() -> (
    signers_len: felt, signers: IndexedSignerModel*
) {
    Account._migrate_storage_if_needed();

    return Signers.get_signers();
}

@view
func get_signer{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(index: felt) -> (
    signer: SignerModel
) {
    Account._migrate_storage_if_needed();

    return Signers.get_signer(index);
}

@view
func get_deferred_remove_signer_req{
    syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr
}() -> (deferred_request: DeferredRemoveSignerRequest) {
    return Signers.get_deferred_remove_signer_req();
}

@view
func get_execution_time_delay{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    ) -> (etd_sec: felt) {
    Account._migrate_storage_if_needed();

    return Account.get_execution_time_delay();
}

// Backward compatibility
@view
func is_valid_signature{
    syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, ecdsa_ptr: SignatureBuiltin*, range_check_ptr
}(hash: felt, signature_len: felt, signature: felt*) -> (is_valid: felt) {
    let (isValid) = isValidSignature(hash, signature_len, signature);
    return (is_valid=isValid);
}

@view
func isValidSignature{
    syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, ecdsa_ptr: SignatureBuiltin*, range_check_ptr
}(hash: felt, signature_len: felt, signature: felt*) -> (isValid: felt) {
    Account._migrate_storage_if_needed();

    let (isValid: felt) = Signers.is_valid_signature(hash, signature_len, signature);
    return (isValid=isValid);
}

// Multisig Entrypoints

@view
func get_multisig{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}() -> (
    multisig_num_signers: felt
) {
    let (multisig_num_signers) = Multisig.get_multisig_num_signers();
    return (multisig_num_signers=multisig_num_signers);
}

@external
func set_multisig{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    num_signers: felt
) -> () {
    Guards.assert_only_self();

    let (num_account_signers) = Account_signers_num_hw_signers.read();
    Multisig.set_multisig(num_signers, num_account_signers);
    return ();
}

@view
func get_pending_multisig_transaction{
    syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr
}() -> (pending_multisig_transaction: PendingMultisigTransaction) {
    let (pending_multisig_transaction) = Multisig.get_pending_multisig_transaction();
    return (pending_multisig_transaction=pending_multisig_transaction);
}

@external
func sign_pending_multisig_transaction{
    syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr
}(
    pending_calldata_len: felt,
    pending_calldata: felt*,
    pending_nonce: felt,
    pending_max_fee: felt,
    pending_transaction_version: felt,
) -> (response_len: felt, response: felt*) {
    Guards.assert_only_self();

    return Multisig.sign_pending_multisig_transaction(
        pending_calldata_len,
        pending_calldata,
        pending_nonce,
        pending_max_fee,
        pending_transaction_version,
    );
}

@external
func disable_multisig{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}() -> () {
    Guards.assert_only_self();

    return Multisig.disable_multisig();
}

@external
func disable_multisig_with_etd{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    ) -> () {
    Guards.assert_only_self();

    let (account_etd) = Account_execution_time_delay_sec.read();
    return Multisig.disable_multisig_with_etd(account_etd);
}

@view
func get_deferred_disable_multisig_req{
    syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr
}() -> (deferred_request: DeferredMultisigDisableRequest) {
    return Multisig.get_deferred_disable_multisig_req();
}

@external
func cancel_deferred_disable_multisig_req{
    syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr
}() -> () {
    Guards.assert_only_self();

    return Multisig.cancel_deferred_disable_multisig_req();
}

// Account entrypoints
@external
func __validate__{
    syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, ecdsa_ptr: SignatureBuiltin*, range_check_ptr
}(call_array_len: felt, call_array: AccountCallArray*, calldata_len: felt, calldata: felt*) -> () {
    alloc_locals;
    let (local block_timestamp) = get_block_timestamp();
    let (local block_num) = get_block_number();
    let (local tx_info) = get_tx_info();

    // Account state House Keeping
    Account._migrate_storage_if_needed();
    Multisig.apply_elapsed_etd_requests(block_timestamp);
    Signers.apply_elapsed_etd_requests(block_timestamp);

    let (account_valid) = Account.account_validate(
        call_array_len, call_array, calldata_len, calldata, tx_info
    );
    assert account_valid = TRUE;

    let (multisig_valid, in_multisig_mode) = Multisig.multisig_validate(
        call_array_len, call_array, calldata_len, calldata, tx_info, block_timestamp, block_num
    );
    assert multisig_valid = TRUE;

    let (signers_valid) = Signers.signers_validate(
        call_array_len,
        call_array[0].to,
        call_array[0].selector,
        calldata_len,
        calldata,
        tx_info,
        block_timestamp,
        block_num,
        in_multisig_mode,
    );
    assert signers_valid = TRUE;

    return ();
}

@external
func __validate_deploy__{
    syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr, ecdsa_ptr: SignatureBuiltin*
}(
    class_hash: felt,
    contract_address_salt: felt,
    implementation_address: felt,
    initializer_selector: felt,
    calldata_len: felt,
    calldata: felt*,
) -> () {
    let (tx_info) = get_tx_info();
    Account.validate_deploy(
        class_hash,
        contract_address_salt,
        implementation_address,
        initializer_selector,
        calldata_len,
        calldata,
    );
    return ();
}

@external
func __validate_declare__{
    syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, ecdsa_ptr: SignatureBuiltin*, range_check_ptr
}(class_hash: felt) -> () {
    let (num_additional_signers) = Account_signers_num_hw_signers.read();
    let (num_multisig_signers) = Multisig_num_signers.read();
    with_attr error_message("Account: declare not supported in non-seed modes") {
        assert num_additional_signers + num_multisig_signers = 0;
    }
    let (tx_info) = get_tx_info();
    with_attr error_message("Account: declare invalid signature") {
        Signers.is_valid_signature(
            tx_info.transaction_hash, tx_info.signature_len, tx_info.signature
        );
    }
    return ();
}

@external
func __execute__{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    call_array_len: felt, call_array: AccountCallArray*, calldata_len: felt, calldata: felt*
) -> (response_len: felt, response: felt*) {
    alloc_locals;
    let (local tx_info) = get_tx_info();

    Guards.assert_no_reentrance();

    // We need to put it here since __validate__ is not called
    // in txn v0 -
    // https://twitter.com/yoavgaziel/status/1594797195538141184
    // should be removed when v0 is dropped
    Guards.assert_valid_transaction_version(tx_info);

    // Handle multisig case (currently only 1 additional signer)
    let (multisig_deferred) = Multisig.multisig_execute(call_array[0].selector, tx_info);
    if (multisig_deferred == TRUE) {
        let (empty_resp: felt*) = alloc();
        return (response_len=0, response=empty_resp);
    }

    let (response_len, response) = Account.execute(
        call_array_len, call_array, calldata_len, calldata
    );
    return (response_len, response);
}
