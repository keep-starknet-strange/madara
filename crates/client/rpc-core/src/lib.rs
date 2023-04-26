//! Starknet RPC API trait and types
//!
//! Starkware maintains [a description of the Starknet API](https://github.com/starkware-libs/starknet-specs/blob/master/api/starknet_api_openrpc.json)
//! using the openRPC specification.
//! This crate uses `jsonrpsee` to define such an API in Rust terms.

#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, HashMap};

use anyhow::Result;
use base64::engine::general_purpose;
use base64::Engine;
use frame_support::storage::bounded_vec::BoundedVec;
use hex::ToHex;
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use mp_starknet::execution::{ContractClassWrapper, EntryPointTypeWrapper, EntryPointWrapper, MaxEntryPoints};
use serde::{Deserialize, Serialize};
use starknet_api::deprecated_contract_class::{EntryPoint, EntryPointType};

pub type FieldElement = String;
pub type BlockNumber = u64;
pub type BlockHash = FieldElement;

pub type ContractAddress = FieldElement;

/// A tag specifying a dynamic reference to a block
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum BlockTag {
    /// The latest accepted block
    #[serde(rename = "latest")]
    Latest,
    /// The current pending block
    #[serde(rename = "pending")]
    Pending,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct BlockHashAndNumber {
    pub block_hash: FieldElement,
    pub block_number: BlockNumber,
}

/// Function call information
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct FunctionCall {
    pub contract_address: FieldElement,
    pub entry_point_selector: FieldElement,
    pub calldata: Vec<FieldElement>,
}

// In order to mix tagged and untagged {de}serialization for BlockId (see starknet RPC standard)
// in the same object, we need this kind of workaround with intermediate types
mod block_id {
    use super::*;

    #[derive(Serialize, Deserialize, Clone)]
    enum BlockIdTaggedVariants {
        #[serde(rename = "block_hash")]
        BlockHash(BlockHash),
        #[serde(rename = "block_number")]
        BlockNumber(BlockNumber),
    }

    #[derive(Serialize, Deserialize, Clone)]
    #[serde(untagged)]
    enum BlockIdUntagged {
        Tagged(BlockIdTaggedVariants),
        BlockTag(BlockTag),
    }

    /// A block hash, number or tag
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    #[serde(from = "BlockIdUntagged")]
    #[serde(into = "BlockIdUntagged")]
    pub enum BlockId {
        BlockHash(BlockHash),
        BlockNumber(BlockNumber),
        BlockTag(BlockTag),
    }

    impl From<BlockIdUntagged> for BlockId {
        fn from(value: BlockIdUntagged) -> Self {
            match value {
                BlockIdUntagged::Tagged(v) => match v {
                    BlockIdTaggedVariants::BlockHash(h) => Self::BlockHash(h),
                    BlockIdTaggedVariants::BlockNumber(n) => Self::BlockNumber(n),
                },
                BlockIdUntagged::BlockTag(t) => Self::BlockTag(t),
            }
        }
    }

    impl From<BlockId> for BlockIdUntagged {
        fn from(value: BlockId) -> Self {
            match value {
                BlockId::BlockHash(h) => Self::Tagged(BlockIdTaggedVariants::BlockHash(h)),
                BlockId::BlockNumber(n) => Self::Tagged(BlockIdTaggedVariants::BlockNumber(n)),
                BlockId::BlockTag(t) => Self::BlockTag(t),
            }
        }
    }
}

pub type Program = String;

pub type Offset = String;
pub type Selector = FieldElement;
/// Deprecated Cairo entry point (pre Sierra)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct DeprecatedCairoEntryPoint {
    pub offset: Offset,
    pub selector: Selector,
}

pub type DeprecatedConstructor = Vec<DeprecatedCairoEntryPoint>;
pub type DeprecatedExternal = Vec<DeprecatedCairoEntryPoint>;
pub type DeprecatedL1Handler = Vec<DeprecatedCairoEntryPoint>;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct DeprecatedEntryPointsByType {
    #[serde(rename = "CONSTRUCTOR")]
    pub constructor: DeprecatedConstructor,
    #[serde(rename = "EXTERNAL")]
    pub external: DeprecatedExternal,
    #[serde(rename = "L1_HANDLER")]
    pub l_1_handler: DeprecatedL1Handler,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Default)]
pub enum FunctionABIType {
    #[serde(rename = "function")]
    #[default]
    Function,
    #[serde(rename = "l1_handler")]
    LOneHandler,
    #[serde(rename = "constructor")]
    Constructor,
}

pub type FunctionName = String;
pub type ParameterName = String;
pub type ParameterType = String;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct TypedParameter {
    pub name: ParameterName,
    #[serde(rename = "type")]
    pub _type: ParameterType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct FunctionABIEntry {
    #[serde(rename = "type")]
    pub _type: FunctionABIType,
    pub name: FunctionName,
    pub inputs: TypedParameter,
    pub outputs: TypedParameter,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Default)]
pub enum EventABIType {
    #[serde(rename = "event")]
    #[default]
    Event,
}

pub type EventName = String;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct EventABIEntry {
    #[serde(rename = "type")]
    pub _type: EventABIType,
    pub name: EventName,
    pub keys: TypedParameter,
    pub data: TypedParameter,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Default)]
pub enum StructABIType {
    #[serde(rename = "struct")]
    #[default]
    Struct,
}

