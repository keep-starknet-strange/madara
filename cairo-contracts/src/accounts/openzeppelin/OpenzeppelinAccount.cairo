// A dummy account contract without any validations.

%lang starknet

from starkware.cairo.common.bool import TRUE
from starkware.cairo.common.cairo_builtins import HashBuiltin, SignatureBuiltin, BitwiseBuiltin
from starkware.starknet.common.syscalls import (
    call_contract,
    deploy,
    get_caller_address,
    get_contract_address,
    get_tx_info
)
from src.accounts.openzeppelin.library import Account

@event
func ContractDeployed(
    address: felt, deployer: felt, classHash: felt, calldata_len: felt, calldata: felt*, salt: felt
) {
}

@external
func __validate_declare__{
    syscall_ptr: felt*,
    pedersen_ptr: HashBuiltin*,
    ecdsa_ptr: SignatureBuiltin*,
    range_check_ptr
}(class_hash: felt) {
    let (tx_info) = get_tx_info();
    Account.is_valid_signature(tx_info.transaction_hash, tx_info.signature_len, tx_info.signature);
    return ();
}

@external
func __validate_deploy__{
    syscall_ptr: felt*,
    pedersen_ptr: HashBuiltin*,
    ecdsa_ptr: SignatureBuiltin*,
    range_check_ptr
}(class_hash: felt, contract_address_salt: felt) {
    let (tx_info) = get_tx_info();
    Account.is_valid_signature(tx_info.transaction_hash, tx_info.signature_len, tx_info.signature);
    return ();
}

@external
func __validate__{
    syscall_ptr: felt*,
    pedersen_ptr: HashBuiltin*,
    ecdsa_ptr: SignatureBuiltin*,
    range_check_ptr
}(contract_address, selector: felt, calldata_len: felt, calldata: felt*) {
    let (tx_info) = get_tx_info();
    Account.is_valid_signature(tx_info.transaction_hash, tx_info.signature_len, tx_info.signature);
    return ();
}

@external
@raw_output
func __execute__{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    contract_address, selector: felt, calldata_len: felt, calldata: felt*
) -> (retdata_size: felt, retdata: felt*) {
    let (retdata_size: felt, retdata: felt*) = call_contract(
        contract_address=contract_address,
        function_selector=selector,
        calldata_size=calldata_len,
        calldata=calldata,
    );
    return (retdata_size=calldata_len, retdata=calldata);
}

@external
func deploy_contract{syscall_ptr: felt*, range_check_ptr}(
    class_hash: felt,
    contract_address_salt: felt,
    constructor_calldata_len: felt,
    constructor_calldata: felt*,
) -> (contract_address: felt) {
    let (contract_address) = deploy(
        class_hash=class_hash,
        contract_address_salt=contract_address_salt,
        constructor_calldata_size=constructor_calldata_len,
        constructor_calldata=constructor_calldata,
        deploy_from_zero=TRUE,
    );
    ContractDeployed.emit(
        address=contract_address,
        deployer=0,
        classHash=class_hash,
        calldata_len=constructor_calldata_len,
        calldata=constructor_calldata,
        salt=contract_address_salt,
    );
    return (contract_address=0);
}
