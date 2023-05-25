use alloc::collections::BTreeMap;
use alloc::string::String;

use blockifier::execution::entry_point::CallInfo;
use blockifier::execution::errors::EntryPointExecutionError;
use blockifier::state::errors::StateError;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::transaction_types::TransactionType;
use frame_support::BoundedVec;
use sp_core::ConstU32;
use starknet_api::transaction::Fee;
use starknet_api::StarknetApiError;

use crate::execution::call_entrypoint_wrapper::{CallEntryPointWrapper, MaxCalldataSize};
use crate::execution::entrypoint_wrapper::EntryPointTypeWrapper;
use crate::execution::types::{ContractAddressWrapper, ContractClassWrapper, Felt252Wrapper};

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

/// Error of conversion between [DeclareTransaction], [InvokeTransaction],
/// [DeployAccountTransaction] and [Transaction].
#[derive(Debug)]
pub enum TransactionConversionError {
    /// Class hash is missing from the object of type [Transaction]
    MissingClassHash,
    /// Class is missing from the object of type [Transaction]
    MissingClass,
}

/// Invoke transaction.
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
pub enum InvokeTransaction {
    V0(InvokeTransactionV0),
    V1(InvokeTransactionV1),
}

// From https://github.com/tdelabro/starknet-api/blob/main/src/transaction.rs
macro_rules! implement_invoke_tx_getters {
    ($(($field:ident, $field_type:ty)),*) => {
        $(pub fn $field(&self) -> $field_type {
            match self {
                Self::V0(tx) => tx.$field.clone(),
                Self::V1(tx) => tx.$field.clone(),
            }
        })*
    };
}

impl InvokeTransaction {
    implement_invoke_tx_getters!(
        (transaction_hash, Felt252Wrapper),
        (nonce, Felt252Wrapper),
        (max_fee, Felt252Wrapper),
        (signature, BoundedVec<Felt252Wrapper, MaxArraySize>),
        (calldata, BoundedVec<Felt252Wrapper, MaxCalldataSize>)
    );

    pub fn version(&self) -> Felt252Wrapper {
        match self {
            InvokeTransaction::V0(_) => Felt252Wrapper::from(0u8),
            InvokeTransaction::V1(_) => Felt252Wrapper::from(1u8),
        }
    }

    pub fn sender_address(&self) -> Felt252Wrapper {
        match self {
            InvokeTransaction::V0(tx) => tx.contract_address,
            InvokeTransaction::V1(tx) => tx.sender_address,
        }
    }
}

/// Invoke transaction v0.
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
pub struct InvokeTransactionV0 {
    /// The hash identifying the transaction
    pub transaction_hash: Felt252Wrapper,
    /// The maximal fee that can be charged for including the transaction
    pub max_fee: Felt252Wrapper,
    pub signature: BoundedVec<Felt252Wrapper, MaxArraySize>,
    pub nonce: Felt252Wrapper,
    pub contract_address: Felt252Wrapper,
    pub entry_point_selector: Felt252Wrapper,
    /// The parameters passed to the function
    pub calldata: BoundedVec<Felt252Wrapper, MaxCalldataSize>,
}

/// Invoke transaction v1.
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
pub struct InvokeTransactionV1 {
    /// The hash identifying the transaction
    pub transaction_hash: Felt252Wrapper,
    /// The maximal fee that can be charged for including the transaction
    pub max_fee: Felt252Wrapper,
    /// Signature
    pub signature: BoundedVec<Felt252Wrapper, MaxArraySize>,
    /// Nonce
    pub nonce: Felt252Wrapper,
    /// Sender address
    pub sender_address: Felt252Wrapper,
    /// The data expected by the account's `execute` function (in most usecases, this includes the
    /// called contract address and a function selector)
    pub calldata: BoundedVec<Felt252Wrapper, MaxCalldataSize>,
}

/// L1 handler transaction.
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
pub struct L1HandlerTransaction {
    /// The hash identifying the transaction
    pub transaction_hash: Felt252Wrapper,
    /// Version of the transaction scheme
    pub version: u64,
    /// The L1->L2 message nonce field of the sn core L1 contract at the time the transaction was
    /// sent
    pub nonce: u64,
    pub contract_address: Felt252Wrapper,
    pub entry_point_selector: Felt252Wrapper,
    /// The parameters passed to the function
    pub calldata: BoundedVec<Felt252Wrapper, MaxCalldataSize>,
}

/// Declare transaction.
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
pub enum DeclareTransaction {
    V1(DeclareTransactionV1),
    V2(DeclareTransactionV2),
}

// From https://github.com/tdelabro/starknet-api/blob/main/src/transaction.rs
macro_rules! implement_declare_tx_getters {
    ($(($field:ident, $field_type:ty)),*) => {
        $(pub fn $field(&self) -> $field_type {
            match self {
                Self::V1(tx) => tx.$field.clone(),
                Self::V2(tx) => tx.$field.clone(),
            }
        })*
    };
}

