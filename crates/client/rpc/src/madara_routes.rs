use jsonrpsee::core::{async_trait, RpcResult};
use log::error;
use sc_block_builder::GetPendingBlockExtrinsics;
use mc_genesis_data_provider::GenesisProvider;
pub use mc_rpc_core::{
    Felt, MadaraRpcApiServer, PredeployedAccountWithBalance, StarknetReadRpcApiServer, StarknetTraceRpcApiServer,
    StarknetWriteRpcApiServer,
};
use mp_felt::Felt252Wrapper;
use mp_hashers::HasherT;
use mp_transactions::from_broadcasted_transactions::try_declare_tx_from_broadcasted_declare_tx_v0;
use mp_transactions::BroadcastedDeclareTransactionV0;
use pallet_starknet_runtime_api::{ConvertTransactionRuntimeApi, StarknetRuntimeApi};
use sc_client_api::backend::Backend;
use sc_client_api::{BlockBackend, StorageProvider};
use sc_transaction_pool::ChainApi;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use starknet_core::types::{BlockId, BlockTag, DeclareTransactionResult, FieldElement, FunctionCall};
use starknet_core::utils::get_selector_from_name;

use crate::errors::StarknetRpcApiError;
use crate::Starknet;

#[async_trait]
impl<A, B, BE, G, C, P, H> MadaraRpcApiServer for Starknet<A, B, BE, G, C, P, H>
where
    A: ChainApi<Block = B> + 'static,
    B: BlockT,
    BE: Backend<B> + 'static,
    C: HeaderBackend<B> + BlockBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
    C: GetPendingBlockExtrinsics<B>,
    G: GenesisProvider + Send + Sync + 'static,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
    P: TransactionPool<Block = B> + 'static,
    H: HasherT + Send + Sync + 'static,
{
    fn predeployed_accounts(&self) -> RpcResult<Vec<PredeployedAccountWithBalance>> {
        let genesis_data = self.genesis_provider.load_genesis_data()?;
        let block_id = BlockId::Tag(BlockTag::Latest);
        let fee_token_address: FieldElement = genesis_data.eth_fee_token_address.0;

        Ok(genesis_data
            .predeployed_accounts
            .into_iter()
            .map(|account| {
                let contract_address: FieldElement = account.contract_address.into();
                let balance_string = &self
                    .call(
                        FunctionCall {
                            contract_address: fee_token_address,
                            entry_point_selector: get_selector_from_name("balanceOf")
                                .expect("the provided method name should be a valid ASCII string."),
                            calldata: vec![contract_address],
                        },
                        block_id,
                    )
                    .expect("FunctionCall attributes should be correct.")[0];
                let balance =
                    Felt252Wrapper::from_hex_be(balance_string).expect("`balanceOf` should return a Felt").into();
                PredeployedAccountWithBalance { account, balance }
            })
            .collect::<Vec<_>>())
    }

    async fn add_declare_transaction_v0(
        &self,
        declare_transaction: BroadcastedDeclareTransactionV0,
    ) -> RpcResult<DeclareTransactionResult> {
        let chain_id = Felt252Wrapper(self.chain_id()?.0);

        let transaction =
            try_declare_tx_from_broadcasted_declare_tx_v0(declare_transaction, chain_id).map_err(|e| {
                error!("Failed to convert BroadcastedDeclareTransactionV0 to DeclareTransaction, error: {e}");
                StarknetRpcApiError::InternalServerError
            })?;

        let (tx_hash, class_hash) = self.declare_tx_common(transaction).await?;

        Ok(DeclareTransactionResult {
            transaction_hash: Felt252Wrapper::from(tx_hash).into(),
            class_hash: Felt252Wrapper::from(class_hash).into(),
        })
    }
}
