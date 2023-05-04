extern crate serde;
extern crate serde_json;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
/// BlockHash
///
/// The hash of the block in which the event was emitted
pub type BlockHash = String;
/// BlockNumber
///
/// The block's number (its height)
pub type BlockNumber = i64;
/// BlockTag
///
/// A tag specifying a dynamic reference to a block
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum BlockTag {
    #[serde(rename = "latest")]
    Latest,
    #[serde(rename = "pending")]
    Pending,
}
/// ContractAddress
///
/// The address of the deployed contract
pub type ContractAddress = String;
/// EntryPointSelector
///
/// A field element. represented by at most 63 hex digits
pub type EntryPointSelector = String;
/// FieldElement
///
/// A field element. represented by at most 63 hex digits
pub type FieldElement = String;
/// Calldata
///
/// The parameters passed to the function
pub type Calldata = Vec<FieldElement>;
/// MaxFee
///
/// The maximal fee that can be charged for including the transaction
pub type MaxFee = String;
/// Version
///
/// Version of the transaction scheme
pub type Version = String;
pub type Signature = Vec<FieldElement>;
/// Nonce
///
/// The nonce for the given address at the end of the block
pub type Nonce = String;
/// BroadcastedTransactionCommonProperties
///
/// common properties of a transaction that is sent to the sequencer (but is not yet in a block)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct BroadcastedTransactionCommonProperties {
    pub max_fee: MaxFee,
    pub version: Version,
    pub signature: Signature,
    pub nonce: Nonce,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct Type {
    #[serde(rename = "type")]
    pub _type: Type,
}
pub type EventEmitter = HashMap<String, serde_json::Value>;
/// InvokeTransactionV0
///
/// invokes a specific function in the desired contract (not necessarily an account)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct InvokeTransactionV0 {
    pub contract_address: ContractAddress,
    pub entry_point_selector: EntryPointSelector,
    pub calldata: Calldata,
}
/// SenderAddress
///
/// A field element. represented by at most 63 hex digits
pub type SenderAddress = String;
/// Calldata
///
/// The data expected by the account's `execute` function (in most use cases, this includes the
/// called contract address and a function selector)
/// InvokeTransactionV1
///
/// initiates a transaction from a given account
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct InvokeTransactionV1 {
    pub sender_address: SenderAddress,
    pub calldata: Calldata,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum InvokeTransactionProperties {
    InvokeTransactionV0(InvokeTransactionV0),
    InvokeTransactionV1(InvokeTransactionV1),
}
/// BroadcastedInvokeTransaction
///
/// mempool representation of an invoke transaction
pub type BroadcastedInvokeTransaction = HashMap<String, serde_json::Value>;
/// Program
///
/// A base64 representation of the compressed program code
pub type Program = String;
/// Offset
///
/// offset of this property within the struct
pub type Offset = i64;
/// Selector
///
/// A unique identifier of the entry point (function) in the program
pub type Selector = String;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct DeprecatedCairoEntryPoint {
    pub offset: Offset,
    pub selector: Selector,
}
pub type DeprecatedConstructor = Vec<DeprecatedCairoEntryPoint>;
pub type DeprecatedExternal = Vec<DeprecatedCairoEntryPoint>;
pub type DeprecatedL1Handler = Vec<DeprecatedCairoEntryPoint>;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct DeprecatedEntryPointsByType {
    #[serde(rename = "CONSTRUCTOR", skip_serializing_if = "Option::is_none")]
    pub constructor: Option<DeprecatedConstructor>,
    #[serde(rename = "EXTERNAL", skip_serializing_if = "Option::is_none")]
    pub external: Option<DeprecatedExternal>,
    #[serde(rename = "L1_HANDLER", skip_serializing_if = "Option::is_none")]
    pub l_1_handler: Option<DeprecatedL1Handler>,
}
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum FunctionABIType {
    #[serde(rename = "function")]
    Function,
    #[serde(rename = "l1_handler")]
    LOneHandler,
    #[serde(rename = "constructor")]
    Constructor,
}
/// FunctionName
///
/// The function name
pub type FunctionName = String;
/// ParameterName
///
/// The parameter's name
pub type ParameterName = String;
/// ParameterType
///
/// The parameter's type
pub type ParameterType = String;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct TypedParameter {
    pub name: ParameterName,
    #[serde(rename = "type")]
    pub _type: ParameterType,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct FunctionABIEntry {
    #[serde(rename = "type")]
    pub _type: FunctionABIType,
    pub name: FunctionName,
    pub inputs: TypedParameter,
    pub outputs: TypedParameter,
}
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum EventABIType {
    #[serde(rename = "event")]
    Event,
}
/// EventName
///
/// The event name
pub type EventName = String;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct EventABIEntry {
    #[serde(rename = "type")]
    pub _type: EventABIType,
    pub name: EventName,
    pub keys: TypedParameter,
    pub data: TypedParameter,
}
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum StructABIType {
    #[serde(rename = "struct")]
    Struct,
}
/// StructName
///
/// The struct name
pub type StructName = String;
pub type Size = i64;
pub type StructMember = HashMap<String, serde_json::Value>;
pub type Members = Vec<StructMember>;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct ContractClass {
    pub sierra_program: SierraProgram,
    pub contract_class_version: ContractClassVersion,
    pub entry_points_by_type: EntryPointsByType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abi: Option<ABI>,
}
/// SenderAddress
///
/// The address of the account contract sending the declaration transaction
pub type BroadcastedDeclareTransactionV1 = HashMap<String, serde_json::Value>;
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum Declare {
    #[serde(rename = "DECLARE")]
    Declare,
}
/// SierraProgram
///
/// The list of Sierra instructions of which the program consists
pub type SierraProgram = Vec<FieldElement>;
/// ContractClassVersion
///
/// The version of the contract class object. Currently, the Starknet OS supports version 0.1.0
pub type ContractClassVersion = String;
/// FunctionIndex
///
/// The index of the function in the program
pub type FunctionIndex = i64;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct SierraEntryPoint {
    pub selector: Selector,
    pub function_idx: FunctionIndex,
}
pub type Constructor = Vec<SierraEntryPoint>;
pub type External = Vec<SierraEntryPoint>;
pub type L1Handler = Vec<SierraEntryPoint>;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct EntryPointsByType {
    #[serde(rename = "CONSTRUCTOR")]
    pub constructor: Constructor,
    #[serde(rename = "EXTERNAL")]
    pub external: External,
    #[serde(rename = "L1_HANDLER")]
    pub l_1_handler: L1Handler,
}
pub type ABI = String;
/// CompiledClassHash
///
/// The Cairo assembly hash corresponding to the declared class
pub type CompiledClassHash = String;
pub type BroadcastedDeclareTransactionV2 = HashMap<String, serde_json::Value>;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum BroadcastedDeclareTransaction {
    BroadcastedDeclareTransactionV1(BroadcastedDeclareTransactionV1),
    BroadcastedDeclareTransactionV2(BroadcastedDeclareTransactionV2),
}
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum DeployAccount {
    #[serde(rename = "DEPLOY_ACCOUNT")]
    DeployAccount,
}
/// ContractAddressSalt
///
/// The salt for the address of the deployed contract
pub type ContractAddressSalt = String;
/// ConstructorCalldata
///
/// The parameters passed to the constructor
pub type ConstructorCalldata = Vec<FieldElement>;
/// ClassHash
///
/// The new class hash
pub type ClassHash = String;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct DeployAccountTransactionProperties {
    #[serde(rename = "type")]
    pub _type: DeployAccount,
    pub contract_address_salt: ContractAddressSalt,
    pub constructor_calldata: ConstructorCalldata,
    pub class_hash: ClassHash,
}
/// BroadcastedDeployAccountTransaction
///
/// Mempool representation of a deploy account transaction
pub type BroadcastedDeployAccountTransaction = HashMap<String, serde_json::Value>;
/// BroadcastedTransaction
///
/// a sequence of transactions to estimate, running each transaction on the state resulting from
/// applying all the previous ones
pub type BroadcastedTransaction = Vec<BroadcastedDeployAccountTransaction>;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum FromBlock {
    BlockHash(BlockHash),
    BlockNumber(BlockNumber),
    BlockTag(BlockTag),
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum ToBlock {
    BlockHash(BlockHash),
    BlockNumber(BlockNumber),
    BlockTag(BlockTag),
}
/// FromContract
///
/// A field element. represented by at most 63 hex digits
pub type FromContract = String;
pub type Keys = Vec<FieldElement>;
/// EventFilter
///
/// An event filter/query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct EventFilter {
    pub from_block: FromBlock,
    pub to_block: ToBlock,
    pub address: FromContract,
    pub keys: Keys,
}
/// ContinuationToken
///
/// Use this token in a subsequent query to obtain the next page. Should not appear if there are no
/// more pages.
pub type ContinuationToken = String;
pub type ChunkSize = i64;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct ResultPageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<ContinuationToken>,
    pub chunk_size: ChunkSize,
}
/// Status
///
/// The status of the transaction
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum Status {
    #[serde(rename = "PENDING")]
    Pending,
    #[serde(rename = "ACCEPTED_ON_L2")]
    AcceptedOnLTwo,
    #[serde(rename = "ACCEPTED_ON_L1")]
    AcceptedOnLOne,
    #[serde(rename = "REJECTED")]
    Rejected,
}
/// ParentHash
///
/// The hash of this block's parent
pub type ParentHash = String;
/// NewRoot
///
/// The new global state root
pub type NewRoot = String;
/// Timestamp
///
/// The time in which the block was created, encoded in Unix time
pub type Timestamp = i64;
/// SequencerAddress
///
/// The StarkNet identity of the sequencer submitting this block
pub type SequencerAddress = String;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct BlockHeader {
    pub block_hash: BlockHash,
    pub parent_hash: ParentHash,
    pub block_number: BlockNumber,
    pub new_root: NewRoot,
    pub timestamp: Timestamp,
    pub sequencer_address: SequencerAddress,
}
/// TransactionHash
///
/// The transaction hash, as assigned in StarkNet
pub type TransactionHash = String;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum Transaction {
    InvokeTransaction(InvokeTransaction),
    L1HandlerTransaction(L1HandlerTransaction),
    DeclareTransaction(DeclareTransaction),
    DeployTransaction(DeployTransaction),
    DeployAccountTransaction(DeployAccountTransaction),
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct BlockBodyWithTransactionHashes {
    pub transactions: Transaction,
}
pub type BlockWithTransactionHashes = HashMap<String, serde_json::Value>;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct BlockBodyWithTransactionsHashes {
    pub transactions: Transaction,
}
/// PendingBlockWithTransactionHashes
///
/// The dynamic block being constructed by the sequencer. Note that this object will be deprecated
/// upon decentralization.
pub type PendingBlockWithTransactionHashes = HashMap<String, serde_json::Value>;
pub type CommonTransactionProperties = HashMap<String, serde_json::Value>;
/// InvokeTransaction
///
/// Initiate a transaction from an account
pub type InvokeTransaction = HashMap<String, serde_json::Value>;

pub type L1HandlerTransaction = HashMap<String, serde_json::Value>;
/// FunctionCall
///
/// Function call information
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct FunctionCall {
    pub contract_address: ContractAddress,
    pub entry_point_selector: EntryPointSelector,
    pub calldata: Calldata,
}
pub type DeclareTransactionV1 = HashMap<String, serde_json::Value>;
pub type DeclareTransactionV2 = HashMap<String, serde_json::Value>;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum DeclareTransaction {
    DeclareTransactionV1(DeclareTransactionV1),
    DeclareTransactionV2(DeclareTransactionV2),
}
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum Deploy {
    #[serde(rename = "DEPLOY")]
    Deploy,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct DeployTransactionProperties {
    pub version: Version,
    #[serde(rename = "type")]
    pub _type: Deploy,
    pub contract_address_salt: ContractAddressSalt,
    pub constructor_calldata: ConstructorCalldata,
}
/// DeployTransaction
///
/// The structure of a deploy transaction. Note that this transaction type is deprecated and will no
/// longer be supported in future versions
pub type DeployTransaction = HashMap<String, serde_json::Value>;
/// DeployAccountTransaction
///
/// Deploys an account contract, charges fee from the pre-funded account addresses
pub type DeployAccountTransaction = HashMap<String, serde_json::Value>;
/// Transactions
///
/// The transactions in this block
pub type Transactions = Vec<Transaction>;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct BlockBodyWithTransactions {
    pub transactions: Transactions,
}
/// BlockWithTransactions
///
/// The block object
pub type BlockWithTransactions = HashMap<String, serde_json::Value>;
/// PendingBlockWithTransactions
///
/// The dynamic block being constructed by the sequencer. Note that this object will be deprecated
/// upon decentralization.
pub type PendingBlockWithTransactions = HashMap<String, serde_json::Value>;
/// OldRoot
///
/// The previous global state root
pub type OldRoot = String;
/// Address
///
/// A field element. represented by at most 63 hex digits
pub type Address = String;
/// Key
///
/// The key of the changed value
pub type Key = String;
/// Value
///
/// The new value applied to the given address
pub type Value = String;
/// StorageEntries
///
/// The changes in the storage of the contract
pub type StorageEntries = Vec<EventEmitter>;
/// ContractStorageDiffItem
///
/// The changes in the storage per contract address
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct ContractStorageDiffItem {
    pub address: Address,
    pub storage_entries: StorageEntries,
}
pub type StorageDiffs = Vec<ContractStorageDiffItem>;
pub type DeprecatedDeclaredClasses = Vec<FieldElement>;
pub type DeclaredClasses = Vec<EventEmitter>;
/// DeployedContractItem
///
/// A new contract deployed as part of the state update
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct DeployedContractItem {
    pub address: Address,
    pub class_hash: ClassHash,
}
pub type DeployedContracts = Vec<DeployedContractItem>;
pub type ReplacedClasses = Vec<EventEmitter>;
pub type Nonces = Vec<EventEmitter>;
/// StateDiff
///
/// The change in state applied in this block, given as a mapping of addresses to the new values
/// and/or new contracts
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct StateDiff {
    pub storage_diffs: StorageDiffs,
    pub deprecated_declared_classes: DeprecatedDeclaredClasses,
    pub declared_classes: DeclaredClasses,
    pub deployed_contracts: DeployedContracts,
    pub replaced_classes: ReplacedClasses,
    pub nonces: Nonces,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct PendingStateUpdate {
    pub old_root: OldRoot,
    pub state_diff: StateDiff,
}
pub type StateUpdate = HashMap<String, serde_json::Value>;
/// ActualFee
///
/// The fee that was charged by the sequencer
pub type ActualFee = String;
/// ToAddress
///
/// The target L1 address the message is sent to
pub type ToAddress = String;
/// Payload
///
/// The payload of the message
pub type Payload = Vec<FieldElement>;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct MessageToL1 {
    pub from_address: FieldElement,
    pub to_address: ToAddress,
    pub payload: Payload,
}
pub type MessagesSent = Vec<MessageToL1>;
/// FromAddress
///
/// A field element. represented by at most 63 hex digits
pub type FromAddress = String;
pub type Data = Vec<FieldElement>;
/// EventContent
///
/// The content of an event
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct EventContent {
    pub keys: Keys,
    pub data: Data,
}
/// Event
///
/// The event information
pub type Event = HashMap<String, serde_json::Value>;
/// Events
///
/// The events emitted as part of this transaction
pub type Events = Vec<Event>;
/// CommonReceiptProperties
///
/// Common properties for a pending transaction receipt
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct CommonReceiptProperties {
    pub transaction_hash: TransactionHash,
    pub actual_fee: ActualFee,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub _type: Option<TransactionType>,
    pub messages_sent: MessagesSent,
    pub events: Events,
}
pub type InvokeTransactionReceipt = HashMap<String, serde_json::Value>;
/// L1HandlerTransactionReceipt
///
/// receipt for l1 handler transaction
pub type L1HandlerTransactionReceipt = HashMap<String, serde_json::Value>;
pub type DeclareTransactionReceipt = HashMap<String, serde_json::Value>;
pub type DeployTransactionReceipt = HashMap<String, serde_json::Value>;
pub type DeployAccountTransactionReceipt = HashMap<String, serde_json::Value>;
/// TransactionType
///
/// The type of the transaction
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum TransactionType {
    #[serde(rename = "DECLARE")]
    Declare,
    #[serde(rename = "DEPLOY")]
    Deploy,
    #[serde(rename = "DEPLOY_ACCOUNT")]
    DeployAccount,
    #[serde(rename = "INVOKE")]
    Invoke,
    #[serde(rename = "L1_HANDLER")]
    LOneHandler,
}
pub type PendingDeployTransactionReceipt = HashMap<String, serde_json::Value>;
/// PendingCommonReceiptProperties
///
/// Common properties for a pending transaction receipt
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct PendingCommonReceiptProperties {
    pub transaction_hash: TransactionHash,
    pub actual_fee: ActualFee,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub _type: Option<TransactionType>,
    pub messages_sent: MessagesSent,
    pub events: Events,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum PendingTransactionReceipt {
    PendingDeployTransactionReceipt(PendingDeployTransactionReceipt),
    PendingCommonReceiptProperties(PendingCommonReceiptProperties),
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct DeprecatedContractClass {
    pub program: Program,
    pub entry_points_by_type: DeprecatedEntryPointsByType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abi: Option<ContractABI>,
}
/// GasConsumed
///
/// The Ethereum gas cost of the transaction (see https://docs.starknet.io/docs/Fees/fee-mechanism for more info)
pub type GasConsumed = String;
/// GasPrice
///
/// The gas price (in gwei) that was used in the cost estimation
pub type GasPrice = String;
/// OverallFee
///
/// The estimated fee for the transaction (in gwei), product of gas_consumed and gas_price
pub type OverallFee = String;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct FeeEstimation {
    pub gas_consumed: GasConsumed,
    pub gas_price: GasPrice,
    pub overall_fee: OverallFee,
}
/// False
///
/// only legal value is FALSE here
pub type False = bool;
/// StartingBlockHash
///
/// The hash of the block from which the sync started
pub type StartingBlockHash = String;
/// StartingBlockNumber
///
/// The number (height) of the block from which the sync started
pub type StartingBlockNumber = String;
/// CurrentBlockHash
///
/// The hash of the current block being synchronized
pub type CurrentBlockHash = String;
/// CurrentBlockNumber
///
/// The number (height) of the current block being synchronized
pub type CurrentBlockNumber = String;
/// HighestBlockHash
///
/// The hash of the estimated highest block to be synchronized
pub type HighestBlockHash = String;
/// HighestBlockNumber
///
/// The number (height) of the estimated highest block to be synchronized
pub type HighestBlockNumber = String;
/// SyncStatus
///
/// An object describing the node synchronization status
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct SyncStatus {
    pub starting_block_hash: StartingBlockHash,
    pub starting_block_num: StartingBlockNumber,
    pub current_block_hash: CurrentBlockHash,
    pub current_block_num: CurrentBlockNumber,
    pub highest_block_hash: HighestBlockHash,
    pub highest_block_num: HighestBlockNumber,
}
/// EventContext
///
/// The event emission information
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct EventContext {
    pub block_hash: BlockHash,
    pub block_number: BlockNumber,
    pub transaction_hash: TransactionHash,
}
/// EmittedEvent
///
/// Event information decorated with metadata on where it was emitted / An event emitted as a result
/// of transaction execution
pub type EmittedEvent = HashMap<String, serde_json::Value>;
pub type MatchingEvents = Vec<EmittedEvent>;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum BlockId {
    BlockHash(BlockHash),
    BlockNumber(BlockNumber),
    BlockTag(BlockTag),
}
/// StorageKey
///
/// A storage key. Represented as up to 62 hex digits, 3 bits, and 5 leading zeroes.
pub type StorageKey = String;
pub type Index = i64;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum StarknetGetBlockHashWithTxHashesResult {
    BlockWithTransactionHashes(BlockWithTransactionHashes),
    PendingBlockWithTransactionHashes(PendingBlockWithTransactionHashes),
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum StarknetGetBlockWithTxsResult {
    BlockWithTransactions(BlockWithTransactions),
    PendingBlockWithTransactions(PendingBlockWithTransactions),
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum StarknetGetStateUpdateResult {
    StateUpdate(StateUpdate),
    PendingStateUpdate(PendingStateUpdate),
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum TransactionReceipt {
    InvokeTransactionReceipt(InvokeTransactionReceipt),
    L1HandlerTransactionReceipt(L1HandlerTransactionReceipt),
    DeclareTransactionReceipt(DeclareTransactionReceipt),
    DeployTransactionReceipt(DeployTransactionReceipt),
    DeployAccountTransactionReceipt(DeployAccountTransactionReceipt),
    PendingTransactionReceipt(PendingTransactionReceipt),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum RPCContractClass {
    DeprecatedContractClass(DeprecatedContractClass),
    ContractClass(ContractClass),
}
pub type BlockTransactionCount = i64;
/// Estimation
///
/// a sequence of fee estimation where the i'th estimate corresponds to the i'th transaction
pub type Estimation = Vec<FeeEstimation>;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct BlockHashAndNumber {
    pub block_hash: BlockHash,
    pub block_number: BlockNumber,
}
/// ChainId
///
/// StarkNet chain id, given in hex representation.
pub type ChainId = String;
pub type PendingTransactions = Vec<Transaction>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum SyncingStatus {
    False(False),
    SyncStatus(SyncStatus),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct EventsChunk {
    pub events: MatchingEvents,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<ContinuationToken>,
}
