%lang starknet

from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.bool import TRUE, FALSE
from starkware.cairo.common.cairo_builtins import HashBuiltin, SignatureBuiltin
from starkware.cairo.common.hash_state import (
    hash_init,
    hash_update,
    hash_update_single,
    hash_update_with_hashchain,
    hash_finalize,
)
from starkware.cairo.common.math import assert_not_zero
from starkware.cairo.common.math_cmp import is_le_felt, is_not_zero
from starkware.starknet.common.constants import INVOKE_HASH_PREFIX
from starkware.starknet.common.syscalls import (
    emit_event,
    get_block_number,
    get_block_timestamp,
    get_contract_address,
    get_tx_info,
    TxInfo,
)

from src.accounts.braavos.library import Account, AccountCallArray, Call
from src.accounts.braavos.signers.library import Account_signers_num_hw_signers, Signers
from src.accounts.braavos.constants import (
    ACCOUNT_DEFAULT_EXECUTION_TIME_DELAY_SEC,
    DISABLE_MULTISIG_SELECTOR,
    DISABLE_MULTISIG_WITH_ETD_SELECTOR,
    MULTISIG_PENDING_TXN_EXPIRY_BLOCK_NUM,
    MULTISIG_PENDING_TXN_EXPIRY_SEC,
    REMOVE_SIGNER_WITH_ETD_SELECTOR,
    SIGN_PENDING_MULTISIG_TXN_SELECTOR,
    SIGNER_TYPE_STARK,
    SIGNER_TYPE_UNUSED,
    TX_VERSION_1_EST_FEE,
)

// Structs
struct PendingMultisigTransaction {
    transaction_hash: felt,
    expire_at_sec: felt,
    expire_at_block_num: felt,
    // Currently support only 2 signers (seed + additional signer)
    // so no need to keep track of multiple signers - in the future:
    // signers: felt* (this is not possible in Starknet storage, maybe a bit map?)
    signer_1_id: felt,
    // We need to know whether pending multisig txn is disable to prevent
    // censorship when seed is stolen - see _authorize_signer
    is_disable_multisig_transaction: felt,
}

struct DeferredMultisigDisableRequest {
    expire_at: felt,
}

// Events
@event
func MultisigDisableRequest(request: DeferredMultisigDisableRequest) {
}

@event
func MultisigDisableRequestCancelled(request: DeferredMultisigDisableRequest) {
}

@event
func MultisigSet(num_signers: felt) {
}

@event
func MultisigDisabled() {
}

// We dont use @event because we want more than 1 key in the events
const MultisigPendingTransactionSelector = 1076481841203195901192246052515948214390765227783939297815575703989242392013;
const MultisigPendingTransactionSignedSelector = 77148960833872616285480930780499646942191152514328985919763224338929016653;

// Storage
@storage_var
func Multisig_num_signers() -> (res: felt) {
}

@storage_var
func Multisig_pending_transaction() -> (res: PendingMultisigTransaction) {
}

@storage_var
func Multisig_deferred_disable_request() -> (res: DeferredMultisigDisableRequest) {
}

namespace Multisig {
    func set_multisig{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
        num_multisig_signers: felt, num_account_signers: felt
    ) -> () {
        with_attr error_message("Multisig: multisig currently supports 2 signers only") {
            assert num_multisig_signers = 2;
        }

        with_attr error_message(
                "Multisig: multisig can only be set if account have additional signers") {
            assert num_account_signers = 1;
        }

        with_attr error_message("Multisig: multisig was already set") {
            let (multisig_signers) = Multisig_num_signers.read();
            assert multisig_signers = 0;
        }

        Multisig_num_signers.write(num_multisig_signers);
        MultisigSet.emit(num_multisig_signers);

        return ();
    }

    func get_multisig_num_signers{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
        ) -> (multisig_num_signers: felt) {
        let (multisig_signers) = Multisig_num_signers.read();

        return (multisig_num_signers=multisig_signers);
    }

