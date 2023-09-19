#[cfg(test)]
#[path = "transaction_test.rs"]
mod transaction_test;

use std::collections::HashMap;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use starknet_api::api_core::{
    ClassHash,
    CompiledClassHash,
    ContractAddress,
    EntryPointSelector,
    EthAddress,
    Nonce,
};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{
    Calldata,
    ContractAddressSalt,
    DeclareTransactionOutput,
    DeployAccountTransactionOutput,
    DeployTransactionOutput,
    Event,
    Fee,
    InvokeTransactionOutput,
    L1HandlerTransactionOutput,
    L1ToL2Payload,
    L2ToL1Payload,
    MessageToL1,
    TransactionExecutionStatus,
    TransactionHash,
    TransactionOffsetInBlock,
    TransactionOutput,
    TransactionSignature,
    TransactionVersion,
};

use crate::reader::ReaderClientError;

lazy_static! {
    static ref TX_V0: TransactionVersion = TransactionVersion(StarkFelt::from(0u128));
    static ref TX_V1: TransactionVersion = TransactionVersion(StarkFelt::from(1u128));
    static ref TX_V2: TransactionVersion = TransactionVersion(StarkFelt::from(2u128));
}

// TODO(dan): consider extracting common fields out (version, hash, type).
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum Transaction {
    #[serde(rename = "DECLARE")]
    Declare(IntermediateDeclareTransaction),
    #[serde(rename = "DEPLOY_ACCOUNT")]
    DeployAccount(DeployAccountTransaction),
    #[serde(rename = "DEPLOY")]
    Deploy(DeployTransaction),
    #[serde(rename = "INVOKE_FUNCTION")]
    Invoke(IntermediateInvokeTransaction),
    #[serde(rename = "L1_HANDLER")]
    L1Handler(L1HandlerTransaction),
}

impl TryFrom<Transaction> for starknet_api::transaction::Transaction {
    type Error = ReaderClientError;
    fn try_from(tx: Transaction) -> Result<Self, ReaderClientError> {
        match tx {
            Transaction::Declare(declare_tx) => {
                Ok(starknet_api::transaction::Transaction::Declare(declare_tx.try_into()?))
            }
            Transaction::Deploy(deploy_tx) => {
                Ok(starknet_api::transaction::Transaction::Deploy(deploy_tx.into()))
            }
            Transaction::DeployAccount(deploy_acc_tx) => {
                Ok(starknet_api::transaction::Transaction::DeployAccount(deploy_acc_tx.into()))
            }
            Transaction::Invoke(invoke_tx) => {
                Ok(starknet_api::transaction::Transaction::Invoke(invoke_tx.try_into()?))
            }
            Transaction::L1Handler(l1_handler_tx) => {
                Ok(starknet_api::transaction::Transaction::L1Handler(l1_handler_tx.into()))
            }
        }
    }
}

impl Transaction {
    pub fn transaction_hash(&self) -> TransactionHash {
        match self {
            Transaction::Declare(tx) => tx.transaction_hash,
            Transaction::Deploy(tx) => tx.transaction_hash,
            Transaction::DeployAccount(tx) => tx.transaction_hash,
            Transaction::Invoke(tx) => tx.transaction_hash,
            Transaction::L1Handler(tx) => tx.transaction_hash,
        }
    }

    pub fn transaction_type(&self) -> TransactionType {
        match self {
            Transaction::Declare(_) => TransactionType::Declare,
            Transaction::Deploy(_) => TransactionType::Deploy,
            Transaction::DeployAccount(_) => TransactionType::DeployAccount,
            Transaction::Invoke(_) => TransactionType::InvokeFunction,
            Transaction::L1Handler(_) => TransactionType::L1Handler,
        }
    }

