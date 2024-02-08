#[starknet::contract]
mod Counter {
    #[storage]
    struct Storage {
        balance_6: felt252, 
    }

    // Increases the balance_6 by the given amount.
    #[external(v0)]
    fn increase_balance_6(ref self: ContractState, amount: felt252) {
        self.balance_6.write(self.balance_6.read() + amount + 6 + 1);
    }

    // Returns the current balance_6.
    #[external(v0)]
    fn get_balance_6(self: @ContractState) -> felt252 {
        self.balance_6.read()
    }
}