    func multisig_execute{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
        selector: felt, tx_info: TxInfo*
    ) -> (multisig_deferred: felt) {
        alloc_locals;
        let (multisig_num_signers) = Multisig_num_signers.read();

        if (multisig_num_signers == 0) {
            return (multisig_deferred=FALSE);
        }
        let (block_timestamp) = get_block_timestamp();
        let (block_num) = get_block_number();
        let (local current_signer) = Signers.resolve_signer_from_sig(
            tx_info.signature_len, tx_info.signature
        );

        let (pending_multisig_txn: PendingMultisigTransaction) = Multisig_pending_transaction.read(
            );
        tempvar is_disable_multisig_selector = 1 - is_not_zero(
            selector - DISABLE_MULTISIG_SELECTOR
        );

        // selector values below should be handled in current execute flow and not be deferred
        // since we are checking on selector, only one of these will be 1 or all 0
        let allowed_selector = is_allowed_selector_for_seed_in_multisig(selector);
        if (allowed_selector == TRUE) {
            return (multisig_deferred=FALSE);
        }

        // Create / Override pending txn
        let expire_at_sec = block_timestamp + MULTISIG_PENDING_TXN_EXPIRY_SEC;
        let expire_at_block_num = block_num + MULTISIG_PENDING_TXN_EXPIRY_BLOCK_NUM;

        let pendingTxn = PendingMultisigTransaction(
            transaction_hash=tx_info.transaction_hash,
            expire_at_sec=expire_at_sec,
            expire_at_block_num=expire_at_block_num,
            signer_1_id=current_signer.index,
            is_disable_multisig_transaction=is_disable_multisig_selector,
        );
        Multisig_pending_transaction.write(pendingTxn);

        let (local pendingTxnEvtKeys: felt*) = alloc();
        assert [pendingTxnEvtKeys] = MultisigPendingTransactionSelector;
        assert [pendingTxnEvtKeys + 1] = current_signer.index;
        let (local pendingTxnEvtData: felt*) = alloc();
        assert [pendingTxnEvtData] = tx_info.transaction_hash;
        assert [pendingTxnEvtData + 1] = expire_at_sec;
        assert [pendingTxnEvtData + 2] = expire_at_block_num;
        emit_event(2, pendingTxnEvtKeys, 3, pendingTxnEvtData);
        return (multisig_deferred=TRUE);
    }

    func get_pending_multisig_transaction{
        syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr
    }() -> (pending_multisig_transaction: PendingMultisigTransaction) {
        let (pending_multisig_transaction) = Multisig_pending_transaction.read();

        return (pending_multisig_transaction=pending_multisig_transaction);
    }

    func sign_pending_multisig_transaction{
        syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr
    }(
        pending_calldata_len: felt,
        pending_calldata: felt*,
        pending_nonce: felt,
        pending_max_fee: felt,
        pending_transaction_version: felt,
    ) -> (response_len: felt, response: felt*) {
        alloc_locals;

        let (pending_multisig_transaction) = Multisig_pending_transaction.read();
        let (local tx_info) = get_tx_info();

        let is_estfee = is_le_felt(TX_VERSION_1_EST_FEE, tx_info.version);
        // Let estimate fee pass for 2nd signer even when txn is still in RECEIVED state
        if (is_estfee == FALSE) {
            with_attr error_message("Multisig: no pending multisig transaction") {
                assert is_not_zero(pending_multisig_transaction.transaction_hash) = TRUE;
            }
        }
        let (current_signer) = Signers.resolve_signer_from_sig(
            tx_info.signature_len, tx_info.signature
        );

        // Let estimate fee pass for 2nd signer even when txn is still in RECEIVED state
        if (is_estfee == FALSE) {
            with_attr error_message("Multisig: multisig signer can only sign once") {
                assert is_not_zero(
                    current_signer.index - pending_multisig_transaction.signer_1_id
                ) = TRUE;
            }
        }

        tempvar nonce_as_additional_data: felt* = new (pending_nonce);
        let (self) = get_contract_address();
        with_attr error_message("Multisig: multisig invalid hash") {
            let hash_ptr = pedersen_ptr;
            with hash_ptr {
                let (computed_hash) = _compute_hash(
                    self,
                    pending_calldata_len,
                    pending_calldata,
                    pending_nonce,
                    pending_max_fee,
                    pending_transaction_version,
                    tx_info.chain_id,
                    nonce_as_additional_data,
                );
            }
            let pedersen_ptr = hash_ptr;

            // Let estimate fee pass for 2nd signer even when txn is still in RECEIVED state
            if (is_estfee == FALSE) {
                assert computed_hash = pending_multisig_transaction.transaction_hash;
            }
        }

        // clear the pending txn and emit the event
        Multisig_pending_transaction.write(
            PendingMultisigTransaction(
                transaction_hash=0,
                expire_at_sec=0,
                expire_at_block_num=0,
                signer_1_id=0,
                is_disable_multisig_transaction=0,
            ),
        );
        let (local pendingTxnSignedEvtKeys: felt*) = alloc();
        assert [pendingTxnSignedEvtKeys] = MultisigPendingTransactionSignedSelector;
        assert [pendingTxnSignedEvtKeys + 1] = computed_hash;
        let (local pendingTxnSignedEvtData: felt*) = alloc();
        assert [pendingTxnSignedEvtData] = current_signer.index;
        emit_event(2, pendingTxnSignedEvtKeys, 1, pendingTxnSignedEvtData);

        // Convert `AccountCallArray` to 'Call'
        // we know pending_calldata is compatible with __execute__'s input
        let call_array_len = pending_calldata[0];
        let call_array = cast(pending_calldata + 1, AccountCallArray*);
        let (calls: Call*) = alloc();
        Account._from_call_array_to_call(
            call_array_len,
            call_array,
            pending_calldata + call_array_len * AccountCallArray.SIZE + 2,
            calls,
        );
        let calls_len = pending_calldata[0];

        // execute call
        let (response: felt*) = alloc();
        let (response_len) = Account._execute_list(calls_len, calls, response);

        return (response_len=response_len, response=response);
    }

