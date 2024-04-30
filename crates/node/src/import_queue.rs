//! Helpers for building the block import queue and pipeline.
//!
//! Block import queue is an implementation of the chain responsibility pattern,
//! where the very last handler is the client and each preceeding handler wraps the
//! current one forming an extra layer.
//!
//! Read more about the block import pipeline:
//!   * https://docs.substrate.io/learn/transaction-lifecycle/#block-authoring-and-block-imports
//!   * https://doc.deepernetwork.org/v3/advanced/block-import/
//!   * https://substrate.stackexchange.com/search?q=block+import
//!
//! In order to avoid confusion:
//!     - Block import is a trait that all handlers in the queue/pipeline have to implement;
//!     - Import queue is a struct that sc_service::build_network accepts as a configuration
//!       parameter;
//!     - Import pipeline is a helper struct that encapsulates a BlockImport used to import own
//!       (authored) blocks plus optionally a link (hook) to Grandpa consensus.
//!
//! Import queue is used to import external blocks (created by other nodes), so typically it would
//! look like:
//!     Aura: ImportQueueVerifier -> [Grandpa: BlockImport -> Client: BlockImport]
//!
//! For own (authored) blocks the chain looks similar (although a bit different):
//!     Aura: BlockAuthoringTask -> [Grandpa: BlockImport -> Client: BlockImport]
//!
//! Read more about consensus:
//!     * https://docs.substrate.io/learn/consensus/
//!     * https://substrate.stackexchange.com/questions/5918/how-can-i-make-my-node-generate-blocks-only-when-they-receive-transactions

use std::sync::Arc;

use madara_runtime::opaque::Block;
use sc_consensus::{BasicQueue, BoxBlockImport, BoxJustificationImport};
use sc_consensus_aura::BuildVerifierParams;
use sc_consensus_grandpa::LinkHalf;
use sc_service::{Configuration, Error as ServiceError, TaskManager};
use sc_telemetry::Telemetry;
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;

use crate::service::{BasicImportQueue, FullClient, FullSelectChain};
use crate::starknet::MadaraBackend;

/// The minimum period of blocks on which justifications will be
/// imported and generated.
pub const GRANDPA_JUSTIFICATION_PERIOD: u32 = 512;

/// Grandpa link (hook) type.
///
/// Import queue provides hooks via the Link trait that can be used to follow its progress.
type GrandpaLinkT = LinkHalf<Block, FullClient, FullSelectChain>;

/// Block import pipeline is a helper struct encapsulating the authored (own) block import and
/// optionally a link (hook) to the Grandpa block import.
pub struct BlockImportPipeline {
    pub block_import: BoxBlockImport<Block>,
    pub grandpa_link: Option<GrandpaLinkT>,
}

/// Build the import queue for default sealing given the block import.
fn build_aura_import_queue(
    client: Arc<FullClient>,
    config: &Configuration,
    task_manager: &TaskManager,
    telemetry: &Option<Telemetry>,
    block_import: BoxBlockImport<Block>,
    justification_import: Option<BoxJustificationImport<Block>>,
) -> Result<BasicImportQueue, ServiceError> {
    let slot_duration = sc_consensus_aura::slot_duration(&*client)?;

    let create_inherent_data_providers = move |_, ()| async move {
        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
        let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
            *timestamp,
            slot_duration,
        );
        Ok((slot, timestamp))
    };

    // We are not using `import_queue` because it requires knowledge of BlockImportT at compile time,
    // although it's not necessary.
    let verifier = sc_consensus_aura::build_verifier::<AuraPair, _, _, _>(BuildVerifierParams {
        client,
        create_inherent_data_providers,
        check_for_equivocation: Default::default(),
        telemetry: telemetry.as_ref().map(|x| x.handle()),
        compatibility_mode: sc_consensus_aura::CompatibilityMode::None,
    });

    Ok(BasicQueue::new(
        verifier,
        block_import,
        justification_import,
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
    ))
}

/// Build a block import queue & pipeline for default sealing.
///
/// If Starknet block import (Sierra class verification) is enabled for prod:
///     Queue (external blocks): AuraVerifier -> StarknetBlockImport -> GrandpaBlockImport -> Client
///     Pipeline (authored blocks): GrandpaBlockImport -> Client
///
/// If Starknet block import is enabled for testing:
///     Pipeline (authored blocks): StarknetBlockImport -> GrandpaBlockImport -> Client
///
/// Otherwise:
///     Queue (external blocks): AuraVerifier -> GrandpaBlockImport -> Client
///     Pipeline (authored blocks): GrandpaBlockImport -> Client
#[allow(unused_variables)]
pub fn build_aura_queue_grandpa_pipeline(
    client: Arc<FullClient>,
    config: &Configuration,
    task_manager: &TaskManager,
    telemetry: &Option<Telemetry>,
    select_chain: FullSelectChain,
    madara_backend: Arc<MadaraBackend>,
) -> Result<(BasicImportQueue, BlockImportPipeline), ServiceError> {
    let (block_import, link) = sc_consensus_grandpa::block_import(
        client.clone(),
        GRANDPA_JUSTIFICATION_PERIOD,
        &client as &Arc<_>,
        select_chain,
        telemetry.as_ref().map(|x| x.handle()),
    )?;

    #[cfg(feature = "sn-block-import")]
    let block_import = mc_starknet_block_import::StarknetBlockImport::new(block_import, client.clone(), madara_backend);

    let import_queue = build_aura_import_queue(
        client.clone(),
        config,
        task_manager,
        telemetry,
        Box::new(block_import.clone()),
        Some(Box::new(block_import.clone())),
    )?;

    // We do not run Sierra class verification for own blocks, unless it's for testing purposes.
    // Here we return GrandpaBlockImport which is wrapped by the outer StarknetBlockImport.
    #[cfg(all(feature = "sn-block-import", not(feature = "sn-block-import-testing")))]
    let block_import = block_import.unwrap();

    let import_pipeline = BlockImportPipeline { block_import: Box::new(block_import), grandpa_link: Some(link) };

    Ok((import_queue, import_pipeline))
}

/// Build a block import queue & pipeline for manual/instant sealing.
///
/// If Starknet block import (Sierra class verification) is enabled for testing:
///     Queue (external blocks): StarknetBlockImport -> Client
///     Pipeline: StarknetBlockImport -> Client
///
/// Otherwise:
///     Queue (external blocks): Client
///     Pipeline (authored blocks): Client
#[allow(unused_variables)]
pub fn build_manual_seal_queue_pipeline(
    client: Arc<FullClient>,
    config: &Configuration,
    task_manager: &TaskManager,
    madara_backend: Arc<MadaraBackend>,
) -> (BasicImportQueue, BlockImportPipeline) {
    #[cfg(not(feature = "sn-block-import-testing"))]
    let block_import = client.clone();

    #[cfg(feature = "sn-block-import-testing")]
    let block_import =
        mc_starknet_block_import::StarknetBlockImport::new(client.clone(), client.clone(), madara_backend);

    let import_queue = sc_consensus_manual_seal::import_queue(
        Box::new(block_import.clone()),
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
    );

    let import_pipeline = BlockImportPipeline { block_import: Box::new(block_import), grandpa_link: None };

    (import_queue, import_pipeline)
}
