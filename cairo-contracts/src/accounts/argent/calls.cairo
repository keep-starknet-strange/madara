%lang starknet

from starkware.cairo.common.cairo_builtins import HashBuiltin, SignatureBuiltin
from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.memcpy import memcpy
from starkware.cairo.common.math import assert_not_zero, assert_le, assert_nn
from starkware.starknet.common.syscalls import call_contract
from starkware.cairo.common.bool import TRUE, FALSE

struct Call {
    to: felt,
    selector: felt,
    calldata_len: felt,
    calldata: felt*,
}

// Tmp struct introduced while we wait for Cairo
// to support passing `[Call]` to __execute__
struct CallArray {
    to: felt,
    selector: felt,
    data_offset: felt,
    data_len: felt,
}

// @notice Executes a list of call array recursively
// @return response_len: The size of the returned data
// @return response: An array of felt populated with the returned data 
//   in the form [len(call_1_data), *call_1_data, len(call_2_data), *call_2_data, ..., len(call_N_data), *call_N_data]
func execute_multicall{syscall_ptr: felt*}(
    call_array_len: felt, call_array: CallArray*, calldata: felt*
) -> (response_len: felt, response: felt*) {
    alloc_locals;

    if (call_array_len == 0) {
        let (response) = alloc();
        return (0, response);
    }

    // call recursively all previous calls
    let (response_len, response: felt*) = execute_multicall(call_array_len - 1, call_array, calldata);

    // handle the last call
    let last_call = call_array[call_array_len - 1];

    // call the last call
    with_attr error_message("multicall {call_array_len} failed") {
        let res = call_contract(
            contract_address=last_call.to,
            function_selector=last_call.selector,
            calldata_size=last_call.data_len,
            calldata=calldata + last_call.data_offset,
        );
    }

    // store response data
    assert [response + response_len] = res.retdata_size;
    memcpy(response + response_len + 1, res.retdata, res.retdata_size);
    return (response_len + res.retdata_size + 1, response);
}
