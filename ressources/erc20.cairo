%lang starknet

from starkware.cairo.common.cairo_builtins import HashBuiltin, SignatureBuiltin
from starkware.cairo.common.math import assert_nn_le, assert_not_zero
from starkware.cairo.common.uint256 import (
    Uint256,
    uint256_add,
    uint256_check,
    uint256_le,
    uint256_sub,
)

// In Solidity ERC20 decimals is a uint8.
const MAX_DECIMALS = 255;

// Events.

@event
func Transfer(from_: felt, to: felt, value: Uint256) {
}

@event
func Approval(owner: felt, spender: felt, value: Uint256) {
}

// Storage.

@storage_var
func ERC20_name() -> (name: felt) {
}

@storage_var
func ERC20_symbol() -> (symbol: felt) {
}

@storage_var
func ERC20_decimals() -> (decimals: felt) {
}

@storage_var
func ERC20_total_supply() -> (total_supply: Uint256) {
}

@storage_var
func ERC20_balances(account: felt) -> (balance: Uint256) {
}

@storage_var
func ERC20_allowances(owner: felt, spender: felt) -> (allowance: Uint256) {
}

// Constructor.

func ERC20_initializer{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    name: felt, symbol: felt, decimals: felt
) {
    assert_nn_le(decimals, MAX_DECIMALS);
    ERC20_name.write(name);
    ERC20_symbol.write(symbol);
    ERC20_decimals.write(decimals);
    return ();
}

// Getters.

@view
func name{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}() -> (name: felt) {
    let (name) = ERC20_name.read();
    return (name=name);
}

@view
func symbol{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}() -> (symbol: felt) {
    let (symbol) = ERC20_symbol.read();
    return (symbol=symbol);
}

@view
func totalSupply{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}() -> (
    totalSupply: Uint256
) {
    let (totalSupply: Uint256) = ERC20_total_supply.read();
    return (totalSupply=totalSupply);
}

@view
func decimals{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}() -> (
    decimals: felt
) {
    let (decimals) = ERC20_decimals.read();
    return (decimals=decimals);
}

@view
func balanceOf{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(account: felt) -> (
    balance: Uint256
) {
    let (balance: Uint256) = ERC20_balances.read(account=account);
    return (balance=balance);
}

@view
func allowance{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    owner: felt, spender: felt
) -> (remaining: Uint256) {
    let (remaining: Uint256) = ERC20_allowances.read(owner=owner, spender=spender);
    return (remaining=remaining);
}

// Internals.

func ERC20_mint{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    recipient: felt, amount: Uint256
) {
    alloc_locals;
    assert_not_zero(recipient);
    uint256_check(amount);

    let (balance: Uint256) = ERC20_balances.read(account=recipient);
    // If uint256_add(balance, amount) overflows then uint256_add(supply, amount) is going to
    // overflow as well and the transaction will be reverted.
    let (new_balance: Uint256, _: felt) = uint256_add(balance, amount);
    ERC20_balances.write(recipient, new_balance);

    let (local supply: Uint256) = ERC20_total_supply.read();
    let (local new_supply: Uint256, is_overflow) = uint256_add(supply, amount);
    assert (is_overflow) = 0;

    ERC20_total_supply.write(new_supply);
    Transfer.emit(0, recipient, amount);
    return ();
}

func ERC20_transfer{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    sender: felt, recipient: felt, amount: Uint256
) {
    alloc_locals;
    assert_not_zero(sender);
    assert_not_zero(recipient);
    uint256_check(amount);  // Almost surely not needed, might remove after confirmation.

    let (local sender_balance: Uint256) = ERC20_balances.read(account=sender);

    // Validates amount <= sender_balance and returns 1 if true.
    let (enough_balance) = uint256_le(amount, sender_balance);
    assert_not_zero(enough_balance);

    // Subtract from sender.
    let (new_sender_balance: Uint256) = uint256_sub(sender_balance, amount);
    ERC20_balances.write(sender, new_sender_balance);

    // Add to recipient's balance.
    let (recipient_balance: Uint256) = ERC20_balances.read(account=recipient);
    // Overflow is not possible because sum is guaranteed by mint to be less than total supply.
    let (new_recipient_balance, _: Uint256) = uint256_add(recipient_balance, amount);
    ERC20_balances.write(recipient, new_recipient_balance);
    Transfer.emit(sender, recipient, amount);
    return ();
}

func ERC20_approve{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    caller: felt, spender: felt, amount: Uint256
) {
    assert_not_zero(caller);
    assert_not_zero(spender);
    uint256_check(amount);
    ERC20_allowances.write(caller, spender, amount);
    Approval.emit(caller, spender, amount);
    return ();
}

func ERC20_burn{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(
    account: felt, amount: Uint256
) {
    alloc_locals;
    assert_not_zero(account);
    uint256_check(amount);

    let (balance: Uint256) = ERC20_balances.read(account);
    // Validates amount <= balance and returns 1 if true.
    let (enough_balance) = uint256_le(amount, balance);
    assert_not_zero(enough_balance);

    let (new_balance: Uint256) = uint256_sub(balance, amount);
    ERC20_balances.write(account, new_balance);

    let (supply: Uint256) = ERC20_total_supply.read();
    let (new_supply: Uint256) = uint256_sub(supply, amount);
    ERC20_total_supply.write(new_supply);
    Transfer.emit(account, 0, amount);
    return ();
}
