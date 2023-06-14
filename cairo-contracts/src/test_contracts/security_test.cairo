%lang starknet

from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.cairo_builtins import EcOpBuiltin, HashBuiltin, SignatureBuiltin
from starkware.cairo.common.dict_access import DictAccess
from starkware.cairo.common.ec_point import EcPoint
from starkware.cairo.common.registers import get_fp_and_pc
from starkware.cairo.common.signature import verify_ecdsa_signature
from starkware.starknet.common.syscalls import (
    CALL_CONTRACT_SELECTOR,
    DEPLOY_SELECTOR,
    Deploy,
    DeployRequest,
    TxInfo,
    call_contract,
    get_contract_address,
    get_tx_info,
    get_tx_signature,
    storage_read,
)

// This function is called to verify that certain storage security errors happen at the expected
// timing.
@external
func foo() {
    assert 0 = 1;
    return ();
}

@external
func empty_function() {
    return ();
}

@contract_interface
namespace SecurityTestContract {
    func foo() {
    }

    func empty_function() {
    }
}

// VM execution failures.

@external
func test_nonrelocatable_syscall_ptr{syscall_ptr}() {
    let syscall_ptr = 0;
    return ();
}

@external
func test_unknown_memory{syscall_ptr: felt*}() {
    assert [ap] = [syscall_ptr];
    return ();
}

@external
func test_subtraction_between_relocatables{syscall_ptr: felt*, range_check_ptr}() {
    tempvar a = syscall_ptr - range_check_ptr;
    return ();
}

@external
func test_relocatables_addition_failure{syscall_ptr}() {
    tempvar a = syscall_ptr + syscall_ptr;
    return ();
}

@external
func test_op0_unknown_double_dereference{syscall_ptr: felt*}() {
    [[ap]] = [ap];
    return ();
}

@external
func test_write_to_program_segment() {
    // Tests a write to the end of the program segment.
    let (_, __pc__) = get_fp_and_pc();
    assert [__pc__ + 1000] = 37;
    return ();
}

@external
func test_exit_main_scope() {
    %{ vm_exit_scope() %}
    %{ vm_enter_scope() %}
    return ();
}

@external
func test_missing_exit_scope() {
    %{ vm_enter_scope() %}
    return ();
}

@external
func test_out_of_bound_memory_value() {
    let (ptr) = alloc();
    tempvar invalid_ptr = ptr - 1;
    return ();
}

@external
func test_non_relocatable_memory_address() {
    let ptr: felt* = cast(10, felt*);
    assert [ptr] = 1;
    return ();
}

@external
func test_bad_expr_eval() {
    let test = [cast(fp, TxInfo*)];
    with_attr error_message("Bad expr: {test}.") {
        assert 1 = 0;
    }
    return ();
}

// Builtin execution failures.

@external
func test_bad_pedersen_values{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*}() {
    // Tests invalid value in the Pedersen builtin.
    // Set result before x and y, so that the auto-deduction mechanism will not be invoked.
    assert pedersen_ptr.result = 0;
    assert pedersen_ptr.x = 0;
    assert pedersen_ptr.y = 0;
    let pedersen_ptr = pedersen_ptr + HashBuiltin.SIZE;
    return ();
}

@external
func test_bad_range_check_values{syscall_ptr: felt*, range_check_ptr: felt*}() {
    assert [range_check_ptr] = 2 ** 128 + 1;
    let range_check_ptr = range_check_ptr + 1;
    return ();
}

@external
func test_missing_signature_hint{syscall_ptr: felt*, ecdsa_ptr: SignatureBuiltin*}() {
    assert [ecdsa_ptr] = SignatureBuiltin(1, 2);
    return ();
}

@external
func test_signature_hint_on_wrong_segment{syscall_ptr: felt*, ecdsa_ptr: SignatureBuiltin*}() {
    let (ptr: SignatureBuiltin*) = alloc();
    verify_ecdsa_signature{ecdsa_ptr=ptr}(0, 0, 0, 0);
    return ();
}

@external
func test_ec_op_invalid_input{
    syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr, ec_op_ptr: EcOpBuiltin*
}() {
    // Choose p = 4 * q.
    // Trying to compute p + 8 * q starts with the following pairs of points:
    //   (p, q),
    //   (p, 2 * q),
    //   (p, 4 * q),
    //   (p, 8 * q),
    // But since p = 4 * q, the pair (p, 4 * q) is invalid (the x-coordinate is the same).
    assert ec_op_ptr[0].p = EcPoint(
        0x6a4beaef5a93425b973179cdba0c9d42f30e01a5f1e2db73da0884b8d6756fc,
        0x72565ec81bc09ff53fbfad99324a92aa5b39fb58267e395e8abe36290ebf24f,
    );
    assert ec_op_ptr[0].q = EcPoint(
        0x654fd7e67a123dd13868093b3b7777f1ffef596c2e324f25ceaf9146698482c,
        0x4fad269cbf860980e38768fe9cb6b0b9ab03ee3fe84cfde2eccce597c874fd8,
    );
    assert ec_op_ptr[0].m = 8;
    let ec_op_ptr = &ec_op_ptr[1];
    return ();
}

