use mc_rpc::{Starknet, StarknetRpcApiServer};
use mc_storage::overrides_handle;
use mp_starknet::execution::types::{ContractAddressWrapper, Felt252Wrapper};
use mp_starknet::transaction::types::EventWrapper;
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use sc_cli::SubstrateCli;
use sc_client_api::BlockBackend;
use sc_service::{Arc, WarpSyncParams};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::bounded_vec;
use sp_runtime::BoundedVec;
use starknet_core::types::{BroadcastedInvokeTransactionV1, FieldElement};

use crate::cli::Cli;
use crate::constants;
use crate::service::{build_manual_seal_import_queue, new_partial};

#[tokio::test]
#[serial_test::serial]
async fn filter_events_by_keys_no_chunk_size() {
    let starknet = setup();

    let filter_keys = vec![vec![FieldElement::from(1_u32)], vec![], vec![FieldElement::from(3_u32)]];

    let event1 = build_event_wrapper_for_test(&["0x1", "0x2", "0x3"], 1);
    let event2 = build_event_wrapper_for_test(&["0x1", "", "0x3"], 2);
    let event3 = build_event_wrapper_for_test(&["0x2", "", "0x3"], 3);
    let event4 = build_event_wrapper_for_test(&["0x1"], 4);

    let events = vec![event1.clone(), event2.clone(), event3.clone(), event4.clone()];

    let (filtered_events, _) = starknet.filter_events_by_params(events, None, filter_keys, None);
    assert_eq!(filtered_events.len(), 2);
    assert_eq!(filtered_events[0], event1);
    assert_eq!(filtered_events[1], event2);
}

#[tokio::test]
#[serial_test::serial]
async fn filter_events_by_address_no_chunk_size() {
    let starknet = setup();

    // the keys which should be filtered out
    let filter_keys = vec![vec![]];

    let event1 = build_event_wrapper_for_test(&["0x1", "0x2", "0x3"], 1);
    let event2 = build_event_wrapper_for_test(&["0x1", "", "0x3"], 2);
    let event3 = build_event_wrapper_for_test(&["0x2", "", "0x3"], 3);
    let event4 = build_event_wrapper_for_test(&["0x1"], 4);

    let events = vec![event1.clone(), event2.clone(), event3.clone(), event4.clone()];

    let (filtered_events, _) =
        starknet.filter_events_by_params(events, Some(Felt252Wrapper::from_dec_str("3").unwrap()), filter_keys, None);
    assert_eq!(filtered_events.len(), 1);
    assert_eq!(filtered_events[0], event3);
}

#[tokio::test]
#[serial_test::serial]
async fn filter_events_by_address_and_keys_no_chunk_size() {
    let starknet = setup();

    let filter_keys = vec![vec![FieldElement::from(1_u32)], vec![], vec![FieldElement::from(3_u32)]];

    let event1 = build_event_wrapper_for_test(&["0x1", "0x2", "0x3"], 1);
    let event2 = build_event_wrapper_for_test(&["0x1", "", "0x3"], 2);
    let event3 = build_event_wrapper_for_test(&["0x2", "", "0x3"], 3);
    let event4 = build_event_wrapper_for_test(&["0x1"], 4);
    let event5 = build_event_wrapper_for_test(&["0x1", "", "0x3"], 3);

    let events = vec![event1.clone(), event2.clone(), event3.clone(), event4.clone(), event5.clone()];

    let (filtered_events, _) =
        starknet.filter_events_by_params(events, Some(Felt252Wrapper::from_dec_str("3").unwrap()), filter_keys, None);
    assert_eq!(filtered_events.len(), 1);
    assert_eq!(filtered_events[0], event5);
}

#[tokio::test]
#[serial_test::serial]
async fn filter_events_by_keys_and_chunk_size() {
    let starknet = setup();

    let filter_keys = vec![vec![FieldElement::from(1_u32)], vec![], vec![FieldElement::from(3_u32)]];

    let event1 = build_event_wrapper_for_test(&["0x1", "0x2", "0x3"], 1);
    let event2 = build_event_wrapper_for_test(&["0x1", "", "0x3"], 2);
    let event3 = build_event_wrapper_for_test(&["0x2", "", "0x3"], 3);
    let event4 = build_event_wrapper_for_test(&["0x1"], 4);

    let events = vec![event1.clone(), event2.clone(), event3.clone(), event4.clone()];

    let (filtered_events, continuation_token) = starknet.filter_events_by_params(events, None, filter_keys, Some(1));
    assert_eq!(filtered_events.len(), 1);
    assert_eq!(filtered_events[0], event1);
    assert_eq!(continuation_token, 1);
}

#[tokio::test]
#[serial_test::serial]
async fn filter_events_single_block() {
    let starknet = setup();
    let result = starknet
        .add_invoke_transaction(starknet_core::types::BroadcastedInvokeTransaction::V1(
            BroadcastedInvokeTransactionV1 {
                nonce: FieldElement::from(0_u32),
                max_fee: FieldElement::from(10000000000_u64),
                signature: vec![],
                sender_address: FieldElement::from_hex_be(
                    "0x0000000000000000000000000000000000000000000000000000000000000001",
                )
                .unwrap(),
                calldata: vec![
                    FieldElement::from_hex_be(constants::TOKEN_CONTRACT_ADDRESS).unwrap(),
                    FieldElement::from_hex_be("0x2").unwrap(),
                    FieldElement::from_hex_be("0x1").unwrap(),
                    FieldElement::from_hex_be("0x0000000000000000000000000000000000000000000000000000000000000000")
                        .unwrap(),
                ],
            },
        ))
        .await;
    dbg!(result.unwrap());
}