impl DeclareTransaction {
    implement_declare_tx_getters!(
        (transaction_hash, Felt252Wrapper),
        (class_hash, Felt252Wrapper),
        (nonce, Felt252Wrapper),
        (sender_address, Felt252Wrapper),
        (max_fee, Felt252Wrapper),
        (signature, BoundedVec<Felt252Wrapper, MaxArraySize>)
    );

    pub fn version(&self) -> Felt252Wrapper {
        match self {
            DeclareTransaction::V1(_) => Felt252Wrapper::from(1u8),
            DeclareTransaction::V2(_) => Felt252Wrapper::from(2u8),
        }
    }
}

/// Declare contract transaction v1.
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
pub struct DeclareTransactionV1 {
    /// The hash identifying the transaction
    pub transaction_hash: Felt252Wrapper,
    /// The maximal fee that can be charged for including the transaction
    pub max_fee: Felt252Wrapper,
    pub signature: BoundedVec<Felt252Wrapper, MaxArraySize>,
    pub nonce: Felt252Wrapper,
    /// The contract class
    pub contract_class: ContractClassWrapper,
    /// The hash of the declared class
    pub class_hash: Felt252Wrapper,
    /// The address of the  contract sending the declaration transaction
    pub sender_address: Felt252Wrapper,
}

/// Declare transaction v2.
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
pub struct DeclareTransactionV2 {
    /// The hash identifying the transaction
    pub transaction_hash: Felt252Wrapper,
    /// The maximal fee that can be charged for including the transaction
    pub max_fee: Felt252Wrapper,
    pub signature: BoundedVec<Felt252Wrapper, MaxArraySize>,
    pub nonce: Felt252Wrapper,
    /// The contract class
    pub contract_class: ContractClassWrapper,
    /// The hash of the declared sierra class
    pub class_hash: Felt252Wrapper,
    /// The hash of the compiled
    pub compiled_class_hash: Felt252Wrapper,
    /// The address of the account contract sending the declaration transaction
    pub sender_address: Felt252Wrapper,
}

/// Deploy account transaction.
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
pub struct DeployAccountTransaction {
    /// The hash identifying the transaction
    pub transaction_hash: Felt252Wrapper,
    /// The maximal fee that can be charged for including the transaction
    pub max_fee: Felt252Wrapper,
    pub signature: BoundedVec<Felt252Wrapper, MaxArraySize>,
    pub nonce: Felt252Wrapper,
    /// The address of the account contract being deployed
    pub sender_address: Felt252Wrapper,
    /// The salt for the address of the deployed contract
    pub contract_address_salt: Felt252Wrapper,
    /// The parameters passed to the constructor
    pub constructor_calldata: BoundedVec<Felt252Wrapper, MaxCalldataSize>,
    /// The hash of the deployed contract's class
    pub class_hash: Felt252Wrapper,
}

impl DeployAccountTransaction {
    pub fn version(&self) -> Felt252Wrapper {
        Felt252Wrapper::from(1u8)
    }

    pub fn call_entrypoint(&self) -> CallEntryPointWrapper {
        CallEntryPointWrapper::new(
            Some(self.class_hash),
            EntryPointTypeWrapper::External,
            None,
            self.constructor_calldata,
            self.sender_address,
            self.sender_address,
        )
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
pub enum Transaction {
    #[serde(rename = "INVOKE")]
    Invoke(InvokeTransaction),
    #[serde(rename = "L1_HANDLER")]
    L1Handler(L1HandlerTransaction),
    #[serde(rename = "DECLARE")]
    Declare(DeclareTransaction),
    #[serde(rename = "DEPLOY_ACCOUNT")]
    DeployAccount(DeployAccountTransaction),
}

/// Returns the information for a Transaction. Missing fields are set to Default
pub struct TransactionInfo {
    pub transaction_hash: Felt252Wrapper,
    pub max_fee: Felt252Wrapper,
    pub signature: BoundedVec<Felt252Wrapper, MaxArraySize>,
    pub nonce: Felt252Wrapper,
    pub version: u64,
    pub contract_address: Felt252Wrapper,
    pub entry_point_selector: Felt252Wrapper,
    pub calldata: BoundedVec<Felt252Wrapper, MaxCalldataSize>,
    pub class_hash: Felt252Wrapper,
}

/// Error enum for transactions.
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
pub enum TransactionError {
    /// Missing input for transaction.
    MissingInput,
    /// Invalid data
    InvalidData,
    /// Invalid transaction version
    InvalidVersion,
}

impl Transaction {}

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
    pub transaction_hash: Felt252Wrapper,
    /// Fee paid for the transaction.
    pub actual_fee: Felt252Wrapper,
    /// Transaction type
    pub tx_type: TxType,
    /// Block Number
    pub block_number: u64,
    /// Block Hash
    pub block_hash: Felt252Wrapper,
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
    pub keys: BoundedVec<Felt252Wrapper, MaxArraySize>,
    /// The data of the event.
    pub data: BoundedVec<Felt252Wrapper, MaxArraySize>,
    /// The address that emitted the event
    pub from_address: ContractAddressWrapper,
}

/// This struct wraps the \[TransactionExecutionInfo\] type from the blockifier.
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
