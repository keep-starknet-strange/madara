#[starknet::contract]
mod Account {
    use array::{ArrayTrait, SpanTrait};
    use box::BoxTrait;
    use ecdsa::check_ecdsa_signature;
    use option::OptionTrait;
    use starknet::account::Call;
    use starknet::{ContractAddress, call_contract_syscall};
    use zeroable::Zeroable;
    use array::ArraySerde;

    #[storage]
    struct Storage {
        public_key: felt252
    }

    #[constructor]
    fn constructor(ref self: ContractState, public_key_: felt252) {
        self.public_key.write(public_key_);
    }

    trait StorageTrait {
        fn validate_transaction(self: @ContractState) -> felt252;
    }
    impl StorageImpl of StorageTrait {
        fn validate_transaction(self: @ContractState) -> felt252 {
            starknet::VALIDATED
        }
    }


    #[abi(embed_v0)]
    fn __validate_deploy__(
        self: @ContractState,
        class_hash: felt252,
        contract_address_salt: felt252,
        public_key_: felt252
    ) -> felt252 {
        self.validate_transaction()
    }

    impl AccountContractImpl of starknet::account::AccountContract<ContractState> {
        fn __validate_declare__(self: @ContractState, class_hash: felt252) -> felt252 {
            self.validate_transaction()
        }

        fn __validate__(ref self: ContractState, calls: Array<Call>) -> felt252 {
            self.validate_transaction()
        }

        fn __execute__(ref self: ContractState, mut calls: Array<Call>) -> Array<Span<felt252>> {
            // Validate caller.
            assert(starknet::get_caller_address().is_zero(), 'INVALID_CALLER');

            // Check the tx version here, since version 0 transaction skip the __validate__ function.
            let tx_info = starknet::get_tx_info().unbox();
            assert(tx_info.version != 0, 'INVALID_TX_VERSION');

            let mut result = ArrayTrait::new();
            loop {
                match calls.pop_front() {
                    Option::Some(call) => {
                        let mut res = call_contract_syscall(
                            address: call.to,
                            entry_point_selector: call.selector,
                            calldata: call.calldata
                        )
                            .unwrap();
                        result.append(res);
                    },
                    Option::None(()) => {
                        break; // Can't break result; because of 'variable was previously moved'
                    },
                };
            };
            result
        }
    }
}