pub type StructName = String;
pub type Size = i64;
pub type StructMember = HashMap<String, serde_json::Value>;
pub type Members = Vec<StructMember>;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct StructABIEntry {
    #[serde(rename = "type")]
    pub _type: StructABIType,
    pub name: StructName,
    pub size: Size,
    pub members: Members,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum ContractABIEntry {
    FunctionABIEntry(FunctionABIEntry),
    EventABIEntry(EventABIEntry),
    StructABIEntry(StructABIEntry),
}

pub type ContractABI = Vec<ContractABIEntry>;
/// Deprecated Cairo contract class (pre Sierra)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct DeprecatedContractClass {
    pub program: Program,
    pub entry_points_by_type: DeprecatedEntryPointsByType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abi: Option<ContractABI>,
}

pub type SierraProgram = Vec<FieldElement>;
pub type ContractClassVersion = String;

pub type FunctionIndex = i64;
/// Cairo sierra entry point
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct SierraEntryPoint {
    pub selector: Selector,
    pub function_idx: FunctionIndex,
}

pub type Constructor = Vec<SierraEntryPoint>;
pub type External = Vec<SierraEntryPoint>;
pub type L1Handler = Vec<SierraEntryPoint>;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct EntryPointsByType {
    #[serde(rename = "CONSTRUCTOR")]
    pub constructor: Constructor,
    #[serde(rename = "EXTERNAL")]
    pub external: External,
    #[serde(rename = "L1_HANDLER")]
    pub l_1_handler: L1Handler,
}
pub type ABI = String;
/// Cairo sierra contract class
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct ContractClass {
    pub sierra_program: SierraProgram,
    pub contract_class_version: ContractClassVersion,
    pub entry_points_by_type: EntryPointsByType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abi: Option<ABI>,
}

/// Starknet contract class
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum RPCContractClass {
    DeprecatedContractClass(DeprecatedContractClass),
    ContractClass(ContractClass),
}

/// Returns a `ContractClassWrapper` from a `RPCContractClass`
pub fn to_rpc_contract_class(contract_class_wrapped: ContractClassWrapper) -> Result<RPCContractClass> {
    Ok(RPCContractClass::DeprecatedContractClass(DeprecatedContractClass {
        program: compress_and_encode_base64(&contract_class_wrapped.program)?,
        entry_points_by_type: to_deprecated_entrypoint_by_type(
            contract_class_wrapped.entry_points_by_type.into_inner(),
        ),
        abi: None,
    }))
}

/// Returns a base64 encoded and compressed string of the input bytes
fn compress_and_encode_base64(data: &[u8]) -> Result<String> {
    let data_compressed = compress(data)?;
    Ok(encode_base64(&data_compressed))
}

/// Returns a compressed vector of bytes
fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut gzip_encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    serde_json::to_writer(&mut gzip_encoder, data)?;
    Ok(gzip_encoder.finish()?)
}

/// Returns a base64 encoded string of the input bytes
fn encode_base64(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
}

/// Returns a deprecated entry point by type from hash map of entry point types to entrypoint
fn to_deprecated_entrypoint_by_type(
    entries: BTreeMap<EntryPointTypeWrapper, BoundedVec<EntryPointWrapper, MaxEntryPoints>>,
) -> DeprecatedEntryPointsByType {
    let mut constructor = vec![];
    let mut external = vec![];
    let mut l_1_handler = vec![];
    entries.into_iter().for_each(|(entry_type, entry_points)| match entry_type {
        EntryPointTypeWrapper::Constructor => {
            constructor = entry_points.into_iter().map(Into::into).collect();
        }
        EntryPointTypeWrapper::External => {
            external = entry_points.into_iter().map(Into::into).collect();
        }
        EntryPointTypeWrapper::L1Handler => {
            l_1_handler = entry_points.into_iter().map(Into::into).collect();
        }
    });
    DeprecatedEntryPointsByType { constructor, external, l_1_handler }
}

impl From<EntryPointWrapper> for DeprecatedCairoEntryPoint {
    fn from(value: EntryPointWrapper) -> Self {
        let selector: String = value.entrypoint_selector.as_fixed_bytes().encode_hex();
        let selector = add_prefix(&selector);
        let offset: String = value.entrypoint_offset.to_be_bytes().as_slice().encode_hex();
        let offset = add_prefix(remove_leading_zeros(&offset));
        Self { selector, offset }
    }
}

/// Add 0x prefix to hex string
fn add_prefix(s: &str) -> String {
    format!("0x{}", s)
}

/// Remove leading zeros from str
fn remove_leading_zeros(s: &str) -> &str {
    s.trim_start_matches('0')
}

pub use block_id::BlockId;

/// Starknet rpc interface.
#[rpc(server, namespace = "starknet")]
pub trait StarknetRpcApi {
    /// Get the most recent accepted block number
    #[method(name = "blockNumber")]
    fn block_number(&self) -> RpcResult<BlockNumber>;

    /// Get the most recent accepted block hash and number
    #[method(name = "blockHashAndNumber")]
    fn block_hash_and_number(&self) -> RpcResult<BlockHashAndNumber>;

    /// Get the number of transactions in a block given a block id
    #[method(name = "getBlockTransactionCount")]
    fn get_block_transaction_count(&self, block_id: BlockId) -> RpcResult<u128>;

    /// Call a contract function at a given block id
    #[method(name = "call")]
    fn call(&self, request: FunctionCall, block_id: BlockId) -> RpcResult<Vec<String>>;

    /// Get the contract class at a given contract address for a given block id
    #[method(name = "getClassAt")]
    fn get_class_at(&self, contract_address: ContractAddress, block_id: BlockId) -> RpcResult<RPCContractClass>;
}
