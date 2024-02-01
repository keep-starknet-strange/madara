#[starknet::contract]
mod Counter {
    #[storage]
    struct Storage {
        balance_0: felt252, 
    }

    // Increases the balance_0 by the given amount.
    #[external(v0)]
    fn increase_balance_0(ref self: ContractState, amount: felt252) {
        self.balance_0.write(self.balance_0.read() + amount + 0 + 1);
    }

    // Returns the current balance_0.
    #[external(v0)]
    fn get_balance_0(self: @ContractState) -> felt252 {
        self.balance_0.read()
    }
}
