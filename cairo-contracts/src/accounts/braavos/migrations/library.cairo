%lang starknet
from starkware.cairo.common.bool import TRUE, FALSE
from starkware.cairo.common.cairo_builtins import HashBuiltin
from starkware.cairo.common.math_cmp import is_le_felt

from src.accounts.braavos.signers.library import (
    Account_signers,
    Account_signers_max_index,
    SignerModel,
)
from src.accounts.braavos.constants import SIGNER_TYPE_SECP256R1

const LEGACY_SIGNER_TYPE_SECP256R1_SWS = 0x3;

namespace Migrations {
    // testnet only contract - deprecate signer type 3 and migrate
    // existing type 3 signers to type 2
    func migrate_000_000_009{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}() -> (
        res: felt
    ) {
        let (max_id) = Account_signers_max_index.read();
        _migrate_type_3_signers(0, max_id);
        return (TRUE,);
    }

    func _migrate_type_3_signers{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
        current_id: felt, max_id: felt
    ) -> () {
        alloc_locals;

        if (is_le_felt(current_id, max_id) == FALSE) {
            return ();
        }

        let (curr_signer) = Account_signers.read(current_id);
        if (curr_signer.type == LEGACY_SIGNER_TYPE_SECP256R1_SWS) {
            Account_signers.write(
                current_id,
                SignerModel(
                    signer_0=curr_signer.signer_0,
                    signer_1=curr_signer.signer_1,
                    signer_2=curr_signer.signer_2,
                    signer_3=curr_signer.signer_3,
                    type=SIGNER_TYPE_SECP256R1,
                    reserved_0=curr_signer.reserved_0,
                    reserved_1=curr_signer.reserved_1,
                ),
            );
            _migrate_type_3_signers(current_id + 1, max_id);
            return ();
        } else {
            _migrate_type_3_signers(current_id + 1, max_id);
            return ();
        }
    }
}
