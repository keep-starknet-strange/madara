use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use blockifier::execution::contract_class::ContractClass;
use blockifier::execution::entry_point::CallInfo;
use blockifier::execution::errors::EntryPointExecutionError;
use blockifier::state::errors::StateError;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::transaction_types::TransactionType;

use frame_support::BoundedVec;
use sp_core::ConstU32;
use starknet_api::api_core::{calculate_contract_address, ClassHash, ContractAddress};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, ContractAddressSalt, Fee};
use starknet_api::StarknetApiError;
use thiserror_no_std::Error;

use crate::crypto::commitment::{
    calculate_deploy_account_tx_hash,
};
use crate::execution::call_entrypoint_wrapper::MaxCalldataSize;
<<<<<<< HEAD

use crate::execution::types::{
    ContractAddressWrapper, ContractClassWrapper, Felt252Wrapper,
};

/// Max size of arrays.
/// TODO: add real value (#250)
#[cfg(not(test))]
pub type MaxArraySize = ConstU32<10000>;

#[cfg(test)]
pub type MaxArraySize = ConstU32<100>;

/// Wrapper type for transaction execution result.
pub type TransactionExecutionResultWrapper<T> = Result<T, TransactionExecutionErrorWrapper>;

/// Wrapper type for transaction execution error.
#[derive(Debug, Error)]
pub enum TransactionExecutionErrorWrapper {
    /// Transaction execution error.
    #[error(transparent)]
    TransactionExecution(#[from] TransactionExecutionError),
    /// Starknet API error.
    #[error(transparent)]
    StarknetApi(#[from] StarknetApiError),
    /// Block context serialization error.
    #[error("Block context serialization error")]
    BlockContextSerializationError,
    /// State error.
    #[error(transparent)]
    StateError(#[from] StateError),
    /// Fee computation error,
    #[error("Fee computation error")]
    FeeComputationError,
    /// Fee transfer error,
    #[error("Fee transfer error. Max fee is {}, Actual fee is {}", max_fee.0, actual_fee.0)]
    FeeTransferError {
        /// Max fee specified by the set.
        max_fee: Fee,
        /// Actual fee.
        actual_fee: Fee,
    },
    /// Cairo resources are not contained in the fee costs.
    #[error("Cairo resources are not contained in the fee costs")]
    CairoResourcesNotContainedInFeeCosts,
    /// Failed to compute the L1 gas usage.
    #[error("Failed to compute the L1 gas usage")]
    FailedToComputeL1GasUsage,
    /// Entrypoint execution error
    #[error(transparent)]
    EntrypointExecution(#[from] EntryPointExecutionError),
    /// Unexpected holes.
    #[error("Unexpected holes: {0}")]
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
#[derive(Debug, Error)]
pub enum TransactionValidationErrorWrapper {
    /// Transaction execution error
    #[error(transparent)]
    TransactionValidationError(#[from] TransactionExecutionError),
    /// Calldata error
    #[error(transparent)]
    CalldataError(#[from] StarknetApiError),
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
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
<<<<<<< HEAD
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[serde(tag = "version")]
pub enum DeclareTransaction {
    #[serde(rename = "0x1")]
    V1(DeclareTransactionV1),
    #[serde(rename = "0x2")]
    V2(DeclareTransactionV2),
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
    /// Signaturecargo
    pub signature: BoundedVec<Felt252Wrapper, MaxArraySize>,
    // Transaction nonce
    pub nonce: Felt252Wrapper,
    /// Contract to declare.
    pub contract_class: ContractClassWrapper,
    /// The hash of the declared class
    pub class_hash: Felt252Wrapper,
    /// The address of the account contract sending the declaration transaction
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
    /// Signaturecarg
    pub signature: BoundedVec<Felt252Wrapper, MaxArraySize>,
    // Transaction nonce
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
        (signature, BoundedVec<Felt252Wrapper, MaxArraySize>),
        (contract_class, ContractClassWrapper)
    );

    pub fn version(&self) -> u8 {
        match self {
            DeclareTransaction::V1(_) => 1u8,
            DeclareTransaction::V2(_) => 2u8,
        }
    }
}

impl From<DeclareTransaction> for Transaction {
    /// converts the transaction to a [Transaction] object
    fn from(tx: DeclareTransaction) -> Transaction {
        Transaction::Declare(tx)
    }
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
    /// Signature
    pub signature: BoundedVec<Felt252Wrapper, MaxArraySize>,
    /// Nonce
    pub nonce: Felt252Wrapper,
    /// The salt for the address of the deployed contract
    pub contract_address_salt: Felt252Wrapper,
    /// The parameters passed to the constructor
    pub constructor_calldata: BoundedVec<Felt252Wrapper, MaxCalldataSize>,
    /// The hash of the deployed contract’s class
    pub class_hash: Felt252Wrapper,
    /// The address of the account contract being deployed
    pub sender_address: Felt252Wrapper,
    // Version transaction scheme
    pub version: u8,
}

impl From<DeployAccountTransaction> for Transaction {
    /// converts the transaction to a [Transaction] object
    fn from(tx: DeployAccountTransaction) -> Transaction {
        Transaction::DeployAccount(tx)
    }
}

/// Error of conversion between [DeclareTransaction], [InvokeTransaction],
/// [DeployAccountTransaction] and [Transaction].
#[derive(Debug, Error)]
pub enum TransactionConversionError {
    /// Wrong transaction type from the object of type [Transaction]
    #[error("Wrong transaction type (Invoke) from the object of type [Transaction]")]
    InvokeType,
    #[error("Wrong transaction type (L1Handler) the object of type [Transaction]")]
    L1HanderType,
    #[error("Wrong transaction type (deploy account) from the object of type [Transaction]")]
    DeployAccountType,
    #[error("Wrong transaction type (declare) from the object of type [Transaction]")]
    DeclareType,
    /// Class hash is missing from the object of type [Transaction]
    #[error("Class hash is missing from the object of type [Transaction]")]
    MissingClassHash,
    /// Casm class hash is missing from the object of type [Transaction]
    #[error("Casm class hash is missing from the object of type [Transaction]")]
    MissingCasmClassHash,
    /// Casm class hash must be None in [Transaction] for version <=1
    #[error("Casm class hash must be None in [Transaction] for version <=1")]
    CasmClashHashNotNone,
    /// Impossible to derive the contract address from the object of type [DeployAccountTransaction]
    #[error("Impossible to derive the contract address from the object of type [DeployAccountTransaction]")]
    ContractAddressDerivationError,
}
impl TryFrom<Transaction> for DeclareTransaction {
    type Error = TransactionConversionError;

    fn try_from(value: Transaction) -> Result<Self, Self::Error> {
        match value {
            Transaction::Invoke(_invoke_tx) => Err(TransactionConversionError::InvokeType),
            Transaction::DeployAccount(_deploy_account_tx) => Err(TransactionConversionError::DeployAccountType),
            Transaction::Declare(declare_tx) => Ok(declare_tx),
        }
    }
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
#[serde(tag = "version")]
pub enum InvokeTransaction {
    #[serde(rename = "0x0")]
    V0(InvokeTransactionV0),
    #[serde(rename = "0x1")]
    V1(InvokeTransactionV1),
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
    /// Signature
    pub signature: BoundedVec<Felt252Wrapper, MaxArraySize>,
    /// Nonce
    pub nonce: Felt252Wrapper,
    /// Contract address
    pub contract_address: Felt252Wrapper,
    /// Entry point selector
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
    /// The data expected by the account’s `execute` function (in most usecases, this includes the
    /// called contract address and a function selector)
    pub calldata: BoundedVec<Felt252Wrapper, MaxCalldataSize>,
}

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

// From https://github.com/tdelabro/starknet-api/blob/main/src/transaction.rs
impl InvokeTransaction {
    implement_invoke_tx_getters!(
        (transaction_hash, Felt252Wrapper),
        (nonce, Felt252Wrapper),
        (max_fee, Felt252Wrapper),
        (signature, BoundedVec<Felt252Wrapper, MaxArraySize>),
        (calldata, BoundedVec<Felt252Wrapper, MaxCalldataSize>)
    );

    pub fn version(&self) -> u8 {
        match self {
            InvokeTransaction::V0(_) => 0_u8,
            InvokeTransaction::V1(_) => 1_u8,
        }
    }

    pub fn sender_address(&self) -> Felt252Wrapper {
        match self {
            InvokeTransaction::V0(tx) => tx.contract_address,
            InvokeTransaction::V1(tx) => tx.sender_address,
        }
    }
}

<<<<<<< HEAD
impl TryFrom<Transaction> for InvokeTransaction {
    type Error = TransactionConversionError;

    fn try_from(value: Transaction) -> Result<Self, Self::Error> {
        match value {
            Transaction::Invoke(invoke_tx) => Ok(invoke_tx),
            Transaction::DeployAccount(_deploy_account_tx) => Err(TransactionConversionError::DeployAccountType),
            Transaction::Declare(_declare_tx) => Err(TransactionConversionError::DeclareType),
        }
    }
}

impl From<InvokeTransaction> for Transaction {
    /// converts the transaction to a [Transaction] object
    fn from(tx: InvokeTransaction) -> Transaction {
        Transaction::Invoke(tx)
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
<<<<<<< HEAD
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum Transaction {
    #[serde(rename = "INVOKE")]
    Invoke(InvokeTransaction),
    #[serde(rename = "DECLARE")]
    Declare(DeclareTransaction),
    #[serde(rename = "DEPLOY_ACCOUNT")]
    DeployAccount(DeployAccountTransaction),
}

impl Transaction {
    pub fn get_signature(&self) -> BoundedVec<Felt252Wrapper, MaxArraySize> {
        match self {
            Transaction::Invoke(invoke_tx) => invoke_tx.signature(),
            Transaction::DeployAccount(deploy_account_tx) => deploy_account_tx.signature.clone(),
            Transaction::Declare(declare_tx) => declare_tx.signature(),
        }
    }

    pub fn get_hash(&self) -> Felt252Wrapper {
        match self {
            Transaction::Invoke(invoke_tx) => invoke_tx.transaction_hash(),
            Transaction::DeployAccount(deploy_account_tx) => deploy_account_tx.transaction_hash.clone(),
            Transaction::Declare(declare_tx) => declare_tx.transaction_hash(),
        }
    }

    pub fn get_nonce(&self) -> Felt252Wrapper {
        match self {
            Transaction::Invoke(invoke_tx) => invoke_tx.nonce(),
            Transaction::DeployAccount(deploy_account_tx) => deploy_account_tx.nonce.clone(),
            Transaction::Declare(declare_tx) => declare_tx.nonce(),
        }
    }

    pub fn get_version(&self) -> u8 {
        match self {
            Transaction::Invoke(invoke_tx) => invoke_tx.version(),
            Transaction::DeployAccount(_deploy_account_tx) => 1_u8,
            Transaction::Declare(declare_tx) => declare_tx.version(),
        }
    }
}

impl TryFrom<Transaction> for DeployAccountTransaction {
    type Error = TransactionConversionError;
    fn try_from(value: Transaction) -> Result<Self, Self::Error> {
        match value {
            Transaction::Invoke(_invoke_tx) => Err(TransactionConversionError::InvokeType),
            Transaction::DeployAccount(deploy_account_tx) => Ok(deploy_account_tx),
            Transaction::Declare(_declare_tx) => Err(TransactionConversionError::DeclareType),
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
    pub transaction_hash: Felt252Wrapper,
    /// Fee paid for the transaction.
    pub actual_fee: Felt252Wrapper,
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
    Error,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum EventError {
    /// Provided keys are invalid.
    #[error("Provided keys are invalid")]
    InvalidKeys,
    /// Provided data is invalid.
    #[error("Provided data is invalid")]
    InvalidData,
    /// Provided from address is invalid.
    #[error("Provided from address is invalid")]
    InvalidFromAddress,
    /// Too many events
    #[error("Too many events")]
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
    Error,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum StateDiffError {
    /// Couldn't register newly deployed contracts.
    #[error("Couldn't register newly deployed contracts")]
    DeployedContractError,
    /// Couldn't register newly declared contracts.
    #[error("Couldn't register newly declared contracts")]
    DeclaredClassError,
}

#[cfg(feature = "std")]
mod reexport_private_types {    
    use starknet_core::types::contract::legacy::{
        LegacyEntrypointOffset, RawLegacyEntryPoint, RawLegacyEntryPoints,
    };
    use starknet_core::types::contract::ComputeClassHashError;
    use starknet_core::types::{
        BroadcastedDeployAccountTransaction, DeclareTransaction as RPCDeclareTransaction, DeclareTransactionV1 as RPCDeclareTransactionV1,
        DeclareTransactionV2 as RPCDeclareTransactionV2, DeployAccountTransaction as RPCDeployAccountTransaction, Event as RPCEvent,
        InvokeTransaction as RPCInvokeTransaction,
        InvokeTransactionV0 as RPCInvokeTransactionV0, InvokeTransactionV1 as RPCInvokeTransactionV1,
        LegacyContractEntryPoint, LegacyEntryPointsByType, StarknetError,
        Transaction as RPCTransaction,
    };

    use super::*;
    /// Wrapper type for broadcasted transaction conversion errors.
    #[derive(Debug, Error)]
    pub enum BroadcastedTransactionConversionErrorWrapper {
        /// Failed to decompress the contract class program
        #[error("Failed to decompress the contract class program")]
        ContractClassProgramDecompressionError,
        /// Failed to deserialize the contract class program
        #[error("Failed to deserialize the contract class program")]
        ContractClassProgramDeserializationError,
        /// Failed to convert signature
        #[error("Failed to convert signature")]
        SignatureConversionError,
        /// Failed to convert calldata
        #[error("Failed to convert calldata")]
        CalldataConversionError,
        /// Failed to convert program to program wrapper"
        #[error("Failed to convert program to program wrapper")]
        ProgramConversionError,
        /// Failed to bound signatures Vec<H256> by MaxArraySize
        #[error("failed to bound signatures Vec<H256> by MaxArraySize")]
        SignatureBoundError,
        /// Failed to bound calldata Vec<U256> by MaxCalldataSize
        #[error("failed to bound calldata Vec<U256> by MaxCalldataSize")]
        CalldataBoundError,
        /// Failed to compile Sierra to Casm
        #[error("failed to compile Sierra to Casm")]
        SierraCompilationError,
        /// Failed to convert Casm contract class to ContractClassV1
        #[error("failed to convert Casm contract class to ContractClassV1")]
        CasmContractClassConversionError,
        /// Computed compiled class hash doesn't match with the request
        #[error("compiled class hash does not match sierra code")]
        CompiledClassHashError,
        /// Starknet Error
        #[error(transparent)]
        StarknetError(#[from] StarknetError),
        /// Failed to convert transaction
        #[error(transparent)]
        TransactionConversionError(#[from] TransactionConversionError),
        /// Failed to compute the contract class hash.
        #[error(transparent)]
        ClassHashComputationError(#[from] ComputeClassHashError),
    }

    fn to_raw_legacy_entry_points(entry_points: LegacyEntryPointsByType) -> RawLegacyEntryPoints {
        RawLegacyEntryPoints {
            constructor: entry_points.constructor.into_iter().map(to_raw_legacy_entry_point).collect(),
            external: entry_points.external.into_iter().map(to_raw_legacy_entry_point).collect(),
            l1_handler: entry_points.l1_handler.into_iter().map(to_raw_legacy_entry_point).collect(),
        }
    }

    fn to_raw_legacy_entry_point(entry_point: LegacyContractEntryPoint) -> RawLegacyEntryPoint {
        RawLegacyEntryPoint {
            offset: LegacyEntrypointOffset::U64AsInt(entry_point.offset),
            selector: entry_point.selector,
        }
    }

    impl DeployAccountTransaction {
        fn try_from(
            tx: BroadcastedDeployAccountTransaction,
            chain_id: Felt252Wrapper,
        ) -> Result<DeployAccountTransaction, BroadcastedTransactionConversionErrorWrapper> {
            let version = 1_u64;
            let contract_address_salt = Felt252Wrapper::from(tx.contract_address_salt);
            let salt_as_felt = StarkFelt(contract_address_salt.into());
            let class_hash = Felt252Wrapper::from(tx.class_hash);
            let signature = tx
                .signature
                .iter()
                .map(|f| (*f).into())
                .collect::<Vec<Felt252Wrapper>>()
                .try_into()
                .map_err(|_| BroadcastedTransactionConversionErrorWrapper::SignatureBoundError)?;
            let constructor_calldata: BoundedVec<Felt252Wrapper, MaxCalldataSize> = tx
                .constructor_calldata
                .iter()
                .map(|f| (*f).into())
                .collect::<Vec<Felt252Wrapper>>()
                .try_into()
                .map_err(|_| BroadcastedTransactionConversionErrorWrapper::CalldataBoundError)?;
            let nonce = Felt252Wrapper::from(tx.nonce);
            let max_fee = Felt252Wrapper::from(tx.max_fee);

            let stark_felt_vec: Vec<StarkFelt> = constructor_calldata.clone()
                .into_inner()
                .into_iter()
                .map(|felt_wrapper| felt_wrapper.try_into().unwrap()) // Here, we are assuming that the conversion will not fail.
                .collect();

            let sender_address: ContractAddressWrapper = calculate_contract_address(
                ContractAddressSalt(salt_as_felt),
                ClassHash(class_hash.try_into().map_err(|_| TransactionConversionError::MissingClassHash)?),
                &Calldata(Arc::new(stark_felt_vec)),
                ContractAddress::default(),
            )
            .map_err(|_| TransactionConversionError::ContractAddressDerivationError)?
            .0
            .0
            .into();

            let transaction_hash = calculate_deploy_account_tx_hash(
                constructor_calldata.clone(),
                max_fee,
                class_hash,
                contract_address_salt,
                nonce,
                version,
                chain_id,
                sender_address,
            );

            Ok(DeployAccountTransaction {
                version: 1_u8,
                transaction_hash,
                max_fee,
                signature,
                nonce,
                contract_address_salt,
                constructor_calldata,
                class_hash,
                sender_address,
            })
        }
    }

    impl From<Transaction> for RPCTransaction {
        fn from(value: Transaction) -> Self {
            match value {
                Transaction::Declare(declare_tx) => match declare_tx {
                    DeclareTransaction::V1(declare_txn_v1) => {
                        let transaction_hash = declare_txn_v1.transaction_hash.0;
                        let max_fee = declare_txn_v1.max_fee.0;
                        let signature = declare_txn_v1.signature.iter().map(|&f| f.0).collect();
                        let nonce = declare_txn_v1.nonce.0;
                        let class_hash = declare_txn_v1.class_hash.0;
                        let sender_address = declare_txn_v1.sender_address.0;
                        RPCTransaction::Declare(RPCDeclareTransaction::V1(RPCDeclareTransactionV1 {
                            transaction_hash,
                            max_fee,
                            signature,
                            nonce,
                            class_hash,
                            sender_address,
                        }))
                    }
                    DeclareTransaction::V2(declare_txn_v2) => {
                        let transaction_hash = declare_txn_v2.transaction_hash.0;
                        let max_fee = declare_txn_v2.max_fee.0;
                        let signature = declare_txn_v2.signature.iter().map(|&f| f.0).collect();
                        let nonce = declare_txn_v2.nonce.0;
                        let class_hash = declare_txn_v2.class_hash.0;
                        let sender_address = declare_txn_v2.sender_address.0;
                        let compiled_class_hash = declare_txn_v2.compiled_class_hash.0;
                        RPCTransaction::Declare(RPCDeclareTransaction::V2(RPCDeclareTransactionV2 {
                            transaction_hash,
                            max_fee,
                            signature,
                            nonce,
                            class_hash,
                            sender_address,
                            compiled_class_hash,
                        }))
                    }
                },
                Transaction::Invoke(invoke_tx) => match invoke_tx {
                    InvokeTransaction::V0(invoke_txn_v0) => {
                        let transaction_hash = invoke_txn_v0.transaction_hash.0;
                        let max_fee = invoke_txn_v0.max_fee.0;
                        let signature = invoke_txn_v0.signature.iter().map(|&f| f.0).collect();
                        let nonce = invoke_txn_v0.nonce.0;
                        let contract_address = invoke_txn_v0.contract_address.0;
                        let entry_point_selector = invoke_txn_v0.entry_point_selector.0;
                        let calldata = invoke_txn_v0.calldata.iter().map(|&f| f.0).collect();
                        RPCTransaction::Invoke(RPCInvokeTransaction::V0(RPCInvokeTransactionV0 {
                            transaction_hash,
                            max_fee,
                            signature,
                            nonce,
                            contract_address,
                            entry_point_selector,
                            calldata,
                        }))
                    }
                    InvokeTransaction::V1(invoke_txn_v1) => {
                        let transaction_hash = invoke_txn_v1.transaction_hash.0;
                        let max_fee = invoke_txn_v1.max_fee.0;
                        let signature = invoke_txn_v1.signature.iter().map(|&f| f.0).collect();
                        let nonce = invoke_txn_v1.nonce.0;
                        let sender_address = invoke_txn_v1.sender_address.0;
                        let calldata = invoke_txn_v1.calldata.iter().map(|&f| f.0).collect();
                        RPCTransaction::Invoke(RPCInvokeTransaction::V1(RPCInvokeTransactionV1 {
                            transaction_hash,
                            max_fee,
                            signature,
                            nonce,
                            sender_address,
                            calldata,
                        }))
                    }
                },
                Transaction::DeployAccount(deploy_tx) => {
                    let transaction_hash = deploy_tx.transaction_hash.0;
                    let max_fee = deploy_tx.max_fee.0;
                    let signature = deploy_tx.signature.iter().map(|&f| f.0).collect();
                    let nonce = deploy_tx.nonce.0;
                    let contract_address_salt = deploy_tx.contract_address_salt.0;
                    let constructor_calldata = deploy_tx.constructor_calldata.iter().map(|&f| f.0).collect();
                    let class_hash = deploy_tx.class_hash.0;
                    RPCTransaction::DeployAccount(RPCDeployAccountTransaction {
                        transaction_hash,
                        max_fee,
                        signature,
                        nonce,
                        contract_address_salt,
                        constructor_calldata,
                        class_hash,
                    })
                }
            }
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

    impl From<EventWrapper> for RPCEvent {
        fn from(value: EventWrapper) -> Self {
            Self {
                from_address: value.from_address.into(),
                keys: value.keys.iter().map(|k| (*k).into()).collect(),
                data: value.data.iter().map(|d| (*d).into()).collect(),
            }
        }
    }
}

#[cfg(feature = "std")]
pub use reexport_private_types::*;
