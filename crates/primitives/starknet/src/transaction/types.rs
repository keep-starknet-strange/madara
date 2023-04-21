use blockifier::transaction::errors::TransactionExecutionError;
use frame_support::BoundedVec;
use sp_core::{ConstU32, H256, U256};
use starknet_api::StarknetApiError;

use crate::execution::{CallEntryPointWrapper, ContractAddressWrapper, ContractClassWrapper};

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
    InvokeTx,
    /// Declare transaction.
    DeclareTx,
    /// Deploy account transaction.
    DeployAccountTx,
    /// Message sent from ethereum.
    L1HandlerTx,
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

/// Information needed to perform the fee transfer.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Default,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct FeeTransferInformation {
    /// Actual fees paid for the transaction.
    pub actual_fee: U256,
    /// Address of the person paying the fees.
    pub payer: ContractAddressWrapper,
}
impl FeeTransferInformation {
    /// Creates a new instance of FeeTransferInformation
    pub fn new(actual_fee: U256, payer: ContractAddressWrapper) -> Self {
        Self { actual_fee, payer }
    }
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