fn build_event_wrapper_for_test(keys: &[&str], address_int: u64) -> EventWrapper {
    let keys_felt = keys.iter().map(|key| Felt252Wrapper::from_hex_be(key).unwrap()).collect::<Vec<Felt252Wrapper>>();
    EventWrapper {
        keys: BoundedVec::try_from(keys_felt).unwrap(),
        data: bounded_vec!(),
        from_address: ContractAddressWrapper::from(address_int),
        transaction_hash: Felt252Wrapper::from(1_u64),
    }
}

fn setup<T>() -> Starknet<
    sp_runtime::generic::Block<
        sp_runtime::generic::Header<u32, sp_runtime::traits::BlakeTwo256>,
        sp_runtime::OpaqueExtrinsic,
    >,
    T,
    sc_service::client::Client<
        sc_client_db::Backend<
            sp_runtime::generic::Block<
                sp_runtime::generic::Header<u32, sp_runtime::traits::BlakeTwo256>,
                sp_runtime::OpaqueExtrinsic,
            >,
        >,
        sc_service::LocalCallExecutor<
            sp_runtime::generic::Block<
                sp_runtime::generic::Header<u32, sp_runtime::traits::BlakeTwo256>,
                sp_runtime::OpaqueExtrinsic,
            >,
            sc_client_db::Backend<
                sp_runtime::generic::Block<
                    sp_runtime::generic::Header<u32, sp_runtime::traits::BlakeTwo256>,
                    sp_runtime::OpaqueExtrinsic,
                >,
            >,
            sc_executor::NativeElseWasmExecutor<crate::service::ExecutorDispatch>,
        >,
        sp_runtime::generic::Block<
            sp_runtime::generic::Header<u32, sp_runtime::traits::BlakeTwo256>,
            sp_runtime::OpaqueExtrinsic,
        >,
        madara_runtime::RuntimeApi,
    >,
    sc_transaction_pool::BasicPool<
        sc_transaction_pool::FullChainApi<
            sc_service::client::Client<
                sc_client_db::Backend<
                    sp_runtime::generic::Block<
                        sp_runtime::generic::Header<u32, sp_runtime::traits::BlakeTwo256>,
                        sp_runtime::OpaqueExtrinsic,
                    >,
                >,
                sc_service::LocalCallExecutor<
                    sp_runtime::generic::Block<
                        sp_runtime::generic::Header<u32, sp_runtime::traits::BlakeTwo256>,
                        sp_runtime::OpaqueExtrinsic,
                    >,
                    sc_client_db::Backend<
                        sp_runtime::generic::Block<
                            sp_runtime::generic::Header<u32, sp_runtime::traits::BlakeTwo256>,
                            sp_runtime::OpaqueExtrinsic,
                        >,
                    >,
                    sc_executor::NativeElseWasmExecutor<crate::service::ExecutorDispatch>,
                >,
                sp_runtime::generic::Block<
                    sp_runtime::generic::Header<u32, sp_runtime::traits::BlakeTwo256>,
                    sp_runtime::OpaqueExtrinsic,
                >,
                madara_runtime::RuntimeApi,
            >,
            sp_runtime::generic::Block<
                sp_runtime::generic::Header<u32, sp_runtime::traits::BlakeTwo256>,
                sp_runtime::OpaqueExtrinsic,
            >,
        >,
        sp_runtime::generic::Block<
            sp_runtime::generic::Header<u32, sp_runtime::traits::BlakeTwo256>,
            sp_runtime::OpaqueExtrinsic,
        >,
    >,
    mp_starknet::crypto::hash::Hasher,
> {
    let cli = Cli::from_iter(["--dev", "--sealing=instant"].iter());

    let config = cli.create_configuration(&cli.run, tokio::runtime::Handle::try_current().unwrap()).unwrap();

    let build_import_queue = build_manual_seal_import_queue;

    let sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container: _,
        select_chain: _,
        transaction_pool,
        other: (_, grandpa_link, _, madara_backend),
    } = new_partial(&config, build_import_queue).unwrap();

    let starting_block = client.info().best_number;
    let overrides = overrides_handle(client.clone());
    let hasher = client.runtime_api().get_hasher(client.info().best_hash).unwrap().into();

    let mut net_config = sc_network::config::FullNetworkConfiguration::new(&config.network);

    let grandpa_protocol_name = sc_consensus_grandpa::protocol_standard_name(
        &client.block_hash(0).ok().flatten().expect("Genesis block exists; qed"),
        &config.chain_spec,
    );

    net_config.add_notification_protocol(sc_consensus_grandpa::grandpa_peers_set_config(grandpa_protocol_name.clone()));
    let warp_sync = Arc::new(sc_consensus_grandpa::warp_proof::NetworkProvider::new(
        backend.clone(),
        grandpa_link.shared_authority_set().clone(),
        Vec::default(),
    ));
    let warp_sync_params = Some(WarpSyncParams::WithProvider(warp_sync));

    let (_, _, _, _, sync_service) = sc_service::build_network(sc_service::BuildNetworkParams {
        config: &config,
        net_config,
        client: client.clone(),
        transaction_pool: transaction_pool.clone(),
        spawn_handle: task_manager.spawn_handle(),
        import_queue,
        block_announce_validator_builder: None,
        warp_sync_params,
    })
    .unwrap();

    Starknet::new(
        client.clone(),
        madara_backend.clone(),
        overrides,
        transaction_pool.clone(),
        sync_service,
        starting_block,
        hasher,
    )
}
