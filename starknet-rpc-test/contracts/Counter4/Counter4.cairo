#[starknet::contract]
mod Counter4 {
    #[storage]
    struct Storage {
        balance_4: felt252, 
    }

    // Increases the balance_4 by the given amount.
    #[external(v0)]
    fn increase_balance_4(ref self: ContractState, amount: felt252) {
        self.balance_4.write(self.balance_4.read() + amount + 4 + 1);
    }

    // Returns the current balance_4.
    #[external(v0)]
    fn get_balance_4(self: @ContractState) -> felt252 {
        self.balance_4.read()
    }
}
