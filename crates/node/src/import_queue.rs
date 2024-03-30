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
//!     - Import pipeline is a helper struct that encapsulates a particular BlockImport
//!       implementation;
//!
//! Import queue is used to import external blocks (created by other nodes), so typically it would
//! look like:     Aura: ImportQueueVerifier -> [Grandpa: BlockImport -> Client: BlockImport]
//!
//! For own (authored) blocks the chain looks similar (although a bit different):
//!     Aura: BlockAuthoringTask -> [Grandpa: BlockImport -> Client: BlockImport]
//!
//! So the common part is basically our block import pipeline.
//!
//! Read more about consensus:
//!     * https://docs.substrate.io/learn/consensus/
//!     * https://substrate.stackexchange.com/questions/5918/how-can-i-make-my-node-generate-blocks-only-when-they-receive-transactions

use std::sync::Arc;

use madara_runtime::opaque::Block;
use madara_runtime::Hash;
use sc_consensus::{BasicQueue, BoxBlockImport, BoxJustificationImport};
use sc_consensus_aura::BuildVerifierParams;
use sc_consensus_grandpa::{GrandpaBlockImport, LinkHalf, SharedAuthoritySet};
use sc_service::{Configuration, Error as ServiceError, TaskManager};
use sc_telemetry::Telemetry;
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;
use sp_runtime::traits::NumberFor;

use crate::service::{BasicImportQueue, FullBackend, FullClient, FullSelectChain};
use crate::starknet::MadaraBackend;

/// The minimum period of blocks on which justifications will be
/// imported and generated.
pub const GRANDPA_JUSTIFICATION_PERIOD: u32 = 512;

/// Outer block import type.
///
/// This is the type of the first handler in the import pipeline.
/// If Sierra class verification is enabled, then it's going to be StarknetBlockImport
/// which wraps GrandpaBlockImport.
#[cfg(feature = "sn-block-import")]
type OuterBlockImportT = mc_starknet_block_import::StarknetBlockImport<
    GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>,
    FullClient,
>;
/// If Sierra class verification is disabled, the outer block import is just GrandpaBlockImport.
#[cfg(not(feature = "sn-block-import"))]
type OuterBlockImportT = GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>;

/// Grandpa link (hook) type.
///
/// Import queue provides hooks via the Link trait that can be used to follow its progress.
type GrandpaLinkT = LinkHalf<Block, FullClient, FullSelectChain>;

/// Block import pipeline is a helper struct encapsulating the actual block import type.
pub struct BlockImportPipeline {
    /// In the simplest case (manual seal) the pipeline consists of Client only
    client: Arc<FullClient>,
    outer_block_import: Option<OuterBlockImportT>,
    grandpa_link: Option<GrandpaLinkT>,
}

impl BlockImportPipeline {
    /// External (foreign) blocks import, intended for use in the import queue.
    pub fn external_block_import(&self) -> BoxBlockImport<Block> {
        if let Some(block_import) = &self.outer_block_import {
            Box::new(block_import.clone())
        } else {
            Box::new(self.client.clone())
        }
    }

    /// External (foreign) justifications (endorsements, attestations) import, intended for use in
    /// the import queue.
    pub fn justification_import(&self) -> Option<BoxJustificationImport<Block>> {
        if let Some(block_import) = &self.outer_block_import { Some(Box::new(block_import.clone())) } else { None }
    }

    /// Authored (own) blocks import, intended for use in the block creating task.
    pub fn authored_block_import(&self) -> BoxBlockImport<Block> {
        if let Some(block_import) = &self.outer_block_import {
            #[cfg(all(feature = "sn-block-import", not(feature = "sn-block-import-testing")))]
            {
                // We do not run Sierra class verification for own blocks, unless it's for testing purposes.
                // Here we return GrandpaBlockImport which is wrapped by the outer StarknetBlockImport.
                return Box::new(block_import.inner().clone());
            }
            Box::new(block_import.clone())
        } else {
            Box::new(self.client.clone())
        }
    }

    /// Get Grandpa shared authority set if it's initialized (not the case for manual seal mode).
    pub fn grandpa_authority_set(&self) -> Option<SharedAuthoritySet<Hash, NumberFor<Block>>> {
        self.grandpa_link.as_ref().map(|link| link.shared_authority_set().clone())
    }

    /// Pop Grandpa link (hook) if it's initialized (not the case for manual seal mode).
    ///
    /// Since GrandpaLinkT is not clonable, this is a one time operation.
    pub fn remove_grandpa_link(&mut self) -> Option<GrandpaLinkT> {
        std::mem::take(&mut self.grandpa_link)
    }
}

/// Build the import queue for default sealing given the block import.
pub fn build_aura_import_queue(
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

/// Build the import queue for manual sealing given the block import.
pub fn build_manual_seal_import_queue(
    block_import: BoxBlockImport<Block>,
    config: &Configuration,
    task_manager: &TaskManager,
) -> Result<BasicImportQueue, ServiceError> {
    let import_queue = sc_consensus_manual_seal::import_queue(
        block_import,
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
    );
    Ok(import_queue)
}

/// Build a block import pipeline for default sealing.
///
/// If Starknet block import (Sierra class verification) is enabled for prod, the following pipeline
/// is used:     StarknetBlockImport -> GrandpaBlockImport -> Client
///
///  Otherwise:
///     GrandpaBlockImport -> Client
#[allow(unused_variables, unused_mut)]
pub fn build_grandpa_pipeline(
    client: Arc<FullClient>,
    select_chain: FullSelectChain,
    telemetry: &Option<Telemetry>,
    madara_backend: Arc<MadaraBackend>,
) -> Result<BlockImportPipeline, ServiceError> {
    let (mut block_import, link) = sc_consensus_grandpa::block_import(
        client.clone(),
        GRANDPA_JUSTIFICATION_PERIOD,
        &client as &Arc<_>,
        select_chain,
        telemetry.as_ref().map(|x| x.handle()),
    )?;

    #[cfg(feature = "sn-block-import")]
    {
        block_import = mc_starknet_block_import::StarknetBlockImport::new(block_import, client.clone(), madara_backend);
    }

    Ok(BlockImportPipeline { client, outer_block_import: Some(block_import), grandpa_link: Some(link) })
}

/// Build a block import pipeline for manual/instant sealing.
///
/// If Starknet block import (Sierra class verification) is enabled for testing, the following
/// pipeline is used:     StarknetBlockImport -> Client
///
///  Otherwise it will contain of the Client only.
#[allow(unused_variables, unused_mut)]
pub fn build_manual_seal_pipeline(client: Arc<FullClient>, madara_backend: Arc<MadaraBackend>) -> BlockImportPipeline {
    let mut outer_block_import = None;

    #[cfg(feature = "sn-block-import-testing")]
    {
        outer_block_import =
            Some(mc_starknet_block_import::StarknetBlockImport::new(client.clone(), client.clone(), madara_backend));
    }

    BlockImportPipeline { client, outer_block_import, grandpa_link: None }
}