    func _compute_hash{syscall_ptr: felt*, hash_ptr: HashBuiltin*, range_check_ptr}(
        contract_address: felt,
        pending_calldata_len: felt,
        pending_calldata: felt*,
        pending_nonce: felt,
        pending_max_fee: felt,
        pending_transaction_version: felt,
        chain_id: felt,
        additional_data: felt*,
    ) -> (computed_hash: felt) {
        let (hash_state_ptr) = hash_init();
        let (hash_state_ptr) = hash_update_single(
            hash_state_ptr=hash_state_ptr, item=INVOKE_HASH_PREFIX
        );
        let (hash_state_ptr) = hash_update_single(
            hash_state_ptr=hash_state_ptr, item=pending_transaction_version
        );
        let (hash_state_ptr) = hash_update_single(
            hash_state_ptr=hash_state_ptr, item=contract_address
        );
        let (hash_state_ptr) = hash_update_single(hash_state_ptr=hash_state_ptr, item=0);
        let (hash_state_ptr) = hash_update_with_hashchain(
            hash_state_ptr=hash_state_ptr,
            data_ptr=pending_calldata,
            data_length=pending_calldata_len,
        );
        let (hash_state_ptr) = hash_update_single(
            hash_state_ptr=hash_state_ptr, item=pending_max_fee
        );
        let (hash_state_ptr) = hash_update_single(hash_state_ptr=hash_state_ptr, item=chain_id);

        let (hash_state_ptr) = hash_update(
            hash_state_ptr=hash_state_ptr, data_ptr=additional_data, data_length=1
        );

        let (computed_hash) = hash_finalize(hash_state_ptr=hash_state_ptr);

        return (computed_hash=computed_hash);
    }

    func disable_multisig{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}() -> () {
        // Discard any pending multisig txn
        Multisig_pending_transaction.write(
            PendingMultisigTransaction(
                transaction_hash=0,
                expire_at_sec=0,
                expire_at_block_num=0,
                signer_1_id=0,
                is_disable_multisig_transaction=0,
            ),
        );

        // Remove multisig signer indication
        Multisig_num_signers.write(0);
        MultisigDisabled.emit();
        return ();
    }

    func disable_multisig_with_etd{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
        account_etd: felt
    ) -> () {
        // For now we limit this API to seed signer only as it has no functional
        // meaning with secp256r1
        let (tx_info) = get_tx_info();
        let (current_signer) = Signers.resolve_signer_from_sig(
            tx_info.signature_len, tx_info.signature
        );
        with_attr error_message(
                "Multisig: disable_multisig_with_etd should be called with seed signer") {
            assert current_signer.signer.type = SIGNER_TYPE_STARK;
        }

        // We dont want to allow endless postponement of etd removals, once
        // there's an etd it should either finish or cancelled
        let (disable_multisig_req) = Multisig_deferred_disable_request.read();
        with_attr error_message("Multisig: already have a pending disable multisig request") {
            assert disable_multisig_req.expire_at = 0;
        }

        let (block_timestamp) = get_block_timestamp();
        with_attr error_message("Multisig: etd not initialized") {
            assert_not_zero(account_etd);
        }
        let expire_at = block_timestamp + account_etd;
        let remove_req = DeferredMultisigDisableRequest(expire_at=expire_at);
        Multisig_deferred_disable_request.write(remove_req);
        MultisigDisableRequest.emit(remove_req);

        return ();
    }

    func get_deferred_disable_multisig_req{
        syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr
    }() -> (deferred_request: DeferredMultisigDisableRequest) {
        let (deferred_request) = Multisig_deferred_disable_request.read();
        return (deferred_request=deferred_request);
    }

    func cancel_deferred_disable_multisig_req{
        syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr
    }() -> () {
        let (deferred_request) = Multisig_deferred_disable_request.read();

        with_attr error_message("Multisig: no deferred disable multisig req") {
            assert_not_zero(deferred_request.expire_at);
        }

        Multisig_deferred_disable_request.write(DeferredMultisigDisableRequest(expire_at=0));
        MultisigDisableRequestCancelled.emit(deferred_request);

        return ();
    }

