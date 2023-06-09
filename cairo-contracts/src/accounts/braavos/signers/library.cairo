%lang starknet

from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.bool import TRUE, FALSE
from starkware.cairo.common.cairo_builtins import HashBuiltin, SignatureBuiltin
from starkware.cairo.common.cairo_secp.bigint import uint256_to_bigint
from starkware.cairo.common.cairo_secp.ec import EcPoint
from starkware.cairo.common.math import assert_not_equal, assert_not_zero, split_felt
from starkware.cairo.common.math_cmp import is_le_felt, is_not_zero
from starkware.cairo.common.signature import verify_ecdsa_signature
from starkware.cairo.common.uint256 import Uint256, uint256_check
from starkware.starknet.common.syscalls import get_block_timestamp, get_tx_info, TxInfo

from src.accounts.braavos.lib.ec import verify_point
from src.accounts.braavos.lib.signature import verify_secp256r1_signature
from src.accounts.braavos.constants import (
    REMOVE_SIGNER_WITH_ETD_SELECTOR,
    SIGNER_TYPE_SECP256R1,
    SIGNER_TYPE_STARK,
    SIGNER_TYPE_UNUSED,
    TX_VERSION_1_EST_FEE,
)

// Structs
struct SignerModel {
    signer_0: felt,
    signer_1: felt,
    signer_2: felt,
    signer_3: felt,
    type: felt,
    reserved_0: felt,
    reserved_1: felt,
}

struct IndexedSignerModel {
    index: felt,
    signer: SignerModel,
}

struct DeferredRemoveSignerRequest {
    expire_at: felt,
    signer_id: felt,
}

// Events
@event
func SignerRemoveRequest(request: DeferredRemoveSignerRequest) {
}

@event
func SignerAdded(signer_id: felt, signer: SignerModel) {
}

@event
func SignerRemoved(signer_id: felt) {
}

@event
func SignerRemoveRequestCancelled(request: DeferredRemoveSignerRequest) {
}

// Storage
@storage_var
func Account_public_key() -> (public_key: felt) {
}

@storage_var
func Account_signers(idx: felt) -> (signer: SignerModel) {
}

@storage_var
func Account_signers_max_index() -> (res: felt) {
}

@storage_var
func Account_signers_num_hw_signers() -> (res: felt) {
}

@storage_var
func Account_deferred_remove_signer() -> (res: DeferredRemoveSignerRequest) {
}

namespace Signers {
    func get_signers{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}() -> (
        signers_len: felt, signers: IndexedSignerModel*
    ) {
        alloc_locals;
        let (max_id) = Account_signers_max_index.read();
        let (signers: IndexedSignerModel*) = alloc();
        let (num_signers) = _get_signers_inner(0, max_id, signers);
        return (signers_len=num_signers, signers=signers);
    }

    func _get_signers_inner{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
        current_id: felt, max_id: felt, signers: IndexedSignerModel*
    ) -> (num_signers: felt) {
        let current_id_overflow = is_le_felt(current_id, max_id);
        if (current_id_overflow == FALSE) {
            return (num_signers=0);
        }

        let (curr_signer) = Account_signers.read(current_id);
        if (curr_signer.type != SIGNER_TYPE_UNUSED) {
            assert [signers] = IndexedSignerModel(
                index=current_id,
                signer=SignerModel(
                    signer_0=curr_signer.signer_0,
                    signer_1=curr_signer.signer_1,
                    signer_2=curr_signer.signer_2,
                    signer_3=curr_signer.signer_3,
                    type=curr_signer.type,
                    reserved_0=curr_signer.reserved_0,
                    reserved_1=curr_signer.reserved_1,
                ),
            );
            let (num_signers) = _get_signers_inner(
                current_id + 1, max_id, signers + IndexedSignerModel.SIZE
            );
            return (num_signers=num_signers + 1);
        } else {
            let (num_signers) = _get_signers_inner(current_id + 1, max_id, signers);
            return (num_signers=num_signers);
        }
    }

    func get_signer{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
        index: felt
    ) -> (signer: SignerModel) {
        let (signer) = Account_signers.read(index);

        return (signer=signer);
    }

