// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
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

//! Substrate transaction pool implementation.

use sc_transaction_pool_api::ChainEvent;
use sp_blockchain::TreeRoute;
use sp_runtime::traits::{Block as BlockT, NumberFor, Saturating};

use crate::LOG_TARGET;

/// The threshold since the last update where we will skip any maintenance for blocks.
///
/// This includes tracking re-orgs and sending out certain notifications. In general this shouldn't
/// happen and may only happen when the node is doing a full sync.
const SKIP_MAINTENANCE_THRESHOLD: u16 = 20;

/// Helper struct for keeping track of the current state of processed new best
/// block and finalized events. The main purpose of keeping track of this state
/// is to figure out which phases (enactment / finalization) of transaction pool
/// maintenance are needed.
///
/// Given the following chain:
///
///   B1-C1-D1-E1
///  /
/// A
///  \
///   B2-C2-D2-E2
///
/// Some scenarios and expected behavior for sequence of `NewBestBlock` (`nbb`) and `Finalized`
/// (`f`) events:
///
/// - `nbb(C1)`, `f(C1)` -> false (enactment was already performed in `nbb(C1))`
/// - `f(C1)`, `nbb(C1)` -> false (enactment was already performed in `f(C1))`
/// - `f(C1)`, `nbb(D2)` -> false (enactment was already performed in `f(C1)`,
/// we should not retract finalized block)
/// - `f(C1)`, `f(C2)`, `nbb(C1)` -> false
/// - `nbb(C1)`, `nbb(C2)` -> true (switching fork is OK)
/// - `nbb(B1)`, `nbb(B2)` -> true
/// - `nbb(B1)`, `nbb(C1)`, `f(C1)` -> false (enactment was already performed in `nbb(B1)`)
/// - `nbb(C1)`, `f(B1)` -> false (enactment was already performed in `nbb(B2)`)
pub struct EnactmentState<Block>
where
    Block: BlockT,
{
    recent_best_block: Block::Hash,
    recent_finalized_block: Block::Hash,
}

/// Enactment action that should be performed after processing the `ChainEvent`
#[derive(Debug)]
pub enum EnactmentAction<Block: BlockT> {
    /// Both phases of maintenance shall be skipped
    Skip,
    /// Both phases of maintenance shall be performed
    HandleEnactment(TreeRoute<Block>),
    /// Enactment phase of maintenance shall be skipped
    HandleFinalization,
}

impl<Block> EnactmentState<Block>
where
    Block: BlockT,
{
    /// Returns a new `EnactmentState` initialized with the given parameters.
    pub fn new(recent_best_block: Block::Hash, recent_finalized_block: Block::Hash) -> Self {
        EnactmentState { recent_best_block, recent_finalized_block }
    }

    /// Returns the recently finalized block.
    pub fn recent_finalized_block(&self) -> Block::Hash {
        self.recent_finalized_block
    }

    /// Updates the state according to the given `ChainEvent`, returning
    /// `Some(tree_route)` with a tree route including the blocks that need to
    /// be enacted/retracted. If no enactment is needed then `None` is returned.
    pub fn update<TreeRouteF, BlockNumberF>(
        &mut self,
        event: &ChainEvent<Block>,
        tree_route: &TreeRouteF,
        hash_to_number: &BlockNumberF,
    ) -> Result<EnactmentAction<Block>, String>
    where
        TreeRouteF: Fn(Block::Hash, Block::Hash) -> Result<TreeRoute<Block>, String>,
        BlockNumberF: Fn(Block::Hash) -> Result<Option<NumberFor<Block>>, String>,
    {
        let new_hash = event.hash();
        let finalized = event.is_finalized();

        // do not proceed with txpool maintain if block distance is to high
        let skip_maintenance = match (hash_to_number(new_hash), hash_to_number(self.recent_best_block)) {
            (Ok(Some(new)), Ok(Some(current))) => new.saturating_sub(current) > SKIP_MAINTENANCE_THRESHOLD.into(),
            _ => true,
        };

        if skip_maintenance {
            log::debug!(target: LOG_TARGET, "skip maintain: tree_route would be too long");
            self.force_update(event);
            return Ok(EnactmentAction::Skip);
        }

        // block was already finalized
        if self.recent_finalized_block == new_hash {
            log::debug!(target: LOG_TARGET, "handle_enactment: block already finalized");
            return Ok(EnactmentAction::Skip);
        }

        // compute actual tree route from best_block to notified block, and use
        // it instead of tree_route provided with event
        let tree_route = tree_route(self.recent_best_block, new_hash)?;

        log::debug!(
            target: LOG_TARGET,
            "resolve hash: {new_hash:?} finalized: {finalized:?} tree_route: (common {:?}, last {:?}) best_block: \
             {:?} finalized_block:{:?}",
            tree_route.common_block(),
            tree_route.last(),
            self.recent_best_block,
            self.recent_finalized_block
        );

        // check if recently finalized block is on retracted path. this could be
        // happening if we first received a finalization event and then a new
        // best event for some old stale best head.
        if tree_route.retracted().iter().any(|x| x.hash == self.recent_finalized_block) {
            log::debug!(
                target: LOG_TARGET,
                "Recently finalized block {} would be retracted by ChainEvent {}, skipping",
                self.recent_finalized_block,
                new_hash
            );
            return Ok(EnactmentAction::Skip);
        }

        if finalized {
            self.recent_finalized_block = new_hash;

            // if there are no enacted blocks in best_block -> hash tree_route,
            // it means that block being finalized was already enacted (this
            // case also covers best_block == new_hash), recent_best_block
            // remains valid.
            if tree_route.enacted().is_empty() {
                log::trace!(target: LOG_TARGET, "handle_enactment: no newly enacted blocks since recent best block");
                return Ok(EnactmentAction::HandleFinalization);
            }

            // otherwise enacted finalized block becomes best block...
        }

        self.recent_best_block = new_hash;

        Ok(EnactmentAction::HandleEnactment(tree_route))
    }

    /// Forces update of the state according to the given `ChainEvent`. Intended to be used as a
    /// fallback when tree_route cannot be computed.
    pub fn force_update(&mut self, event: &ChainEvent<Block>) {
        match event {
            ChainEvent::NewBestBlock { hash, .. } => self.recent_best_block = *hash,
            ChainEvent::Finalized { hash, .. } => self.recent_finalized_block = *hash,
        };
        log::debug!(
            target: LOG_TARGET,
            "forced update: {:?}, {:?}",
            self.recent_best_block,
            self.recent_finalized_block,
        );
    }
}
