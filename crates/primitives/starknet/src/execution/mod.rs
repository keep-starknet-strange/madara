//! Starknet execution functionality.

/// Types related to entrypoints.
pub mod types;

use alloc::sync::Arc;
use alloc::{format, vec};

use blockifier::block_context::BlockContext;
use blockifier::execution::contract_class::ContractClass;
use blockifier::execution::entry_point::{CallEntryPoint, CallInfo, CallType, ExecutionContext, ExecutionResources};
use blockifier::state::cached_state::CachedState;
use blockifier::state::state_api::StateReader;
use blockifier::transaction::objects::AccountTransactionContext;
use frame_support::BoundedVec;
use serde_json::{from_slice, to_string};
use sp_core::{ConstU32, H256, U256};
use starknet_api::api_core::{ClassHash, ContractAddress, EntryPointSelector};
use starknet_api::deprecated_contract_class::{EntryPoint, EntryPointOffset, EntryPointType, Program};
use starknet_api::hash::StarkFelt;
use starknet_api::stdlib::collections::HashMap;
use starknet_api::transaction::Calldata;

use self::types::{EntryPointExecutionErrorWrapper, EntryPointExecutionResultWrapper};
use crate::block::serialize::SerializeBlockContext;
use crate::block::Block as StarknetBlock;
use crate::transaction::types::MaxArraySize;

/// The address of a contract.
pub type ContractAddressWrapper = [u8; 32];

/// Maximum vector sizes.
// type MaxCalldataSize = ConstU32<4294967295>;
// type MaxAbiSize = ConstU32<4294967295>;
type MaxProgramSize = ConstU32<4294967295>;
type MaxEntryPoints = ConstU32<4294967295>;

/// Wrapper type for class hash field.
pub type ClassHashWrapper = [u8; 32];

/// Contract Class
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct ContractClassWrapper {
    /// Contract class program json.
    pub program: BoundedVec<u8, MaxProgramSize>,
    // /// Contract class abi.
    // pub abi: BoundedVec<ContractClassAbiEntryWrapper, MaxAbiSize>,
    /// Contract class entrypoints.
    pub entry_points_by_type: BoundedVec<u8, MaxEntryPoints>,
}

impl ContractClassWrapper {
    /// Creates a new instance of a contract class.
    pub fn new(program: BoundedVec<u8, MaxProgramSize>, entry_points_by_type: BoundedVec<u8, MaxProgramSize>) -> Self {
        Self { program, entry_points_by_type }
    }

    /// Convert to starknet contract class.
    pub fn to_starknet_contract_class(&self) -> Result<ContractClass, serde_json::Error> {
        let program = from_slice::<Program>(self.program.as_ref())?;
        let entrypoints =
            from_slice::<HashMap<EntryPointType, vec::Vec<EntryPoint>>>(self.entry_points_by_type.as_ref())?;
        Ok(ContractClass { program, abi: None, entry_points_by_type: entrypoints })
    }
}

impl From<ContractClass> for ContractClassWrapper {
    fn from(contract_class: ContractClass) -> Self {
        let program_string = to_string(&contract_class.program).unwrap();
        let entrypoints_string = to_string(&contract_class.entry_points_by_type).unwrap();
        Self {
            program: BoundedVec::try_from(program_string.as_bytes().to_vec()).unwrap(),
            entry_points_by_type: BoundedVec::try_from(entrypoints_string.as_bytes().to_vec()).unwrap(),
        }
    }
}

impl Default for ContractClassWrapper {
    fn default() -> Self {
        Self {
            program: BoundedVec::try_from(vec![]).unwrap(),
            entry_points_by_type: BoundedVec::try_from(vec![]).unwrap(),
        }
    }
}

/// Enum that represents all the entrypoints types.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    PartialOrd,
    Ord,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum EntryPointTypeWrapper {
    /// Constructor.
    Constructor,
    /// External.
    External,
    /// L1 Handler.
    L1Handler,
}

impl From<EntryPointType> for EntryPointTypeWrapper {
    fn from(entry_point_type: EntryPointType) -> Self {
        match entry_point_type {
            EntryPointType::Constructor => EntryPointTypeWrapper::Constructor,
            EntryPointType::External => EntryPointTypeWrapper::External,
            EntryPointType::L1Handler => EntryPointTypeWrapper::L1Handler,
        }
    }
}

impl EntryPointTypeWrapper {
    /// Convert to starknet entrypoint type.
    pub fn to_starknet_entry_point_type(&self) -> EntryPointType {
        match self {
            EntryPointTypeWrapper::Constructor => EntryPointType::Constructor,
            EntryPointTypeWrapper::External => EntryPointType::External,
            EntryPointTypeWrapper::L1Handler => EntryPointType::L1Handler,
        }
    }
}

// pub enum ContractClassAbiEntryWrapper {
// 	/// An event abi entry.
//     Event(EventAbiEntry),
//     /// A function abi entry.
//     Function(FunctionAbiEntryWithType),
//     /// A struct abi entry.
//     Struct(StructAbiEntry),
// }

/// Representation of a Starknet Entry Point.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    PartialOrd,
    Ord,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct EntryPointWrapper {
    /// The entrypoint selector
    pub entrypoint_selector: H256,
    /// The entrypoint offset
    pub entrypoint_offset: U256,
}

