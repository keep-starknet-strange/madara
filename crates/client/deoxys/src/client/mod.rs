//! Defines a generic implementation of a Starknet JSON-RPC server client.

use starknet_core::types::requests::*;
use starknet_core::types::*;

use self::json_rpc::{JsonRpcClient, JsonRpcClientError};

pub mod json_rpc;

// For some reason, `starknet-core` does not define a type that requests the spec version.
// Let's do it ourselves.
struct SpecVersionRequest;

impl self::json_rpc::Request for SpecVersionRequest {
    const METHOD: &'static str = "starknet_specVersion";
    type Response = Box<str>;
    type Params = [(); 0];
    #[inline(always)]
    fn into_params(self) -> Self::Params {
        [(); 0]
    }
}

/// A generic implementation of a Starknet JSON-RPC client.
pub struct StarknetClient<T> {
    inner: JsonRpcClient<T>,
}

impl<T> StarknetClient<T> {
    /// Creates a new [`StarknetClient`] with the given transport layer.
    pub fn new(transport: T) -> Self {
        Self { inner: JsonRpcClient::new(transport) }
    }
}

impl<T: json_rpc::Transport> StarknetClient<T> {
    /// Requets the version of the Starknet JSON-RPC specification being used by the server.
    pub async fn spec_version(&self) -> Result<Box<str>, JsonRpcClientError<T::Error>> {
        self.inner.request(SpecVersionRequest).await
    }

    /// Returns information about the specified block.
    ///
    /// Transactions are not fully sent in the response. Instead, only their hashes are
    /// communicated.
    pub async fn get_block_with_tx_hashes(
        &self,
        block_id: BlockId,
    ) -> Result<starknet_core::types::MaybePendingBlockWithTxHashes, JsonRpcClientError<T::Error>> {
        self.inner.request(GetBlockWithTxHashesRequest { block_id }).await
    }

    /// Returns information about the specified block.
    ///
    /// Transactions are fully sent in the response. If you do not need the full transactions,
    /// consider using [`get_block_with_tx_hashes`](Self::get_block_with_tx_hashes) instead.
    pub async fn get_block_with_txs(
        &self,
        block_id: BlockId,
    ) -> Result<MaybePendingBlockWithTxs, JsonRpcClientError<T::Error>> {
        self.inner.request(GetBlockWithTxsRequest { block_id }).await
    }

    /// Returns information about the result of executing the specified block.
    pub async fn get_state_update(
        &self,
        block_id: BlockId,
    ) -> Result<MaybePendingStateUpdate, JsonRpcClientError<T::Error>> {
        self.inner.request(GetStateUpdateRequest { block_id }).await
    }

    /// Returns the value of the storage cell at the specified address.
    ///
    /// # Arguments
    ///
    /// - `contract_address`: The address of the contract to read from.
    ///
    /// - `key`: The key of the storage cell to read within the given contract.
    ///
    /// - `block_id`: The block to read from.
    pub async fn get_storage_at(
        &self,
        contract_address: FieldElement,
        key: FieldElement,
        block_id: BlockId,
    ) -> Result<FieldElement, JsonRpcClientError<T::Error>> {
        self.inner.request(GetStorageAtRequest { contract_address, key, block_id }).await
    }

    // TODO:
    //  Include the `starknet_getTransactionStatus` method here. For some reason it's not
    //  defined by `starknet-core`.

    /// Returns information about the specified transaction.
    pub async fn get_transaction_by_hash(
        &self,
        transaction_hash: FieldElement,
    ) -> Result<Transaction, JsonRpcClientError<T::Error>> {
        self.inner.request(GetTransactionByHashRequest { transaction_hash }).await
    }

    /// Returns information about the transaction at the specified index in the specified block.
    pub async fn get_transaction_by_block_id_and_index(
        &self,
        block_id: BlockId,
        index: u64,
    ) -> Result<Transaction, JsonRpcClientError<T::Error>> {
        self.inner.request(GetTransactionByBlockIdAndIndexRequest { block_id, index }).await
    }

    /// Returns the transaction receipt for the specified transaction.
    pub async fn get_transaction_receipt(
        &self,
        transaction_hash: FieldElement,
    ) -> Result<TransactionReceipt, JsonRpcClientError<T::Error>> {
        self.inner.request(GetTransactionReceiptRequest { transaction_hash }).await
    }

    /// Returns the contract definition for the specified class.
    ///
    /// # Arguments
    ///
    /// - `block_id`: The block to read from.
    ///
    /// - `class_hash`: The hash of the requested contract class.
    pub async fn get_class(
        &self,
        block_id: BlockId,
        class_hash: FieldElement,
    ) -> Result<ContractClass, JsonRpcClientError<T::Error>> {
        self.inner.request(GetClassRequest { block_id, class_hash }).await
    }

    /// Returns the class hash of the specified contract address.
    ///
    /// # Arguments
    ///
    /// - `block_id`: The block to read from.
    ///
    /// - `contract_address`: The subject contract address.
    pub async fn get_class_hash_at(
        &self,
        block_id: BlockId,
        contract_address: FieldElement,
    ) -> Result<FieldElement, JsonRpcClientError<T::Error>> {
        self.inner.request(GetClassHashAtRequest { block_id, contract_address }).await
    }

    /// Returns the contract definition for the specified class.
    ///
    /// # Arguments
    ///
    /// - `block_id`: The block to read from.
    ///
    /// - `contract_address`: The subject contract address.
    pub async fn get_class_at(
        &self,
        block_id: BlockId,
        contract_address: FieldElement,
    ) -> Result<ContractClass, JsonRpcClientError<T::Error>> {
        self.inner.request(GetClassAtRequest { block_id, contract_address }).await
    }

