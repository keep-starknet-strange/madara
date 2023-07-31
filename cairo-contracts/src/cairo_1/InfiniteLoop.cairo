#[starknet::contract]
mod InfiniteLoop {
    #[storage]
    struct Storage {
        counter: u256, 
    }

    #[external]
    fn infinite_loop(ref self: ContractState) {
        self.counter.write(1);
        loop {
            if self.counter.read() < 1 {
                break ();
            }
            self.counter.write(self.counter.read() + 1);
        }
    }
}
