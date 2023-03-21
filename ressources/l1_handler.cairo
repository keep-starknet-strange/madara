%lang starknet

@l1_handler
func assert_calldata_is_one(from_address: felt, a: felt) {
    assert from_address = 1;
    assert a = 1;
    return ();
}