    /// Returns the number of transactions in the specified block.
    pub async fn get_block_transaction_count(&self, block_id: BlockId) -> Result<u64, JsonRpcClientError<T::Error>> {
        self.inner.request(GetBlockTransactionCountRequest { block_id }).await
    }

    /// Calls the specified function.
    ///
    /// # Arguments
    ///
    /// - `contract_address`: The address of the contract to call.
    ///
    /// - `selector`: The selector of the function to call.
    ///
    /// - `calldata`: The calldata to pass to the function.
    ///
    /// - `block_id`: The block referencing the state to use for the call.
    ///
    /// # Returns
    ///
    /// The return value of the function.
    pub async fn call(
        &self,
        contract_address: FieldElement,
        selector: FieldElement,
        calldata: Vec<FieldElement>,
        block_id: BlockId,
    ) -> Result<FieldElement, JsonRpcClientError<T::Error>> {
        self.inner
            .request(CallRequest {
                request: FunctionCall { contract_address, entry_point_selector: selector, calldata },
                block_id,
            })
            .await
    }

    /// Estimates the cost of the specified StarkNet transactions.
    ///
    /// # Arguments
    ///
    /// - `request`: The transactions to estimate the cost of.
    ///
    /// - `block_id`: The block referencing the state to use for the estimation.
    pub async fn estimate_fee(
        &self,
        request: Vec<BroadcastedTransaction>,
        block_id: BlockId,
    ) -> Result<FeeEstimate, JsonRpcClientError<T::Error>> {
        self.inner.request(EstimateFeeRequest { request, block_id }).await
    }

    /// Estimates the resources required by the l1_handler transaction induced by the provided
    /// message.
    ///
    /// # Arguments
    ///
    /// - `message`: The message to estimate the resources of.
    ///
    /// - `block_id`: The block referencing the state to use for the estimation.
    pub async fn estimate_message_fee(
        &self,
        message: MsgFromL1,
        block_id: BlockId,
    ) -> Result<FeeEstimate, JsonRpcClientError<T::Error>> {
        self.inner.request(EstimateMessageFeeRequest { message, block_id }).await
    }

    /// Returns the number (height) of the latest block.
    pub async fn block_number(&self) -> Result<u64, JsonRpcClientError<T::Error>> {
        self.inner.request(BlockNumberRequest).await
    }

    /// Returns the hash and number of the latest block.
    pub async fn block_hash_and_number(&self) -> Result<(FieldElement, u64), JsonRpcClientError<T::Error>> {
        match self.inner.request(BlockHashAndNumberRequest).await {
            Ok(BlockHashAndNumber { block_hash, block_number }) => Ok((block_hash, block_number)),
            Err(err) => Err(err),
        }
    }

    /// Returns the current syncing status of the node.
    ///
    /// # Returns
    ///
    /// - `None` if the node is not syncing.
    ///
    /// - `Some(status)` if the node is syncing. In that case, `status` is the state of the
    ///   syncronization operation.
    pub async fn syncing(&self) -> Result<Option<SyncStatus>, JsonRpcClientError<T::Error>> {
        match self.inner.request(SyncingRequest).await {
            Ok(SyncStatusType::NotSyncing) => Ok(None),
            Ok(SyncStatusType::Syncing(status)) => Ok(Some(status)),
            Err(err) => Err(err),
        }
    }

    /// Returns the events matching the provided filter.
    pub async fn get_events_manually(
        &self,
        filter: EventFilterWithPage,
    ) -> Result<EventsPage, JsonRpcClientError<T::Error>> {
        self.inner.request(GetEventsRequest { filter }).await
    }

    /// Returns an "iterator" that can be used to gets the events matching the provided filter.
    pub fn get_events(&self, filter: EventFilter, chunk_size: u64) -> Events<'_, T> {
        Events { inner: self, continuation_token: None, filter, chunk_size }
    }

    /// Returns the nonce associated with the specified contract.
    /// 
    /// # Arguments
    /// 
    /// - `contract_address`: The address of the contract to get the nonce of.
    /// 
    /// - `block_id`: The block to read from.
    pub async fn get_nonce(&self, contract_address: FieldElement, block_id: BlockId) -> Result<FieldElement, JsonRpcClientError<T::Error>> {
        self.inner.request(GetNonceRequest {
            contract_address,
            block_id
        }).await
    }
}

/// An "iterator" that can be used to gets the events matching the provided filter.
///
/// This structure automatically handles pagination.
pub struct Events<'a, T> {
    inner: &'a StarknetClient<T>,
    continuation_token: Option<String>,

    // IMPROVE(nils-mathieu):
    //  Cloning this value is not very efficient. We can avoid it by creating a custom "EventFilter"
    //  struct that contains references rather than owned values. That would be make the API a bit
    //  harder to use though.
    filter: EventFilter,

    /// The size of the requested pages.
    chunk_size: u64,
}

impl<'a, T: json_rpc::Transport> Events<'a, T> {
    /// Returns the next batch of events.
    pub async fn next_page(&mut self) -> Result<Vec<EmittedEvent>, JsonRpcClientError<T::Error>> {
        let events = self
            .inner
            .get_events_manually(EventFilterWithPage {
                event_filter: self.filter.clone(),
                result_page_request: ResultPageRequest {
                    continuation_token: self.continuation_token.clone(),
                    chunk_size: self.chunk_size,
                },
            })
            .await?;

        self.continuation_token = events.continuation_token;

        Ok(events.events)
    }
}
