// SPDX-License-Identifier: MIT
// Based on OpenZeppelin Contracts for Cairo ~v0.2.0 (upgrades/Proxy.cairo)

%lang starknet

from starkware.cairo.common.cairo_builtins import HashBuiltin
from starkware.starknet.common.syscalls import library_call_l1_handler, library_call
from resources.account.braavos.openzepellin_deps.library import Proxy

//
// Constructor
//

@constructor
func constructor{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    implementation_address: felt, initializer_selector: felt, calldata_len: felt, calldata: felt*
) {
    Proxy._set_implementation(implementation_address);

    library_call(
        class_hash=implementation_address,
        function_selector=initializer_selector,
        calldata_size=calldata_len,
        calldata=calldata,
    );
    return ();
}

//
// Getters
//

@view
func get_implementation{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}() -> (
    implementation: felt
) {
    let (impl) = Proxy.get_implementation();
    return (implementation=impl);
}

//
// Fallback functions
//

@external
@raw_input
@raw_output
func __default__{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    selector: felt, calldata_size: felt, calldata: felt*
) -> (retdata_size: felt, retdata: felt*) {
    let (address) = Proxy.get_implementation();

    let (retdata_size: felt, retdata: felt*) = library_call(
        class_hash=address,
        function_selector=selector,
        calldata_size=calldata_size,
        calldata=calldata,
    );

    return (retdata_size=retdata_size, retdata=retdata);
}

@l1_handler
@raw_input
func __l1_default__{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    selector: felt, calldata_size: felt, calldata: felt*
) {
    let (address) = Proxy.get_implementation();

    library_call_l1_handler(
        class_hash=address,
        function_selector=selector,
        calldata_size=calldata_size,
        calldata=calldata,
    );

    return ();
}