#[starknet::contract]
mod Counter2 {
    #[storage]
    struct Storage {
        balance_2: felt252, 
    }

    // Increases the balance_2 by the given amount.
    #[external(v0)]
    fn increase_balance_2(ref self: ContractState, amount: felt252) {
        self.balance_2.write(self.balance_2.read() + amount + 2 + 1);
    }

    // Returns the current balance_2.
    #[external(v0)]
    fn get_balance_2(self: @ContractState) -> felt252 {
        self.balance_2.read()
    }
}
