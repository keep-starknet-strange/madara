use alloc::sync::Arc;
use alloc::{format, vec};

use blockifier::block_context::BlockContext;
use blockifier::execution::entry_point::{CallEntryPoint, CallInfo, CallType, ExecutionContext, ExecutionResources};
use blockifier::state::state_api::State;
use blockifier::transaction::objects::AccountTransactionContext;
use frame_support::BoundedVec;
use sp_core::{ConstU32, H256, U256};
use starknet_api::api_core::{ClassHash, ContractAddress, EntryPointSelector};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::Calldata;

use super::entrypoint_wrapper::{
    EntryPointExecutionErrorWrapper, EntryPointExecutionResultWrapper, EntryPointTypeWrapper,
};
use super::types::{ClassHashWrapper, ContractAddressWrapper};
use crate::block::serialize::SerializeBlockContext;
use crate::block::Block as StarknetBlock;

/// Max number of calldata / tx.
pub type MaxCalldataSize = ConstU32<{ u32::MAX }>;

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
    pub calldata: BoundedVec<U256, MaxCalldataSize>,
    /// The storage address
    pub storage_address: ContractAddressWrapper,
    /// The caller address
    pub caller_address: ContractAddressWrapper,
}
// Regular implementation.
impl CallEntryPointWrapper {
    /// Creates a new instance of a call entrypoint.
    pub fn new(
        class_hash: Option<ClassHashWrapper>,
        entrypoint_type: EntryPointTypeWrapper,
        entrypoint_selector: Option<H256>,
        calldata: BoundedVec<U256, MaxCalldataSize>,
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
            // I have no idea what I'm doing
            // starknet-lib is constantly breaking it's api
            // I hope it's nothing important ¯\_(ツ)_/¯
            code_address: None,
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
    pub fn execute<S: State>(
        &self,
        state: &mut S,
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

// Traits implementation.
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
