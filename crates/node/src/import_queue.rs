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

#[cfg(feature = "sn-block-import")]
type OuterBlockImportT = mc_starknet_block_import::StarknetBlockImport<
    GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>,
    FullClient,
>;

#[cfg(not(feature = "sn-block-import"))]
type OuterBlockImportT = GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>;

type GrandpaLinkT = LinkHalf<Block, FullClient, FullSelectChain>;

pub struct BlockImportPipeline {
    client: Arc<FullClient>,
    outer_block_import: Option<OuterBlockImportT>,
    grandpa_link: Option<GrandpaLinkT>,
}

impl BlockImportPipeline {
    pub fn external_block_import(&self) -> BoxBlockImport<Block> {
        if let Some(block_import) = &self.outer_block_import {
            Box::new(block_import.clone())
        } else {
            Box::new(self.client.clone())
        }
    }

    pub fn justification_import(&self) -> Option<BoxJustificationImport<Block>> {
        if let Some(block_import) = &self.outer_block_import { Some(Box::new(block_import.clone())) } else { None }
    }

    pub fn authored_block_import(&self) -> BoxBlockImport<Block> {
        if let Some(block_import) = &self.outer_block_import {
            #[cfg(all(feature = "sn-block-import", not(feature = "sn-block-import-testing")))]
            {
                return Box::new(block_import.inner().clone());
            }
            Box::new(block_import.clone())
        } else {
            Box::new(self.client.clone())
        }
    }

    pub fn grandpa_authority_set(&self) -> Option<SharedAuthoritySet<Hash, NumberFor<Block>>> {
        self.grandpa_link.as_ref().map(|link| link.shared_authority_set().clone())
    }

    pub fn remove_grandpa_link(&mut self) -> Option<GrandpaLinkT> {
        std::mem::take(&mut self.grandpa_link)
    }
}

/// Build the import queue for the template runtime (aura + grandpa + starknet).
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

/// Build the import queue for the template runtime (manual seal).
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
