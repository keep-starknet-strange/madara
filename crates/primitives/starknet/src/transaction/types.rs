use alloc::collections::BTreeMap;
use alloc::string::String;

use blockifier::execution::entry_point::CallInfo;
use blockifier::execution::errors::EntryPointExecutionError;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::transaction_types::TransactionType;
use frame_support::BoundedVec;
use sp_core::{ConstU32, H256, U256};
use starknet_api::transaction::Fee;
use starknet_api::StarknetApiError;

use crate::execution::types::{CallEntryPointWrapper, ContractAddressWrapper, ContractClassWrapper};

/// Max size of arrays.
/// TODO: add real value (#250)
pub type MaxArraySize = ConstU32<10000>;

/// Wrapper type for transaction execution result.
pub type TransactionExecutionResultWrapper<T> = Result<T, TransactionExecutionErrorWrapper>;

/// Wrapper type for transaction execution error.
#[derive(Debug)]
pub enum TransactionExecutionErrorWrapper {
    /// Transaction execution error.
    TransactionExecution(TransactionExecutionError),
    /// Starknet API error.
    StarknetApi(StarknetApiError),
    /// Block context serialization error.
    BlockContextSerializationError,
    /// Fee computation error,
    FeeComputationError,
    /// Fee transfer error,
    FeeTransferError {
        /// Max fee specified by the user.
        max_fee: Fee,
        /// Actual fee.
        actual_fee: Fee,
    },
    /// Cairo resources are not contained in the fee costs.
    CairoResourcesNotContainedInFeeCosts,
    /// Failed to compute the L1 gas usage.
    FailedToComputeL1GasUsage,
    /// Entrypoint execution error
    EntrypointExecution(EntryPointExecutionError),
    /// Unexpected holes.
    UnexpectedHoles(String),
}

/// Different tx types.
/// See `https://docs.starknet.io/documentation/architecture_and_concepts/Blocks/transactions/` for more details.
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
pub enum TxType {
    /// Regular invoke transaction.
    Invoke,
    /// Declare transaction.
    Declare,
    /// Deploy account transaction.
    DeployAccount,
    /// Message sent from ethereum.
    L1Handler,
}
impl From<TransactionType> for TxType {
    fn from(value: TransactionType) -> Self {
        match value {
            TransactionType::Declare => Self::Declare,
            TransactionType::DeployAccount => Self::DeployAccount,
            TransactionType::InvokeFunction => Self::Invoke,
            TransactionType::L1Handler => Self::L1Handler,
        }
    }
}
impl From<TxType> for TransactionType {
    fn from(value: TxType) -> Self {
        match value {
            TxType::Declare => Self::Declare,
            TxType::DeployAccount => Self::DeployAccount,
            TxType::Invoke => Self::InvokeFunction,
            TxType::L1Handler => Self::L1Handler,
        }
    }
}
/// Representation of a Starknet transaction.
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
pub struct Transaction {
    /// The version of the transaction.
    pub version: u8,
    /// Transaction hash.
    pub hash: H256,
    /// Signature.
    pub signature: BoundedVec<H256, MaxArraySize>,
    /// Sender Address
    pub sender_address: ContractAddressWrapper,
    /// Nonce
    pub nonce: U256,
    /// Call entrypoint
    pub call_entrypoint: CallEntryPointWrapper,
    /// Contract Class
    pub contract_class: Option<ContractClassWrapper>,
    /// Contract Address Salt
    pub contract_address_salt: Option<H256>,
}

/// Representation of a Starknet transaction receipt.
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
pub struct TransactionReceiptWrapper {
    /// Transaction hash.
    pub transaction_hash: H256,
    /// Fee paid for the transaction.
    pub actual_fee: U256,
    /// Transaction type
    pub tx_type: TxType,
    /// Messages sent in the transaction.
    // pub messages_sent: BoundedVec<Message, MaxArraySize>, // TODO: add messages
    /// Events emitted in the transaction.
    pub events: BoundedVec<EventWrapper, MaxArraySize>,
}

/// Representation of a Starknet event.
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
pub struct EventWrapper {
    /// The keys (topics) of the event.
    pub keys: BoundedVec<H256, MaxArraySize>,
    /// The data of the event.
    pub data: BoundedVec<H256, MaxArraySize>,
    /// The address that emitted the event
    pub from_address: ContractAddressWrapper,
}

/// This struct wraps the [TransactionExecutionInfo] type from the blockifier.
#[derive(Debug)]
pub struct TransactionExecutionInfoWrapper {
    /// Transaction validation call info; [None] for `L1Handler`.
    pub validate_call_info: Option<CallInfo>,
    /// Transaction execution call info; [None] for `Declare`.
    pub execute_call_info: Option<CallInfo>,
    /// Fee transfer call info; [None] for `L1Handler`.
    pub fee_transfer_call_info: Option<CallInfo>,
    /// The actual fee that was charged (in Wei).
    pub actual_fee: Fee,
    /// Actual execution resources the transaction is charged for,
    /// including L1 gas and additional OS resources estimation.
    pub actual_resources: BTreeMap<String, usize>,
}

/// Error enum wrapper for events.
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
pub enum EventError {
    /// Provided keys are invalid.
    InvalidKeys,
    /// Provided data is invalid.
    InvalidData,
    /// Provided from address is invalid.
    InvalidFromAddress,
    /// Too many events
    TooManyEvents,
}

/// Error enum wrapper for state diffs.
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
pub enum StateDiffError {
    /// Couldn't register newly deployed contracts.
    DeployedContractError,
    /// Couldn't register newly declared contracts.
    DeclaredClassError,
}
