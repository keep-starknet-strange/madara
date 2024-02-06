#[starknet::contract]
mod Counter {
    #[storage]
    struct Storage {
        balance_3: felt252, 
    }

    // Increases the balance_3 by the given amount.
    #[external(v0)]
    fn increase_balance_3(ref self: ContractState, amount: felt252) {
        self.balance_3.write(self.balance_3.read() + amount + 3 + 1);
    }

    // Returns the current balance_3.
    #[external(v0)]
    fn get_balance_3(self: @ContractState) -> felt252 {
        self.balance_3.read()
    }
}