    func is_allowed_selector_for_seed_in_multisig(selector: felt) -> felt {
        tempvar is_sign_pending_selector = 1 - is_not_zero(
            selector - SIGN_PENDING_MULTISIG_TXN_SELECTOR
        );
        tempvar is_disable_multisig_with_etd_selector = 1 - is_not_zero(
            selector - DISABLE_MULTISIG_WITH_ETD_SELECTOR
        );
        tempvar is_remove_signer_with_etd_selector = 1 - is_not_zero(
            selector - REMOVE_SIGNER_WITH_ETD_SELECTOR
        );
        // Only one of the above will be 1 as we are comparing the same selector
        return (
            is_sign_pending_selector +
            is_disable_multisig_with_etd_selector +
            is_remove_signer_with_etd_selector
        );
    }

    func discard_expired_multisig_pending_transaction{
        syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr
    }(pending_multisig_txn: PendingMultisigTransaction, block_num: felt, block_timestamp: felt) -> (
        processed_pending_txn: PendingMultisigTransaction
    ) {
        if (pending_multisig_txn.transaction_hash == 0) {
            return (processed_pending_txn=pending_multisig_txn);
        }

        // only if both block and time elapsed then discard the pending txn
        let expiry_block_num_expired = is_le_felt(
            pending_multisig_txn.expire_at_block_num, block_num
        );
        let expiry_sec_expired = is_le_felt(pending_multisig_txn.expire_at_sec, block_timestamp);
        if (expiry_block_num_expired * expiry_sec_expired == TRUE) {
            let empty_pending_txn = PendingMultisigTransaction(
                transaction_hash=0,
                expire_at_sec=0,
                expire_at_block_num=0,
                signer_1_id=0,
                is_disable_multisig_transaction=0,
            );
            Multisig_pending_transaction.write(empty_pending_txn);
            return (processed_pending_txn=empty_pending_txn);
        }

        return (processed_pending_txn=pending_multisig_txn);
    }

    func apply_elapsed_etd_requests{
        syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr
    }(block_timestamp: felt) -> () {
        let (disable_multisig_req) = Multisig_deferred_disable_request.read();
        let have_disable_multisig_etd = is_not_zero(disable_multisig_req.expire_at);
        let disable_multisig_etd_expired = is_le_felt(
            disable_multisig_req.expire_at, block_timestamp
        );

        if (have_disable_multisig_etd * disable_multisig_etd_expired == TRUE) {
            disable_multisig();
            return ();
        }

        return ();
    }

    func multisig_validate{
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
        block_timestamp: felt,
        block_num: felt,
    ) -> (valid: felt, is_multisig_mode: felt) {
        alloc_locals;

        let (num_multisig_signers) = Multisig_num_signers.read();
        let is_multisig_mode = is_not_zero(num_multisig_signers);
        if (is_multisig_mode == FALSE) {
            return (valid=TRUE, is_multisig_mode=FALSE);
        }

        let (num_additional_signers) = Account_signers_num_hw_signers.read();
        let have_additional_signers = is_not_zero(num_additional_signers);
        if (have_additional_signers == FALSE) {
            // This will happen when remove signer with etd was not bundled
            // with a disable multisig with etd, so we handle it here.
            disable_multisig();
            return (valid=TRUE, is_multisig_mode=FALSE);
        }

        let (pending_multisig_txn) = Multisig_pending_transaction.read();
        let (pending_multisig_txn) = discard_expired_multisig_pending_transaction(
            pending_multisig_txn, block_num, block_timestamp
        );
        let (local current_signer) = Signers.resolve_signer_from_sig(
            tx_info.signature_len, tx_info.signature
        );

        tempvar is_stark_signer = 1 - is_not_zero(current_signer.signer.type - SIGNER_TYPE_STARK);

        // Protect against censorship when seed is stolen and tries to override
        // pending multisig txns preventing the second signer from recovering the account.
        // In this case, seed is only allowed to approve the txn or do ETD actions
        let is_pending_txn_diff_signer = is_not_zero(
            pending_multisig_txn.signer_1_id - current_signer.index
        );
        with_attr error_message("Multisig: invalid entry point for seed signing") {
            if ((is_stark_signer *
                is_pending_txn_diff_signer *
                pending_multisig_txn.is_disable_multisig_transaction) == TRUE) {
                assert is_allowed_selector_for_seed_in_multisig([call_array].selector) = TRUE;
            }
        }

        return (valid=TRUE, is_multisig_mode=TRUE);
    }
}
