// SPDX-License-Identifier: Apache-2.0
// This file is part of Frontier.
//
// Copyright (c) 2020-2022 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::large_enum_variant)]
#![deny(unused_crate_dependencies)]

use mp_starknet::block::Block as StarknetBlock;
use scale_codec::{Decode, Encode};
use sp_core::H256;
use sp_runtime::generic::{Digest, OpaqueDigestItemId};
use sp_runtime::ConsensusEngineId;

pub const MADARA_ENGINE_ID: ConsensusEngineId = [b'm', b'a', b'd', b'a'];

#[derive(Clone, PartialEq, Eq)]
pub enum Log {
    Post(PostLog),
}

#[derive(Decode, Encode, Clone, PartialEq, Eq)]
pub enum PostLog {
    /// Ethereum block hash.
    #[codec(index = 3)]
    BlockHash(H256),
}

#[derive(Decode, Encode, Clone, PartialEq, Eq)]
pub struct Hashes {
    /// Starknet block hash.
    pub block_hash: H256,
}

impl Hashes {
    pub fn from_block(block: StarknetBlock) -> Self {
        Hashes { block_hash: block.header().hash() }
    }
}

#[derive(Clone, Debug)]
pub enum FindLogError {
    NotFound,
    MultipleLogs,
}

pub fn find_post_log(digest: &Digest) -> Result<PostLog, FindLogError> {
    _find_log(digest, OpaqueDigestItemId::Consensus(&MADARA_ENGINE_ID))
}

fn _find_log<Log: Decode>(digest: &Digest, digest_item_id: OpaqueDigestItemId) -> Result<Log, FindLogError> {
    let mut found = None;

    for log in digest.logs() {
        let log = log.try_to::<Log>(digest_item_id);
        match (log, found.is_some()) {
            (Some(_), true) => return Err(FindLogError::MultipleLogs),
            (Some(log), false) => found = Some(log),
            (None, _) => (),
        }
    }

    found.ok_or(FindLogError::NotFound)
}

pub fn find_log(digest: &Digest) -> Result<Log, FindLogError> {
    let mut found = None;

    for log in digest.logs() {
        let post_log = log.try_to::<PostLog>(OpaqueDigestItemId::Consensus(&MADARA_ENGINE_ID));
        match (post_log, found.is_some()) {
            (Some(_), true) => return Err(FindLogError::MultipleLogs),
            (Some(post_log), false) => found = Some(Log::Post(post_log)),
            (None, _) => (),
        }
    }

    found.ok_or(FindLogError::NotFound)
}

pub fn ensure_log(digest: &Digest) -> Result<(), FindLogError> {
    find_log(digest).map(|_log| ())
}
