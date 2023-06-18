// Copyright (C) 2023 Polytope Labs (Caymans) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Simnode for Standalone runtimes with Aura Consensus
use futures::channel::mpsc;
use futures::future::Either;
use futures::StreamExt;
use mc_storage::overrides_handle;
use sc_consensus_manual_seal::consensus::timestamp::SlotTimestampProvider;
use sc_consensus_manual_seal::rpc::{ManualSeal, ManualSealApiServer};
use sc_consensus_manual_seal::{run_manual_seal, EngineCommand, ManualSealParams};
use sc_executor_common::wasm_runtime::{HeapAllocStrategy, DEFAULT_HEAP_ALLOC_STRATEGY};
use sc_service::error::Error as ServiceError;
use sc_service::{build_network, spawn_tasks, BuildNetworkParams, Configuration, SpawnTasksParams, TaskManager};
use sc_simnode::{SimnodeApiServer, SimnodeRpcHandler};
use sc_transaction_pool_api::TransactionPool;
use sp_blockchain::HeaderBackend;

use crate::{command, rpc, service};

/// Set up and run simnode
pub async fn start_simnode(config: Configuration) -> Result<TaskManager, ServiceError> {
    use sc_consensus_manual_seal::consensus::aura::AuraConsensusDataProvider;

    let instant = true;

    let heap_pages = config
        .default_heap_pages
        .map_or(DEFAULT_HEAP_ALLOC_STRATEGY, |h| HeapAllocStrategy::Static { extra_pages: h as _ });

    let executor = sc_simnode::Executor::builder()
        .with_execution_method(config.wasm_method)
        .with_onchain_heap_alloc_strategy(heap_pages)
        .with_offchain_heap_alloc_strategy(heap_pages)
        .with_max_runtime_instances(config.max_runtime_instances)
        .with_runtime_cache_size(config.runtime_cache_size)
        .build();

    // pass the custom executor along
    let service::NewPartialComponents {
        client,
        backend,
        mut task_manager,
        keystore_container,
        select_chain,
        import_queue,
        transaction_pool,
        other: (block_import, _grandpa_link, mut telemetry, madara_backend),
    } = service::new_partial::<_, _>(&config, service::build_aura_grandpa_import_queue, executor)?;

    let net_config = sc_network::config::FullNetworkConfiguration::new(&config.network);

    let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) = {
        let params = BuildNetworkParams {
            config: &config,
            net_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync_params: None,
        };
        build_network(params)?
    };

    // offchain workers
    sc_service::build_offchain_workers(&config, task_manager.spawn_handle(), client.clone(), network.clone());

    // Proposer object for block authorship.
    let env = sc_basic_authorship::ProposerFactory::new(
        task_manager.spawn_handle(),
        client.clone(),
        transaction_pool.clone(),
        config.prometheus_registry(),
        None,
    );

    let overrides = overrides_handle(client.clone());
    let starting_block = client.info().best_number;

    let starknet_rpc_params = rpc::StarknetDeps {
        client: client.clone(),
        madara_backend: madara_backend.clone(),
        overrides,
        sync_service: sync_service.clone(),
        starting_block,
    };

    let rpc_extensions_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();
        let graph = transaction_pool.pool().clone();

        Box::new(move |deny_unsafe, _| {
            let deps = rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                graph: graph.clone(),
                deny_unsafe,
                starknet: starknet_rpc_params.clone(),
                command_sink: None,
            };
            crate::rpc::create_full(deps)
        })
    };

    // Channel for the rpc handler to communicate with the authorship task.
    let (command_sink, commands_stream) = mpsc::channel(10);

    let rpc_sink = command_sink.clone();

    let rpc_handlers = {
        let client = client.clone();
        let backend = backend.clone();
        let params = SpawnTasksParams {
            config,
            client: client.clone(),
            backend: backend.clone(),
            task_manager: &mut task_manager,
            keystore: keystore_container.keystore(),
            transaction_pool: transaction_pool.clone(),
            rpc_builder: Box::new(move |deny_unsafe, subscription_executor| {
                let mut io = rpc_extensions_builder(deny_unsafe, subscription_executor)?;

                io.merge(SimnodeRpcHandler::<command::RuntimeInfo>::new(client.clone(), backend.clone()).into_rpc())
                    .map_err(|_| sc_service::Error::Other("Unable to merge simnode rpc api".to_string()))?;

                io.merge(ManualSeal::new(rpc_sink.clone()).into_rpc())
                    .map_err(|_| sc_service::Error::Other("Unable to merge manual seal rpc api".to_string()))?;
                Ok(io)
            }),
            network,
            system_rpc_tx,
            tx_handler_controller,
            sync_service,
            telemetry: telemetry.as_mut(),
        };
        spawn_tasks(params)?
    };

    network_starter.start_network();
    let _rpc_handler = rpc_handlers.handle();

    run_manual_seal(ManualSealParams {
        block_import,
        env,
        client: client.clone(),
        pool: transaction_pool.clone(),
        commands_stream: if instant {
            let tx_notifications =
                transaction_pool.import_notification_stream().map(move |_| EngineCommand::SealNewBlock {
                    create_empty: true,
                    // parachains need their blocks finalized instantly to be part of the main
                    // chain.
                    finalize: true,
                    parent_hash: None,
                    sender: None,
                });

            Either::Left(futures::stream::select(tx_notifications, commands_stream))
        } else {
            Either::Right(commands_stream)
        },
        select_chain,
        consensus_data_provider: Some(Box::new(AuraConsensusDataProvider::new(client.clone()))),
        create_inherent_data_providers: {
            let client = client.clone();
            move |_, _| {
                let client = client.clone();
                async move {
                    let client = client.clone();

                    let timestamp = SlotTimestampProvider::new_aura(client).map_err(|err| format!("{:?}", err))?;

                    let aura = sp_consensus_aura::inherents::InherentDataProvider::new(timestamp.slot());

                    Ok((timestamp, aura))
                }
            }
        },
    })
    .await;

    Ok(task_manager)
}
