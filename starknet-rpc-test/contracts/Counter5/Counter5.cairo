#[starknet::contract]
mod Counter5 {
    #[storage]
    struct Storage {
        balance_5: felt252, 
    }

    // Increases the balance_5 by the given amount.
    #[external(v0)]
    fn increase_balance_5(ref self: ContractState, amount: felt252) {
        self.balance_5.write(self.balance_5.read() + amount + 5 + 1);
    }

    // Returns the current balance_5.
    #[external(v0)]
    fn get_balance_5(self: @ContractState) -> felt252 {
        self.balance_5.read()
    }
}