    func add_signer{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
        signer: SignerModel
    ) -> (signer_id: felt) {
        // For now we only support adding 1 additional secp256r1 signer and that's it
        with_attr error_message("Signers: can only add 1 secp256r1 signer") {
            assert signer.type = SIGNER_TYPE_SECP256R1;
            let (num_hw_signers) = Account_signers_num_hw_signers.read();
            assert num_hw_signers = 0;
            Account_signers_num_hw_signers.write(num_hw_signers + 1);
        }

        // Make sure we're adding a valid secp256r1 point
        with_attr error_message("Signers: invalid secp256r1 signer") {
            let x_uint256 = Uint256(low=signer.signer_0, high=signer.signer_1);
            uint256_check(x_uint256);
            let y_uint256 = Uint256(low=signer.signer_2, high=signer.signer_3);
            uint256_check(y_uint256);
            let (x_bigint3) = uint256_to_bigint(x_uint256);
            let (y_bigint3) = uint256_to_bigint(y_uint256);
            verify_point(EcPoint(x=x_bigint3, y=y_bigint3));
        }

        let (max_id) = Account_signers_max_index.read();
        let avail_id = max_id + 1;
        Account_signers.write(avail_id, signer);
        Account_signers_max_index.write(avail_id);

        SignerAdded.emit(avail_id, signer);
        return (signer_id=avail_id);
    }

    func swap_signers{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
        remove_index: felt, added_signer: SignerModel, in_multisig_mode: felt
    ) -> (signer_id: felt) {
        alloc_locals;

        let (local tx_info: TxInfo*) = get_tx_info();
        let (current_signer) = Signers.resolve_signer_from_sig(
            tx_info.signature_len, tx_info.signature
        );

        // We only allow hw signer to swap unless we're in multisig then seed can also
        // initiate or approve swap
        with_attr error_message(
                "Signers: can only swap secp256r1 signers using a secp256r1 signer") {
            // DeMorgan on valid_signer OR multisig mode
            assert (1 - in_multisig_mode) * is_not_zero(
                current_signer.signer.type - SIGNER_TYPE_SECP256R1
            ) = FALSE;
        }

        with_attr error_message("Signers: cannot remove signer 0") {
            assert_not_equal(remove_index, 0);
        }
        let (removed_signer) = Account_signers.read(remove_index);
        with_attr error_message("Signers: swap only supported for secp256r1 signer") {
            assert added_signer.type = SIGNER_TYPE_SECP256R1;
        }

        // At this point we verified
        // 1. a secp256r1 signer issued the request
        // 2. we're removing a secp256r1 signer
        // 3. we're adding a secp256r1 signer instead of the same type

        remove_signer(remove_index);

        let (added_signer_id) = add_signer(added_signer);

        return (signer_id=added_signer_id);
    }

    func remove_signer{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
        index: felt
    ) -> () {
        with_attr error_message("Signers: cannot remove signer 0") {
            assert_not_equal(index, 0);
        }

        // NOTE: We know that add_signer limits us to have only 1 additional secp256r1 signer
        let (removed_signer) = Account_signers.read(index);
        Account_signers.write(
            index,
            SignerModel(
                signer_0=SIGNER_TYPE_UNUSED,
                signer_1=SIGNER_TYPE_UNUSED,
                signer_2=SIGNER_TYPE_UNUSED,
                signer_3=SIGNER_TYPE_UNUSED,
                type=SIGNER_TYPE_UNUSED,
                reserved_0=SIGNER_TYPE_UNUSED,
                reserved_1=SIGNER_TYPE_UNUSED,
            ),
        );

        Account_deferred_remove_signer.write(DeferredRemoveSignerRequest(expire_at=0, signer_id=0));

        if (removed_signer.type == SIGNER_TYPE_SECP256R1) {
            let (num_hw_signers) = Account_signers_num_hw_signers.read();
            // enforce only 1 additional signer - when support more need to guarantee
            // that non-hws cannot remove hws
            assert num_hw_signers = 1;
            Account_signers_num_hw_signers.write(num_hw_signers - 1);
            tempvar syscall_ptr = syscall_ptr;
            tempvar pedersen_ptr = pedersen_ptr;
            tempvar range_check_ptr = range_check_ptr;
        } else {
            // FIXME: ASSERT (and maybe remove revokes handling)
            tempvar syscall_ptr = syscall_ptr;
            tempvar pedersen_ptr = pedersen_ptr;
            tempvar range_check_ptr = range_check_ptr;
        }

        SignerRemoved.emit(index);
        return ();
    }

