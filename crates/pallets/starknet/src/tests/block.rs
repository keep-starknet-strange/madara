use alloc::sync::Arc;

use frame_support::assert_ok;
use mp_digest_log::{ensure_log, find_starknet_block};
use mp_felt::Felt252Wrapper;
use mp_sequencer_address::DEFAULT_SEQUENCER_ADDRESS;
use starknet_api::api_core::{ChainId, ContractAddress, PatriciaKey};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::hash::StarkFelt;

use super::mock::default_mock::*;
use super::mock::*;
use crate::tests::constants::FEE_TOKEN_ADDRESS;
use crate::tests::get_invoke_dummy;
use crate::{Config, SeqAddrUpdate, SequencerAddress};

#[test]
fn store_block_no_pending_transactions_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        // initialize first block
        let header = System::finalize();
        const BLOCK_NUMBER: u64 = 1;
        System::initialize(&BLOCK_NUMBER, &header.hash(), &Default::default());

        // testing store_block
        Starknet::store_block(BLOCK_NUMBER);
        // check digest saved
        // check saved digest is correct, 0 transactions
        let digest = frame_system::Pallet::<MockRuntime>::digest();
        let block = find_starknet_block(&digest).unwrap();
        assert_ok!(ensure_log(&digest));
        assert_eq!(0, block.transactions().len());

        // check BlockHash correct
        let blockhash = block.header().hash::<<MockRuntime as Config>::SystemHash>();
        assert_eq!(blockhash, Starknet::block_hash(BLOCK_NUMBER));
        // check pending storage killed
        assert_eq!(0, Starknet::pending().len());
        assert_eq!(0, Starknet::pending_hashes().len());
        assert_eq!(0, Starknet::event_count());
    });
}

#[test]
fn store_block_with_pending_transactions_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        // initialize first block
        let header = System::finalize();
        const BLOCK_NUMBER: u64 = 1;
        System::initialize(&BLOCK_NUMBER, &header.hash(), &Default::default());

        SeqAddrUpdate::<MockRuntime>::put(true);
        let default_addr = ContractAddress(PatriciaKey(StarkFelt::new(DEFAULT_SEQUENCER_ADDRESS).unwrap()));
        SequencerAddress::<MockRuntime>::put(default_addr);

        // perform transactions
        // first invoke transaction
        let transaction = get_invoke_dummy(Felt252Wrapper::ZERO);

        assert_ok!(Starknet::invoke(RuntimeOrigin::none(), transaction.into()));

        // second invoke transaction
        let transaction = get_invoke_dummy(Felt252Wrapper::ONE);

        assert_ok!(Starknet::invoke(RuntimeOrigin::none(), transaction.into()));

        // testing store_block
        Starknet::store_block(BLOCK_NUMBER);
        // check digest saved
        // check saved digest is correct, transactions included
        let digest = frame_system::Pallet::<MockRuntime>::digest();
        let block = find_starknet_block(&digest).unwrap();
        assert_ok!(ensure_log(&digest));
        assert_eq!(2, block.transactions().len());

        // check BlockHash correct
        let blockhash = block.header().hash::<<MockRuntime as Config>::SystemHash>();
        assert_eq!(blockhash, Starknet::block_hash(BLOCK_NUMBER));
        // check pending storage killed
        assert_eq!(0, Starknet::pending().len());
        assert_eq!(0, Starknet::event_count());
    });
}

#[test]
fn get_block_context_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        // initialize first block
        let header = System::finalize();
        const BLOCK_NUMBER: u64 = 1;
        System::initialize(&BLOCK_NUMBER, &header.hash(), &Default::default());

        SeqAddrUpdate::<MockRuntime>::put(true);
        let default_addr = ContractAddress(PatriciaKey(StarkFelt::new(DEFAULT_SEQUENCER_ADDRESS).unwrap()));
        SequencerAddress::<MockRuntime>::put(default_addr);

        let block_context = Starknet::get_block_context();
        // correct block_number
        assert_eq!(BlockNumber(BLOCK_NUMBER), block_context.block_number);
        // correct block_timestamp
        assert_eq!(BlockTimestamp(0), block_context.block_timestamp);
        // correct chain_id
        assert_eq!(ChainId(Starknet::chain_id_str()), block_context.chain_id);
        // correct sequencer_address
        assert_eq!(default_addr, block_context.sequencer_address);
        // correct fee_token_address
        assert_eq!(
            ContractAddress::try_from(StarkFelt::try_from(FEE_TOKEN_ADDRESS).unwrap()).unwrap(),
            block_context.fee_token_address
        );
        // correct vm_resource_fee_cost
        let vm_resoursce_fee_cost: Arc<_> = Default::default();
        assert_eq!(vm_resoursce_fee_cost, block_context.vm_resource_fee_cost);
        // correct invoke_tx_max_n_steps: T::InvokeTxMaxNSteps::get(),
        assert_eq!(InvokeTxMaxNSteps::get(), block_context.invoke_tx_max_n_steps);
        // correct validate_max_n_steps: T::ValidateMaxNSteps::get(),
        assert_eq!(ValidateMaxNSteps::get(), block_context.validate_max_n_steps);
        // correct gas_price,
        assert_eq!(10, block_context.gas_price);
    });
}
