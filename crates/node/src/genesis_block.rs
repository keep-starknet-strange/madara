use std::marker::PhantomData;
use std::sync::Arc;

use mp_digest_log::{Log, MADARA_ENGINE_ID};
use mp_starknet::block::Block as StarknetBlock;
use sc_client_api::backend::Backend;
use sc_client_api::BlockImportOperation;
use sc_executor::RuntimeVersionOf;
use sc_service::{resolve_state_version_from_wasm, BuildGenesisBlock};
use sp_api::Encode;
use sp_core::storage::{StateVersion, Storage};
use sp_runtime::traits::{Block as BlockT, Hash as HashT, Header as HeaderT, Zero};
use sp_runtime::{BuildStorage, Digest, DigestItem};

/// Custom genesis block builder for Madara.
pub struct MadaraGenesisBlockBuilder<Block: BlockT, B, E> {
    genesis_storage: Storage,
    commit_genesis_state: bool,
    backend: Arc<B>,
    executor: E,
    _phantom: PhantomData<Block>,
}

impl<Block: BlockT, B: Backend<Block>, E: RuntimeVersionOf> MadaraGenesisBlockBuilder<Block, B, E> {
    /// Constructs a new instance of [`MadaraGenesisBlockBuilder`].
    pub fn new(
        build_genesis_storage: &dyn BuildStorage,
        commit_genesis_state: bool,
        backend: Arc<B>,
        executor: E,
    ) -> sp_blockchain::Result<Self> {
        let genesis_storage = build_genesis_storage.build_storage().map_err(sp_blockchain::Error::Storage)?;
        Ok(Self { genesis_storage, commit_genesis_state, backend, executor, _phantom: PhantomData::<Block> })
    }
}

impl<Block: BlockT, B: Backend<Block>, E: RuntimeVersionOf> BuildGenesisBlock<Block>
    for MadaraGenesisBlockBuilder<Block, B, E>
{
    type BlockImportOperation = <B as Backend<Block>>::BlockImportOperation;

    fn build_genesis_block(self) -> sp_blockchain::Result<(Block, Self::BlockImportOperation)> {
        let Self { genesis_storage, commit_genesis_state, backend, executor, _phantom } = self;

        let genesis_state_version = resolve_state_version_from_wasm(&genesis_storage, &executor)?;
        let mut op = backend.begin_operation()?;
        let state_root = op.set_genesis_state(genesis_storage, commit_genesis_state, genesis_state_version)?;
        let genesis_block = construct_genesis_block::<Block>(state_root, genesis_state_version);

        Ok((genesis_block, op))
    }
}

/// Construct genesis block.
fn construct_genesis_block<Block: BlockT>(state_root: Block::Hash, state_version: StateVersion) -> Block {
    let extrinsics_root =
        <<<Block as BlockT>::Header as HeaderT>::Hashing as HashT>::trie_root(Vec::new(), state_version);

    let mut digest = vec![];
    let block = StarknetBlock::default();

    digest.push(DigestItem::Consensus(MADARA_ENGINE_ID, Log::Block(block).encode()));

    Block::new(
        <<Block as BlockT>::Header as HeaderT>::new(
            Zero::zero(),
            extrinsics_root,
            state_root,
            Default::default(),
            Digest { logs: digest },
        ),
        Default::default(),
    )
}