impl EntryPointWrapper {
    /// Creates a new instance of an entrypoint.
    pub fn new(entrypoint_selector: H256, entrypoint_offset: U256) -> Self {
        Self { entrypoint_selector, entrypoint_offset }
    }

    /// Convert to Starknet EntryPoint
    pub fn to_starknet_entry_point(&self) -> EntryPoint {
        EntryPoint {
            selector: EntryPointSelector(StarkFelt::new(self.entrypoint_selector.0).unwrap()),
            offset: EntryPointOffset(self.entrypoint_offset.as_usize()),
        }
    }
}

impl From<EntryPoint> for EntryPointWrapper {
    fn from(entry_point: EntryPoint) -> Self {
        Self {
            entrypoint_selector: H256::from_slice(entry_point.selector.0.bytes()),
            entrypoint_offset: U256::from(entry_point.offset.0),
        }
    }
}

/// Representation of a Starknet Call Entry Point.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct CallEntryPointWrapper {
    /// The class hash
    pub class_hash: Option<ClassHashWrapper>,
    /// The entrypoint type
    pub entrypoint_type: EntryPointTypeWrapper,
    /// The entrypoint selector
    /// An invoke transaction without an entry point selector invokes the 'execute' function.
    pub entrypoint_selector: Option<H256>,
    /// The Calldata
    pub calldata: BoundedVec<U256, MaxArraySize>,
    /// The storage address
    pub storage_address: ContractAddressWrapper,
    /// The caller address
    pub caller_address: ContractAddressWrapper,
}
impl EntryPointTypeWrapper {
    /// Convert Madara entrypoint type to Starknet entrypoint type.
    pub fn to_starknet(&self) -> EntryPointType {
        match self {
            Self::Constructor => EntryPointType::Constructor,
            Self::External => EntryPointType::External,
            Self::L1Handler => EntryPointType::L1Handler,
        }
    }
}

impl CallEntryPointWrapper {
    /// Creates a new instance of a call entrypoint.
    pub fn new(
        class_hash: Option<ClassHashWrapper>,
        entrypoint_type: EntryPointTypeWrapper,
        entrypoint_selector: Option<H256>,
        calldata: BoundedVec<U256, MaxArraySize>,
        storage_address: ContractAddressWrapper,
        caller_address: ContractAddressWrapper,
    ) -> Self {
        Self { class_hash, entrypoint_type, entrypoint_selector, calldata, storage_address, caller_address }
    }

    /// Convert to Starknet CallEntryPoint
    pub fn to_starknet_call_entry_point(&self) -> CallEntryPoint {
        let class_hash = self.class_hash.map(|class_hash| ClassHash(StarkFelt::new(class_hash).unwrap()));
        CallEntryPoint {
            class_hash,
            entry_point_type: self.entrypoint_type.to_starknet(),
            entry_point_selector: EntryPointSelector(
                StarkFelt::new(self.entrypoint_selector.unwrap_or_default().0).unwrap(),
            ),
            calldata: Calldata(Arc::new(
                self.calldata
                    .clone()
                    .into_inner()
                    .iter()
                    .map(|x| StarkFelt::try_from(format!("0x{x:X}").as_str()).unwrap())
                    .collect(),
            )),
            storage_address: ContractAddress::try_from(StarkFelt::new(self.storage_address).unwrap()).unwrap(),
            caller_address: ContractAddress::try_from(StarkFelt::new(self.caller_address).unwrap()).unwrap(),
            call_type: CallType::Call,
        }
    }

    /// Executes an entry point.
    ///
    /// # Arguments
    ///
    /// * `self` - The entry point to execute.
    /// * `state` - The state to execute the entry point on.
    /// * `block` - The block to execute the entry point on.
    /// * `fee_token_address` - The fee token address.
    ///
    /// # Returns
    ///
    /// * The result of the entry point execution.
    pub fn execute<S: StateReader>(
        &self,
        state: &mut CachedState<S>,
        block: StarknetBlock,
        fee_token_address: ContractAddressWrapper,
    ) -> EntryPointExecutionResultWrapper<CallInfo> {
        let call_entry_point = self.to_starknet_call_entry_point();

        let execution_resources = &mut ExecutionResources::default();
        let execution_context = &mut ExecutionContext::default();
        let account_context = AccountTransactionContext::default();

        // Create the block context.
        let block_context = BlockContext::try_serialize(block.header().clone(), fee_token_address)
            .map_err(|_| EntryPointExecutionErrorWrapper::BlockContextSerializationError)?;

        call_entry_point
            .execute(state, execution_resources, execution_context, &block_context, &account_context)
            .map_err(EntryPointExecutionErrorWrapper::EntryPointExecution)
    }
}
impl Default for CallEntryPointWrapper {
    fn default() -> Self {
        Self {
            class_hash: Some(ClassHashWrapper::default()),
            entrypoint_type: EntryPointTypeWrapper::External,
            entrypoint_selector: Some(H256::default()),
            calldata: BoundedVec::try_from(vec![U256::zero(); 32]).unwrap(),
            storage_address: ContractAddressWrapper::default(),
            caller_address: ContractAddressWrapper::default(),
        }
    }
}
