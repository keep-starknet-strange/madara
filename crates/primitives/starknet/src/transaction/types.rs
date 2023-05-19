use alloc::collections::BTreeMap;
use alloc::string::String;

use blockifier::execution::entry_point::CallInfo;
use blockifier::execution::errors::EntryPointExecutionError;
use blockifier::state::errors::StateError;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::transaction_types::TransactionType;
use frame_support::BoundedVec;
use sp_core::{ConstU32, H256, U256};
use starknet_api::transaction::Fee;
use starknet_api::StarknetApiError;

use crate::crypto::commitment::{
    calculate_declare_tx_hash, calculate_deploy_account_tx_hash, calculate_invoke_tx_hash,
};
use crate::execution::call_entrypoint_wrapper::MaxCalldataSize;
use crate::execution::entrypoint_wrapper::EntryPointTypeWrapper;
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
    /// State error.
    StateError(StateError),
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

impl From<TransactionValidationErrorWrapper> for TransactionExecutionErrorWrapper {
    fn from(error: TransactionValidationErrorWrapper) -> Self {
        match error {
            TransactionValidationErrorWrapper::TransactionValidationError(e) => Self::TransactionExecution(e),
            TransactionValidationErrorWrapper::CalldataError(e) => Self::StarknetApi(e),
        }
    }
}

/// Wrapper type for transaction validation result.
pub type TransactionValidationResultWrapper<T> = Result<T, TransactionValidationErrorWrapper>;

/// Wrapper type for transaction validation error.
#[derive(Debug)]
pub enum TransactionValidationErrorWrapper {
    /// Transaction execution error
    TransactionValidationError(TransactionExecutionError),
    /// Calldata error
    CalldataError(StarknetApiError),
}

impl From<EntryPointExecutionError> for TransactionValidationErrorWrapper {
    fn from(error: EntryPointExecutionError) -> Self {
        Self::TransactionValidationError(TransactionExecutionError::from(error))
    }
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

/// Declare transaction.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct DeclareTransaction {
    /// Transaction version.
    pub version: u8,
    /// Transaction sender address.
    pub sender_address: ContractAddressWrapper,
    /// Class hash to declare.
    pub compiled_class_hash: [u8; 32],
    /// Contract to declare.
    pub contract_class: ContractClassWrapper,
    /// Account contract nonce.
    pub nonce: U256,
    /// Transaction signature.
    pub signature: BoundedVec<H256, MaxArraySize>,
    /// Max fee.
    pub max_fee: U256,
}

/// Deploy account transaction.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct DeployAccountTransaction {
    /// Transaction version.
    pub version: u8,
    /// Transaction sender address.
    pub sender_address: ContractAddressWrapper,
    /// Transaction calldata.
    pub calldata: BoundedVec<U256, MaxCalldataSize>,
    /// Account contract nonce.
    pub nonce: U256,
    /// Transaction salt.
    pub salt: U256,
    /// Transaction signature.
    pub signature: BoundedVec<H256, MaxArraySize>,
    /// Account class hash.
    pub account_class_hash: [u8; 32],
    /// Max fee.
    pub max_fee: U256,
}

/// Error of conversion between [DeclareTransaction], [InvokeTransaction],
/// [DeployAccountTransaction] and [Transaction].
#[derive(Debug)]
pub enum TransactionConversionError {
    /// Class hash is missing from the object of type [Transaction]
    MissingClassHash,
    /// Class is missing from the object of type [Transaction]
    MissingClass,
}
impl TryFrom<Transaction> for DeclareTransaction {
    type Error = TransactionConversionError;
    fn try_from(value: Transaction) -> Result<Self, Self::Error> {
        Ok(Self {
            version: value.version,
            signature: value.signature,
            sender_address: value.sender_address,
            nonce: value.nonce,
            contract_class: value.contract_class.ok_or(TransactionConversionError::MissingClass)?,
            compiled_class_hash: value
                .call_entrypoint
                .class_hash
                .ok_or(TransactionConversionError::MissingClassHash)?,
            max_fee: value.max_fee,
        })
    }
}

/// Invoke transaction.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct InvokeTransaction {
    /// Transaction version.
    pub version: u8,
    /// Transaction sender address.
    pub sender_address: ContractAddressWrapper,
    /// Transaction calldata.
    pub calldata: BoundedVec<U256, MaxCalldataSize>,
    /// Account contract nonce.
    pub nonce: U256,
    /// Transaction signature.
    pub signature: BoundedVec<H256, MaxArraySize>,
    /// Max fee.
    pub max_fee: U256,
}

impl From<Transaction> for InvokeTransaction {
    fn from(value: Transaction) -> Self {
        Self {
            version: value.version,
            signature: value.signature,
            sender_address: value.sender_address,
            nonce: value.nonce,
            calldata: value.call_entrypoint.calldata,
            max_fee: value.max_fee,
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
    pub contract_address_salt: Option<U256>,
    /// Max fee.
    pub max_fee: U256,
}

impl TryFrom<Transaction> for DeployAccountTransaction {
    type Error = TransactionConversionError;
    fn try_from(value: Transaction) -> Result<Self, Self::Error> {
        Ok(Self {
            version: value.version,
            signature: value.signature,
            sender_address: value.sender_address,
            nonce: value.nonce,
            calldata: value.call_entrypoint.calldata,
            salt: value.contract_address_salt.unwrap_or_default(),
            account_class_hash: value.call_entrypoint.class_hash.ok_or(TransactionConversionError::MissingClassHash)?,
            max_fee: value.max_fee,
        })
    }
}

impl From<InvokeTransaction> for Transaction {
    fn from(value: InvokeTransaction) -> Self {
        Self {
            version: value.version,
            hash: calculate_invoke_tx_hash(value.clone()),
            signature: value.signature,
            sender_address: value.sender_address,
            nonce: value.nonce,
            call_entrypoint: CallEntryPointWrapper::new(
                None,
                EntryPointTypeWrapper::External,
                None,
                value.calldata,
                value.sender_address,
                value.sender_address,
            ),
            contract_class: None,
            contract_address_salt: None,
            max_fee: value.max_fee,
        }
    }
}
impl From<DeclareTransaction> for Transaction {
    fn from(value: DeclareTransaction) -> Self {
        Self {
            version: value.version,
            hash: calculate_declare_tx_hash(value.clone()),
            signature: value.signature,
            sender_address: value.sender_address,
            nonce: value.nonce,
            call_entrypoint: CallEntryPointWrapper::new(
                Some(value.compiled_class_hash),
                EntryPointTypeWrapper::External,
                None,
                BoundedVec::default(),
                value.sender_address,
                value.sender_address,
            ),
            contract_class: Some(value.contract_class),
            contract_address_salt: None,
            max_fee: value.max_fee,
        }
    }
}

impl From<DeployAccountTransaction> for Transaction {
    fn from(value: DeployAccountTransaction) -> Self {
        Self {
            version: value.version,
            hash: calculate_deploy_account_tx_hash(value.clone()),
            signature: value.signature,
            sender_address: value.sender_address,
            nonce: value.nonce,
            call_entrypoint: CallEntryPointWrapper::new(
                Some(value.account_class_hash),
                EntryPointTypeWrapper::External,
                None,
                value.calldata,
                value.sender_address,
                value.sender_address,
            ),
            contract_class: None,
            contract_address_salt: Some(value.salt),
            max_fee: value.max_fee,
        }
    }
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
    /// Block Number
    pub block_number: u64,
    /// Block Hash
    pub block_hash: U256,
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
