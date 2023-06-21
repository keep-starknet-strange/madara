use alloc::format;
use alloc::sync::Arc;

use blockifier::block_context::BlockContext;
use blockifier::execution::entry_point::{
    CallEntryPoint, CallInfo, CallType, EntryPointExecutionContext, ExecutionResources,
};
use blockifier::state::state_api::State;
use blockifier::transaction::objects::AccountTransactionContext;
use cairo_vm::felt::Felt252;
use frame_support::BoundedVec;
use sp_core::ConstU32;
use starknet_api::api_core::{ClassHash, ContractAddress, EntryPointSelector};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::Calldata;
use starknet_api::StarknetApiError;

use super::entrypoint_wrapper::{
    EntryPointExecutionErrorWrapper, EntryPointExecutionResultWrapper, EntryPointTypeWrapper,
};
use super::types::{ClassHashWrapper, ContractAddressWrapper, Felt252Wrapper};

/// Max number of calldata / tx.
#[cfg(not(test))]
pub type MaxCalldataSize = ConstU32<{ u32::MAX }>;

#[cfg(test)]
pub type MaxCalldataSize = ConstU32<100>;

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
    pub entrypoint_selector: Option<Felt252Wrapper>,
    /// The Calldata
    pub calldata: BoundedVec<Felt252Wrapper, MaxCalldataSize>,
    /// The storage address
    pub storage_address: ContractAddressWrapper,
    /// The caller address
    pub caller_address: ContractAddressWrapper,
    /// The initial gas
    pub initial_gas: Felt252Wrapper,
}
// Regular implementation.
impl CallEntryPointWrapper {
    /// Creates a new instance of a call entrypoint.
    pub fn new(
        class_hash: Option<ClassHashWrapper>,
        entrypoint_type: EntryPointTypeWrapper,
        entrypoint_selector: Option<Felt252Wrapper>,
        calldata: BoundedVec<Felt252Wrapper, MaxCalldataSize>,
        storage_address: ContractAddressWrapper,
        caller_address: ContractAddressWrapper,
        initial_gas: Felt252Wrapper,
    ) -> Self {
        Self {
            class_hash,
            entrypoint_type,
            entrypoint_selector,
            calldata,
            storage_address,
            caller_address,
            initial_gas,
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
        block_context: BlockContext,
    ) -> EntryPointExecutionResultWrapper<CallInfo> {
        let call_entry_point: CallEntryPoint =
            self.clone().try_into().map_err(EntryPointExecutionErrorWrapper::StarknetApi)?;

        let execution_resources = &mut ExecutionResources::default();
        let account_context = AccountTransactionContext::default();
        let max_steps = block_context.invoke_tx_max_n_steps;
        let context = &mut EntryPointExecutionContext::new(block_context, account_context, max_steps);

        call_entry_point
            .execute(state, execution_resources, context)
            .map_err(EntryPointExecutionErrorWrapper::EntryPointExecution)
    }
}

// Traits implementation.
impl Default for CallEntryPointWrapper {
    fn default() -> Self {
        Self {
            class_hash: None,
            entrypoint_type: EntryPointTypeWrapper::External,
            entrypoint_selector: Some(Felt252Wrapper::default()),
            calldata: BoundedVec::default(),
            storage_address: ContractAddressWrapper::default(),
            caller_address: ContractAddressWrapper::default(),
            initial_gas: Felt252Wrapper::default(),
        }
    }
}

impl TryInto<CallEntryPoint> for CallEntryPointWrapper {
    type Error = StarknetApiError;

    fn try_into(self) -> Result<CallEntryPoint, Self::Error> {
        let class_hash = if let Some(class_hash) = self.class_hash {
            Some(ClassHash(StarkFelt::new(class_hash.into())?))
        } else {
            None
        };

        let entrypoint = CallEntryPoint {
            class_hash,
            entry_point_type: self.entrypoint_type.clone().into(),
            entry_point_selector: EntryPointSelector(StarkFelt::new(
                self.entrypoint_selector.unwrap_or_default().into(),
            )?),
            calldata: Calldata(Arc::new(
                self.calldata
                    .clone()
                    .into_inner()
                    .iter()
                    .map(|x| StarkFelt::try_from(format!("0x{:X}", x.0).as_str()).unwrap())
                    .collect(),
            )),
            storage_address: ContractAddress::try_from(StarkFelt::new(self.storage_address.into())?)?,
            caller_address: ContractAddress::try_from(StarkFelt::new(self.caller_address.into())?)?,
            call_type: CallType::Call,
            // I have no idea what I'm doing
            // starknet-lib is constantly breaking it's api
            // I hope it's nothing important ¯\_(ツ)_/¯
            code_address: None,
            initial_gas: Felt252::from_bytes_be(&self.initial_gas.0.to_bytes_be()),
        };

        Ok(entrypoint)
    }
}
