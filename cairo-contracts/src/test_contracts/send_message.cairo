%lang starknet

from starkware.starknet.common.messages import send_message_to_l1
from starkware.cairo.common.alloc import alloc

@external
func send_message_l2_to_l1{syscall_ptr : felt*}(to_address: felt, payload_len: felt, payload: felt*) {
    send_message_to_l1(to_address, payload_len, payload);
    return ();
}
