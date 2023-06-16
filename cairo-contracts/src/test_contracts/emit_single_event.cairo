%lang starknet

from starkware.cairo.common.cairo_builtins import BitwiseBuiltin, HashBuiltin

@event
func do_event1() {
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
    do_event1.emit();
    return ();
}