@external
func test_ec_op_point_not_on_curve{
    syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr, ec_op_ptr: EcOpBuiltin*
}() {
    tempvar p = EcPoint(
        0x654fd7e67a123dd13868093b3b7777f1ffef596c2e324f25ceaf9146698482c,
        0x4fad269cbf860980e38768fe9cb6b0b9ab03ee3fe84cfde2eccce597c874fd8,
    );
    assert ec_op_ptr[0].p = p;
    assert ec_op_ptr[0].q = EcPoint(x=p.x, y=p.y + 1);
    assert ec_op_ptr[0].m = 7;
    let ec_op_ptr = &ec_op_ptr[1];
    return ();
}

@external
func maybe_call_foo{syscall_ptr: felt*, range_check_ptr}(call_foo: felt) {
    if (call_foo != 0) {
        SecurityTestContract.foo(contract_address=100);
        return ();
    }
    return ();
}

// Syscall execution failures.

@external
func test_read_bad_address{syscall_ptr: felt*, range_check_ptr}(call_foo: felt) {
    storage_read(address=2 ** 251);

    maybe_call_foo(call_foo=call_foo);
    return ();
}

@external
func test_relocatable_storage_address{syscall_ptr: felt*, range_check_ptr}(call_foo: felt) {
    storage_read(address=cast(syscall_ptr, felt));

    maybe_call_foo(call_foo=call_foo);
    return ();
}

@external
func test_bad_call_address{syscall_ptr: felt*}() {
    let (calldata) = alloc();

    call_contract(
        contract_address=0x17, function_selector=0x19, calldata_size=0, calldata=calldata
    );
    return ();
}

@external
func test_bad_syscall_request_arg_type{syscall_ptr: felt*}() {
    assert syscall_ptr[0] = CALL_CONTRACT_SELECTOR;
    // Contract address.
    assert syscall_ptr[1] = 0;
    // Function selector.
    assert syscall_ptr[2] = 0;
    // Calldata size.
    assert syscall_ptr[3] = 1;
    // Calldata - should be a pointer, but we are passing a felt.
    assert syscall_ptr[4] = 0;
    %{ syscall_handler.call_contract(segments=segments, syscall_ptr=ids.syscall_ptr) %}
    return ();
}

@external
func test_bad_call_selector{syscall_ptr: felt*}() {
    let (contract_address) = get_contract_address();
    let (calldata) = alloc();

    call_contract(
        contract_address=contract_address,
        function_selector=0x19,
        calldata_size=0,
        calldata=calldata,
    );
    return ();
}

@external
func test_bad_deploy_from_zero_field{syscall_ptr: felt*}() {
    let syscall = [cast(syscall_ptr, Deploy*)];
    assert syscall.request = DeployRequest(
        selector=DEPLOY_SELECTOR,
        class_hash=1,
        contract_address_salt=1,
        constructor_calldata_size=0,
        constructor_calldata=new (),
        deploy_from_zero=2,
    );

    %{ syscall_handler.deploy(segments=segments, syscall_ptr=ids.syscall_ptr) %}
    return ();
}

// Post-run validation failures.

// Create a hole in the range check segment. Calling this function will fail.
@external
func test_builtin_hole{range_check_ptr}() {
    assert [range_check_ptr + 1] = 17;
    let range_check_ptr = range_check_ptr + 2;
    return ();
}

@external
func test_missing_pedersen_values{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*}() {
    // Tests missing values in the Pedersen builtin.
    assert pedersen_ptr.result = 0;
    let pedersen_ptr = pedersen_ptr + HashBuiltin.SIZE;
    return ();
}

@external
func test_bad_builtin_stop_ptr{range_check_ptr}() {
    let range_check_ptr = range_check_ptr + 2;
    return ();
}

@external
func test_access_after_syscall_stop_ptr{syscall_ptr: felt*}() {
    assert [syscall_ptr] = 17;
    return ();
}

@external
func test_bad_syscall_stop_ptr{syscall_ptr}() {
    assert [syscall_ptr] = 0;
    let syscall_ptr = syscall_ptr + 1;
    return ();
}

@external
func test_out_of_bounds_write_to_signature_segment{syscall_ptr: felt*}() {
    let (signature_len: felt, signature: felt*) = get_tx_signature();
    assert signature[signature_len] = 17;
    return ();
}

@external
func test_out_of_bounds_write_to_tx_info_segment{syscall_ptr: felt*}() {
    let (tx_info_segment: felt*) = get_tx_info();
    assert tx_info_segment[TxInfo.SIZE] = 17;
    return ();
}

@external
func test_write_to_call_contract_return_value{syscall_ptr: felt*}() {
    let (calldata) = alloc();
    let (contract_address) = get_contract_address();

    let (retdata_size, retdata) = call_contract(
        contract_address=contract_address,
        function_selector=SecurityTestContract.EMPTY_FUNCTION_SELECTOR,
        calldata_size=0,
        calldata=calldata,
    );

    assert retdata[0] = 0;

    return ();
}

@external
func test_out_of_bounds_write_to_calldata_segment{syscall_ptr: felt*}(
    array_len: felt, array: felt*
) {
    assert array[array_len] = 0;
    return ();
}
