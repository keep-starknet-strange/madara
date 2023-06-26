%lang starknet

from starkware.cairo.common.cairo_builtins import BitwiseBuiltin, HashBuiltin

@event
func external() {
}
@constructor
func constructor{
    syscall_ptr: felt*,
    pedersen_ptr: HashBuiltin*,
    range_check_ptr,
}() {
    return ();
}

@external
func emit_external{ syscall_ptr : felt*, pedersen_ptr : HashBuiltin*, range_check_ptr}() {
    external.emit();
    return ();
}