    pub fn contract_address(&self) -> Option<ContractAddress> {
        match self {
            Transaction::Deploy(tx) => Some(tx.contract_address),
            Transaction::DeployAccount(tx) => Some(tx.contract_address),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct L1HandlerTransaction {
    pub transaction_hash: TransactionHash,
    pub version: TransactionVersion,
    #[serde(default)]
    pub nonce: Nonce,
    pub contract_address: ContractAddress,
    pub entry_point_selector: EntryPointSelector,
    pub calldata: Calldata,
}

impl From<L1HandlerTransaction> for starknet_api::transaction::L1HandlerTransaction {
    fn from(l1_handler_tx: L1HandlerTransaction) -> Self {
        starknet_api::transaction::L1HandlerTransaction {
            version: l1_handler_tx.version,
            nonce: l1_handler_tx.nonce,
            contract_address: l1_handler_tx.contract_address,
            entry_point_selector: l1_handler_tx.entry_point_selector,
            calldata: l1_handler_tx.calldata,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct IntermediateDeclareTransaction {
    pub class_hash: ClassHash,
    pub compiled_class_hash: Option<CompiledClassHash>,
    pub sender_address: ContractAddress,
    pub nonce: Nonce,
    pub max_fee: Fee,
    pub version: TransactionVersion,
    pub transaction_hash: TransactionHash,
    pub signature: TransactionSignature,
}

impl TryFrom<IntermediateDeclareTransaction> for starknet_api::transaction::DeclareTransaction {
    type Error = ReaderClientError;

    fn try_from(declare_tx: IntermediateDeclareTransaction) -> Result<Self, ReaderClientError> {
        match declare_tx.version {
            v if v == *TX_V0 => Ok(Self::V0(declare_tx.into())),
            v if v == *TX_V1 => Ok(Self::V1(declare_tx.into())),
            v if v == *TX_V2 => Ok(Self::V2(declare_tx.try_into()?)),
            _ => Err(ReaderClientError::BadTransaction {
                tx_hash: declare_tx.transaction_hash,
                msg: format!("Declare version {:?} is not supported.", declare_tx.version),
            }),
        }
    }
}

impl From<IntermediateDeclareTransaction> for starknet_api::transaction::DeclareTransactionV0V1 {
    fn from(declare_tx: IntermediateDeclareTransaction) -> Self {
        Self {
            max_fee: declare_tx.max_fee,
            signature: declare_tx.signature,
            nonce: declare_tx.nonce,
            class_hash: declare_tx.class_hash,
            sender_address: declare_tx.sender_address,
        }
    }
}

impl TryFrom<IntermediateDeclareTransaction> for starknet_api::transaction::DeclareTransactionV2 {
    type Error = ReaderClientError;

    fn try_from(declare_tx: IntermediateDeclareTransaction) -> Result<Self, ReaderClientError> {
        Ok(Self {
            max_fee: declare_tx.max_fee,
            signature: declare_tx.signature,
            nonce: declare_tx.nonce,
            class_hash: declare_tx.class_hash,
            compiled_class_hash: declare_tx.compiled_class_hash.ok_or(
                ReaderClientError::BadTransaction {
                    tx_hash: declare_tx.transaction_hash,
                    msg: "Declare V2 must contain compiled_class_hash field.".to_string(),
                },
            )?,
            sender_address: declare_tx.sender_address,
        })
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DeployTransaction {
    pub contract_address: ContractAddress,
    pub contract_address_salt: ContractAddressSalt,
    pub class_hash: ClassHash,
    pub constructor_calldata: Calldata,
    pub transaction_hash: TransactionHash,
    #[serde(default)]
    pub version: TransactionVersion,
}

impl From<DeployTransaction> for starknet_api::transaction::DeployTransaction {
    fn from(deploy_tx: DeployTransaction) -> Self {
        starknet_api::transaction::DeployTransaction {
            version: deploy_tx.version,
            constructor_calldata: deploy_tx.constructor_calldata,
            class_hash: deploy_tx.class_hash,
            contract_address_salt: deploy_tx.contract_address_salt,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DeployAccountTransaction {
    pub contract_address: ContractAddress,
    pub contract_address_salt: ContractAddressSalt,
    pub class_hash: ClassHash,
    pub constructor_calldata: Calldata,
    pub nonce: Nonce,
    pub max_fee: Fee,
    pub signature: TransactionSignature,
    pub transaction_hash: TransactionHash,
    #[serde(default)]
    pub version: TransactionVersion,
}

impl From<DeployAccountTransaction> for starknet_api::transaction::DeployAccountTransaction {
    fn from(deploy_tx: DeployAccountTransaction) -> Self {
        starknet_api::transaction::DeployAccountTransaction {
            version: deploy_tx.version,
            constructor_calldata: deploy_tx.constructor_calldata,
            class_hash: deploy_tx.class_hash,
            contract_address_salt: deploy_tx.contract_address_salt,
            max_fee: deploy_tx.max_fee,
            signature: deploy_tx.signature,
            nonce: deploy_tx.nonce,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct IntermediateInvokeTransaction {
    pub calldata: Calldata,
    // In early versions of starknet, the `sender_address` field was originally named
    // `contract_address`.
    #[serde(alias = "contract_address")]
    pub sender_address: ContractAddress,
    pub entry_point_selector: Option<EntryPointSelector>,
    #[serde(default)]
    pub nonce: Option<Nonce>,
    pub max_fee: Fee,
    pub signature: TransactionSignature,
    pub transaction_hash: TransactionHash,
    pub version: TransactionVersion,
}

impl TryFrom<IntermediateInvokeTransaction> for starknet_api::transaction::InvokeTransaction {
    type Error = ReaderClientError;

    fn try_from(invoke_tx: IntermediateInvokeTransaction) -> Result<Self, ReaderClientError> {
        match invoke_tx.version {
            v if v == *TX_V0 => Ok(Self::V0(invoke_tx.try_into()?)),
            v if v == *TX_V1 => Ok(Self::V1(invoke_tx.try_into()?)),
            _ => Err(ReaderClientError::BadTransaction {
                tx_hash: invoke_tx.transaction_hash,
                msg: format!("Invoke version {:?} is not supported.", invoke_tx.version),
            }),
        }
    }
}

impl TryFrom<IntermediateInvokeTransaction> for starknet_api::transaction::InvokeTransactionV0 {
    type Error = ReaderClientError;

    fn try_from(invoke_tx: IntermediateInvokeTransaction) -> Result<Self, ReaderClientError> {
        Ok(Self {
            max_fee: invoke_tx.max_fee,
            signature: invoke_tx.signature,
            contract_address: invoke_tx.sender_address,
            entry_point_selector: invoke_tx.entry_point_selector.ok_or(
                ReaderClientError::BadTransaction {
                    tx_hash: invoke_tx.transaction_hash,
                    msg: "Invoke V0 must contain entry_point_selector field.".to_string(),
                },
            )?,
            calldata: invoke_tx.calldata,
        })
    }
}

impl TryFrom<IntermediateInvokeTransaction> for starknet_api::transaction::InvokeTransactionV1 {
    type Error = ReaderClientError;

    fn try_from(invoke_tx: IntermediateInvokeTransaction) -> Result<Self, ReaderClientError> {
        // TODO(yair): Consider asserting that entry_point_selector is None.
        Ok(Self {
            max_fee: invoke_tx.max_fee,
            signature: invoke_tx.signature,
            nonce: invoke_tx.nonce.ok_or(ReaderClientError::BadTransaction {
                tx_hash: invoke_tx.transaction_hash,
                msg: "Invoke V1 must contain nonce field.".to_string(),
            })?,
            sender_address: invoke_tx.sender_address,
            calldata: invoke_tx.calldata,
        })
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct TransactionReceipt {
    pub transaction_index: TransactionOffsetInBlock,
    pub transaction_hash: TransactionHash,
    #[serde(default)]
    pub l1_to_l2_consumed_message: L1ToL2Message,
    pub l2_to_l1_messages: Vec<L2ToL1Message>,
    pub events: Vec<Event>,
    #[serde(default)]
    pub execution_resources: ExecutionResources,
    pub actual_fee: Fee,
    #[serde(default)]
    pub execution_status: TransactionExecutionStatus,
}

impl TransactionReceipt {
    pub fn into_starknet_api_transaction_output(
        self,
        transaction: &Transaction,
    ) -> TransactionOutput {
        let messages_sent = self.l2_to_l1_messages.into_iter().map(MessageToL1::from).collect();
        let contract_address = transaction.contract_address();
        match transaction.transaction_type() {
            TransactionType::Declare => TransactionOutput::Declare(DeclareTransactionOutput {
                actual_fee: self.actual_fee,
                messages_sent,
                events: self.events,
                execution_status: self.execution_status,
            }),
            TransactionType::Deploy => TransactionOutput::Deploy(DeployTransactionOutput {
                actual_fee: self.actual_fee,
                messages_sent,
                events: self.events,
                contract_address: contract_address
                    .expect("Deploy transaction must have a contract address."),
                execution_status: self.execution_status,
            }),
            TransactionType::DeployAccount => {
                TransactionOutput::DeployAccount(DeployAccountTransactionOutput {
                    actual_fee: self.actual_fee,
                    messages_sent,
                    events: self.events,
                    contract_address: contract_address
                        .expect("Deploy account transaction must have a contract address."),
                    execution_status: self.execution_status,
                })
            }
            TransactionType::InvokeFunction => TransactionOutput::Invoke(InvokeTransactionOutput {
                actual_fee: self.actual_fee,
                messages_sent,
                events: self.events,
                execution_status: self.execution_status,
            }),
            TransactionType::L1Handler => {
                TransactionOutput::L1Handler(L1HandlerTransactionOutput {
                    actual_fee: self.actual_fee,
                    messages_sent,
                    events: self.events,
                    execution_status: self.execution_status,
                })
            }
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct ExecutionResources {
    pub n_steps: u64,
    pub builtin_instance_counter: BuiltinInstanceCounter,
    pub n_memory_holes: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum BuiltinInstanceCounter {
    NonEmpty(HashMap<String, u64>),
    Empty(EmptyBuiltinInstanceCounter),
}

impl Default for BuiltinInstanceCounter {
    fn default() -> Self {
        BuiltinInstanceCounter::Empty(EmptyBuiltinInstanceCounter {})
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct EmptyBuiltinInstanceCounter {}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct L1ToL2Nonce(pub StarkHash);

#[derive(Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct L1ToL2Message {
    pub from_address: EthAddress,
    pub to_address: ContractAddress,
    pub selector: EntryPointSelector,
    pub payload: L1ToL2Payload,
    #[serde(default)]
    pub nonce: L1ToL2Nonce,
}

impl From<L1ToL2Message> for starknet_api::transaction::MessageToL2 {
    fn from(message: L1ToL2Message) -> Self {
        starknet_api::transaction::MessageToL2 {
            from_address: message.from_address,
            payload: message.payload,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct L2ToL1Message {
    pub from_address: ContractAddress,
    pub to_address: EthAddress,
    pub payload: L2ToL1Payload,
}

impl From<L2ToL1Message> for starknet_api::transaction::MessageToL1 {
    fn from(message: L2ToL1Message) -> Self {
        starknet_api::transaction::MessageToL1 {
            to_address: message.to_address,
            payload: message.payload,
            from_address: message.from_address,
        }
    }
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord, Default,
)]
pub enum TransactionType {
    #[serde(rename(deserialize = "DECLARE", serialize = "DECLARE"))]
    Declare,
    #[serde(rename(deserialize = "DEPLOY", serialize = "DEPLOY"))]
    Deploy,
    #[serde(rename(deserialize = "DEPLOY_ACCOUNT", serialize = "DEPLOY_ACCOUNT"))]
    DeployAccount,
    #[serde(rename(deserialize = "INVOKE_FUNCTION", serialize = "INVOKE_FUNCTION"))]
    #[default]
    InvokeFunction,
    #[serde(rename(deserialize = "L1_HANDLER", serialize = "L1_HANDLER"))]
    L1Handler,
}
