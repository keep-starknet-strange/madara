%lang starknet

from starkware.starknet.common.messages import send_message_to_l1
from starkware.cairo.common.alloc import alloc

@external
func send_message_l2_to_l1{syscall_ptr : felt*}() {
    let (payload: felt*) = alloc();
    send_message_to_l1(1, 0, payload);
    return ();
}

@l1_handler
func send_message_l1_to_l2(from_address: felt) {
    assert from_address = 1;
    return ();
}
