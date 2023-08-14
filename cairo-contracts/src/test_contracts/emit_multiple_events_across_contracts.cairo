%lang starknet

from starkware.cairo.common.cairo_builtins import BitwiseBuiltin, HashBuiltin

@contract_interface
namespace IExternalContract {
    func emit_external() {
    }
}

@event
func internal() {
}

@storage_var 
func external_contract_addr() -> (contract: felt) {
}

@constructor
func constructor{
    syscall_ptr: felt*,
    pedersen_ptr: HashBuiltin*,
    range_check_ptr,
}(_external_contract_addr: felt) {
    external_contract_addr.write(_external_contract_addr);
    return ();
}

@external
func emit_internal{ syscall_ptr : felt*, pedersen_ptr : HashBuiltin*, range_check_ptr}() {
    internal.emit();
    return();
} 

@external
func emit_external{ syscall_ptr : felt*, pedersen_ptr : HashBuiltin*, range_check_ptr}() {

    let (_external_contract_addr) = external_contract_addr.read();
    IExternalContract.emit_external(contract_address=_external_contract_addr);
    return();
}

@external
func emit_sandwich{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}() {
    let (_external_contract_addr) = external_contract_addr.read();
    internal.emit();
    IExternalContract.emit_external(contract_address=_external_contract_addr);
    internal.emit();
    IExternalContract.emit_external(contract_address=_external_contract_addr);
    internal.emit();
    return();
}