    func remove_signer_with_etd{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
        index: felt, account_etd: felt
    ) -> () {
        with_attr error_message("Signers: cannot remove signer 0") {
            assert_not_equal(index, 0);
        }

        // Make sure we remove a hw signer, this also implies that there is one
        let (removed_signer) = Account_signers.read(index);
        with_attr error_message("Signers: tried removing invalid signer") {
            assert removed_signer.type = SIGNER_TYPE_SECP256R1;
        }

        // For now we limit this API to seed signer only as it has no functional meaning with secp256r1
        let (tx_info) = get_tx_info();
        let (current_signer) = resolve_signer_from_sig(tx_info.signature_len, tx_info.signature);
        with_attr error_message(
                "Signers: remove_signer_with_etd should be called with seed signer") {
            assert current_signer.signer.type = SIGNER_TYPE_STARK;
        }

        // We dont want to allow endless postponement of etd removals, once
        // there's an etd it should either finish or cancelled
        let (remove_signer_req) = Account_deferred_remove_signer.read();
        with_attr error_message("Signers: already have a pending remove signer request") {
            assert remove_signer_req.expire_at = 0;
        }

        let (block_timestamp) = get_block_timestamp();
        with_attr error_message("Signers: etd not initialized") {
            assert_not_zero(account_etd);
        }
        let expire_at = block_timestamp + account_etd;
        let remove_req = DeferredRemoveSignerRequest(expire_at=expire_at, signer_id=index);
        Account_deferred_remove_signer.write(remove_req);
        SignerRemoveRequest.emit(remove_req);
        return ();
    }

    func get_deferred_remove_signer_req{
        syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr
    }() -> (deferred_request: DeferredRemoveSignerRequest) {
        let (deferred_request) = Account_deferred_remove_signer.read();

        return (deferred_request=deferred_request);
    }

    func cancel_deferred_remove_signer_req{
        syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr
    }(removed_signer_id: felt) -> () {
        // remove_signer_id is for future compatibility where we can possibly have multiple hw signers
        let (deferred_request) = Account_deferred_remove_signer.read();

        with_attr error_message("Signers: invalid remove signer request to cancel") {
            assert_not_zero(deferred_request.expire_at);
            assert deferred_request.signer_id = removed_signer_id;
        }

        Account_deferred_remove_signer.write(DeferredRemoveSignerRequest(expire_at=0, signer_id=0));
        SignerRemoveRequestCancelled.emit(deferred_request);

        return ();
    }

    func resolve_signer_from_sig{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
        signature_len: felt, signature: felt*
    ) -> (signer: IndexedSignerModel) {
        if (signature_len == 2) {
            let (signer) = Account_signers.read(0);
            let indexed_signer = IndexedSignerModel(index=0, signer=signer);
            return (signer=indexed_signer);
        } else {
            let (signer) = Account_signers.read(signature[0]);
            let indexed_signer = IndexedSignerModel(index=signature[0], signer=signer);
            return (signer=indexed_signer);
        }
    }

    func apply_elapsed_etd_requests{
        syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr
    }(block_timestamp: felt) -> () {
        let (remove_signer_req) = Account_deferred_remove_signer.read();
        let have_remove_signer_etd = is_not_zero(remove_signer_req.expire_at);
        let remove_signer_etd_expired = is_le_felt(remove_signer_req.expire_at, block_timestamp);

        if (have_remove_signer_etd * remove_signer_etd_expired == TRUE) {
            remove_signer(remove_signer_req.signer_id);
            return ();
        }

        return ();
    }

    func signers_validate{
        syscall_ptr: felt*,
        pedersen_ptr: HashBuiltin*,
        range_check_ptr,
        ecdsa_ptr: SignatureBuiltin*,
    }(
        call_array_len: felt,
        call_0_to: felt,
        call_0_sel: felt,
        calldata_len: felt,
        calldata: felt*,
        tx_info: TxInfo*,
        block_timestamp: felt,
        block_num: felt,
        in_multisig_mode,
    ) -> (valid: felt) {
        // Authorize Signer
        _authorize_signer(
            tx_info.account_contract_address,
            tx_info.signature_len,
            tx_info.signature,
            call_array_len,
            call_0_to,
            call_0_sel,
            block_timestamp,
            in_multisig_mode,
        );

        // For estimate fee txns we skip sig validation - client side should account for it
        if (is_le_felt(TX_VERSION_1_EST_FEE, tx_info.version) == TRUE) {
            return (valid=TRUE);
        }

        // Validate signature
        with_attr error_message("Signers: invalid signature") {
            let (is_valid) = is_valid_signature(
                tx_info.transaction_hash, tx_info.signature_len, tx_info.signature
            );
            assert is_valid = TRUE;
        }

        return (valid=TRUE);
    }

