#[starknet::contract]
mod Counter {
    #[storage]
    struct Storage {
        balance_1: felt252, 
    }

    // Increases the balance_1 by the given amount.
    #[external(v0)]
    fn increase_balance_1(ref self: ContractState, amount: felt252) {
        self.balance_1.write(self.balance_1.read() + amount + 1 + 1);
    }

    // Returns the current balance_1.
    #[external(v0)]
    fn get_balance_1(self: @ContractState) -> felt252 {
        self.balance_1.read()
    }
}
