#[starknet::contract]
mod TransactionRevert {
    #[storage]
    struct Storage {
        balance: felt252, 
    }

    // Increases the balance by the given amount and then panics.
    #[external(v0)]
    fn increase_balance(ref self: ContractState, amount: felt252) {
        self.balance.write(self.balance.read() + amount);
        panic(array![2]);
    }

    // Returns the current balance.
    #[external(v0)]
    fn get_balance(self: @ContractState) -> felt252 {
        self.balance.read()
    }
}