    func _authorize_signer{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
        self: felt,
        signature_len: felt,
        signature: felt*,
        call_array_len: felt,
        call_0_to: felt,
        call_0_sel: felt,
        block_timestamp: felt,
        in_multisig_mode: felt,
    ) -> () {
        alloc_locals;

        let (num_additional_signers) = Account_signers_num_hw_signers.read();
        let (tx_info) = get_tx_info();
        let (signer) = Signers.resolve_signer_from_sig(signature_len, signature);

        // Dont limit txns on: not(secp256r1) OR multisig
        // the if below is boolean equivalent via DeMorgan identity
        if (num_additional_signers * (1 - in_multisig_mode) == FALSE) {
            return ();
        }

        if (signer.signer.type == SIGNER_TYPE_SECP256R1) {
            // We either don't have a pending removal, or it wasn't expired yet
            // so we're good to go
            return ();
        }

        // else: At this point we have secp256r1 signer (num_additional_signers > 0)
        // we're not in multisig and txn was sent with seed signer

        // 0. be defensive about the fact that we only allow seed signing
        // revisit when additional signer types are supported
        with_attr error_message("Signers: either secp256r1 or seed signers are expected") {
            assert signer.signer.type = SIGNER_TYPE_STARK;
        }

        // 1. Limit seed signer only to ETD signer removal
        with_attr error_message("Signers: invalid entry point for seed signing") {
            assert call_0_to = self;
            assert call_0_sel = REMOVE_SIGNER_WITH_ETD_SELECTOR;
        }
        with_attr error_message("Signers: only a single call is allowed with seed signing") {
            assert call_array_len = 1;
        }

        return ();
    }

    func _is_valid_stark_signature{
        syscall_ptr: felt*,
        pedersen_ptr: HashBuiltin*,
        range_check_ptr,
        ecdsa_ptr: SignatureBuiltin*,
    }(public_key: felt, hash: felt, signature_len: felt, signature: felt*) -> (is_valid: felt) {
        // This interface expects a signature pointer and length to make
        // no assumption about signature validation schemes.
        // But this implementation does, and it expects a (sig_r, sig_s) pair.
        let sig_r = signature[0];
        let sig_s = signature[1];

        verify_ecdsa_signature(
            message=hash, public_key=public_key, signature_r=sig_r, signature_s=sig_s
        );

        return (is_valid=TRUE);
    }

    func _is_valid_secp256r1_signature{
        syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr
    }(signer: SignerModel, hash: felt, signature_len: felt, signature: felt*) -> (is_valid: felt) {
        // x,y were verified in add_signer
        let (x) = uint256_to_bigint(Uint256(low=signer.signer_0, high=signer.signer_1));
        let (y) = uint256_to_bigint(Uint256(low=signer.signer_2, high=signer.signer_3));
        // validate r,s
        let r_uint256 = Uint256(low=signature[0], high=signature[1]);
        uint256_check(r_uint256);
        let s_uint256 = Uint256(low=signature[2], high=signature[3]);
        uint256_check(s_uint256);
        let (r_bigint3) = uint256_to_bigint(r_uint256);
        let (s_bigint3) = uint256_to_bigint(s_uint256);
        let (hash_high, hash_low) = split_felt(hash);
        let (hash_bigint3) = uint256_to_bigint(Uint256(low=hash_low, high=hash_high));
        verify_secp256r1_signature(hash_bigint3, r_bigint3, s_bigint3, EcPoint(x=x, y=y));
        return (is_valid=TRUE);
    }

    func is_valid_signature{
        syscall_ptr: felt*,
        pedersen_ptr: HashBuiltin*,
        range_check_ptr,
        ecdsa_ptr: SignatureBuiltin*,
    }(hash: felt, signature_len: felt, signature: felt*) -> (is_valid: felt) {
        if (signature_len == 2) {
            // Keep compatibility for STARK signers from default SDKs/CLIs
            let (signer_0) = Account_signers.read(0);
            _is_valid_stark_signature(signer_0.signer_0, hash, signature_len, signature);
            return (is_valid=TRUE);
        }

        let (signer) = Account_signers.read(signature[0]);

        if (signer.type == SIGNER_TYPE_STARK) {
            with_attr error_message("Signers: Invalid signature length") {
                // 1 signer idx + 2 felts (r,s)
                assert signature_len = 3;
            }

            _is_valid_stark_signature(signer.signer_0, hash, signature_len - 1, signature + 1);
            return (is_valid=TRUE);
        }

        if (signer.type == SIGNER_TYPE_SECP256R1) {
            with_attr error_message("Signers: Invalid signature length") {
                // 1 signer idx + 2 x uint256 (r,s)
                assert signature_len = 5;
            }

            _is_valid_secp256r1_signature(signer, hash, signature_len - 1, signature + 1);
            return (is_valid=TRUE);
        }

        // Unsupported signer type!
        with_attr error_message("Signers: unsupported signer type") {
            assert_not_zero(0);
        }

        return (is_valid=FALSE);
    }
}
