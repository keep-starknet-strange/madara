#[starknet::contract]
mod Counter {
    #[storage]
    struct Storage {
        balance_0: felt252, 
    }

    #[external(v0)]
    fn increase_balance(ref self: ContractState, amount: felt252) {
        self.balance_0.write(self.balance_0.read() + amount);
    }

    #[external(v0)]
    fn get_balance(self: @ContractState) -> felt252 {
        self.balance_0.read()
    }
}
