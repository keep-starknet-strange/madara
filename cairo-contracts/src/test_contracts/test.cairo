%lang starknet

from starkware.cairo.common.bool import FALSE
from starkware.cairo.common.cairo_builtins import BitwiseBuiltin, HashBuiltin
from starkware.starknet.common.syscalls import (
    storage_read,
    storage_write,
    library_call,
    deploy,
    call_contract,
)
from starkware.starknet.core.os.contract_address.contract_address import get_contract_address

@event
func Event(value: felt) {
}

@storage_var
func number_map(key: felt) -> (value: felt) {
}

@constructor
func constructor{syscall_ptr: felt*}(address: felt, value: felt) {
    storage_write(address=address, value=value);
    return ();
}

@external
func without_arg() {
    return ();
}

@external
func emit_event{syscall_ptr: felt*, range_check_ptr}() {
    Event.emit(1);
    return ();
}

@external
func with_arg(num: felt) {
    assert num = 25;
    return ();
}

@external
func return_result(num: felt) -> (result: felt) {
    return (result=num);
}

@external
func bitwise_and{bitwise_ptr: BitwiseBuiltin*}(x: felt, y: felt) {
    bitwise_ptr.x = x;
    bitwise_ptr.y = y;
    let x_and_y = bitwise_ptr.x_and_y;
    let x_xor_y = bitwise_ptr.x_xor_y;
    let x_or_y = bitwise_ptr.x_or_y;
    let bitwise_ptr = bitwise_ptr + BitwiseBuiltin.SIZE;
    assert x_and_y = 15;
    return ();
}

@external
func sqrt{range_check_ptr}(value: felt) {
    alloc_locals;
    local root: felt;

    %{
        from starkware.python.math_utils import isqrt
        value = ids.value % PRIME
        assert value < 2 ** 250, f"value={value} is outside of the range [0, 2**250)."
        assert 2 ** 250 < PRIME
        ids.root = isqrt(value)
    %}

    assert root = 9;
    return ();
}

@external
func test_storage_read_write{syscall_ptr: felt*}(address: felt, value: felt) -> (result: felt) {
    storage_write(address=address, value=value);
    let (read_value) = storage_read(address=address);
    return (result=read_value);
}

@external
@raw_output
func test_library_call{syscall_ptr: felt*}(
    class_hash: felt, selector: felt, calldata_len: felt, calldata: felt*
) -> (retdata_size: felt, retdata: felt*) {
    let (retdata_size: felt, retdata: felt*) = library_call(
        class_hash=class_hash,
        function_selector=selector,
        calldata_size=calldata_len,
        calldata=calldata,
    );
    return (retdata_size=retdata_size, retdata=retdata);
}

@external
func test_nested_library_call{syscall_ptr: felt*}(
    class_hash: felt, lib_selector: felt, nested_selector: felt, calldata_len: felt, calldata: felt*
) -> (result: felt) {
    alloc_locals;
    assert calldata_len = 2;
    local nested_library_calldata: felt* = new (class_hash, nested_selector, 2,
        calldata[0] + 1, calldata[1] + 1);
    let (retdata_size: felt, retdata: felt*) = library_call(
        class_hash=class_hash,
        function_selector=lib_selector,
        calldata_size=5,
        calldata=nested_library_calldata,
    );

    let (retdata_size: felt, retdata: felt*) = library_call(
        class_hash=class_hash,
        function_selector=nested_selector,
        calldata_size=calldata_len,
        calldata=calldata,
    );

    return (result=0);
}

@external
@raw_output
func test_call_contract{syscall_ptr: felt*}(
    contract_address: felt, function_selector: felt, calldata_len: felt, calldata: felt*
) -> (retdata_size: felt, retdata: felt*) {
    let (retdata_size: felt, retdata: felt*) = call_contract(
        contract_address=contract_address,
        function_selector=function_selector,
        calldata_size=calldata_len,
        calldata=calldata,
    );
    return (retdata_size=retdata_size, retdata=retdata);
}

@external
func test_deploy{syscall_ptr: felt*}(
    class_hash: felt,
    contract_address_salt: felt,
    constructor_calldata_len: felt,
    constructor_calldata: felt*,
    deploy_from_zero: felt,
) -> (contract_address: felt) {
    let (contract_address) = deploy(
        class_hash=class_hash,
        contract_address_salt=contract_address_salt,
        constructor_calldata_size=constructor_calldata_len,
        constructor_calldata=constructor_calldata,
        deploy_from_zero=deploy_from_zero,
    );
    return (contract_address=contract_address);
}

@external
func test_storage_var{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}() {
    number_map.write(key=1, value=39);
    let (val) = number_map.read(key=1);
    assert val = 39;
    return ();
}

@external
func test_contract_address{pedersen_ptr: HashBuiltin*, range_check_ptr}(
    salt: felt,
    class_hash: felt,
    constructor_calldata_len: felt,
    constructor_calldata: felt*,
    deployer_address: felt,
) -> (contract_address: felt) {
    let (contract_address) = get_contract_address{hash_ptr=pedersen_ptr}(
        salt=salt,
        class_hash=class_hash,
        constructor_calldata_size=constructor_calldata_len,
        constructor_calldata=constructor_calldata,
        deployer_address=deployer_address,
    );

    return (contract_address=contract_address);
}
