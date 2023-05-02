//! Starknet RPC server API implementation
//!
//! It uses the madara client and backend in order to answer queries.

mod errors;
mod madara_backend_client;

use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;

use errors::StarknetRpcApiError;
use hex::FromHex;
use jsonrpsee::core::RpcResult;
use log::error;
pub use mc_rpc_core::StarknetRpcApiServer;
use mc_rpc_core::{
    BlockHashAndNumber, BlockId as StarknetBlockId, ContractAddress, FieldElement, FunctionCall, RPCContractClass,
    SierraContractClass,
};
use mc_storage::OverrideHandle;
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use sc_client_api::backend::{Backend, StorageProvider};
use sp_api::{ApiError, ProvideRuntimeApi};
use sp_arithmetic::traits::UniqueSaturatedInto;
use sp_blockchain::HeaderBackend;
use sp_core::{H256, U256};
use sp_runtime::traits::Block as BlockT;
use starknet_api::hash::StarkFelt;

/// A Starknet RPC server for Madara
pub struct Starknet<B: BlockT, BE, C> {
    client: Arc<C>,
    backend: Arc<mc_db::Backend<B>>,
    overrides: Arc<OverrideHandle<B>>,
    _marker: PhantomData<(B, BE)>,
}

impl<B: BlockT, BE, C> Starknet<B, BE, C> {
    pub fn new(client: Arc<C>, backend: Arc<mc_db::Backend<B>>, overrides: Arc<OverrideHandle<B>>) -> Self {
        Self { client, backend, overrides, _marker: PhantomData }
    }
}

impl<B, BE, C> Starknet<B, BE, C>
where
    B: BlockT,
    C: HeaderBackend<B> + 'static,
{
    pub fn current_block_number(&self) -> RpcResult<u64> {
        Ok(UniqueSaturatedInto::<u64>::unique_saturated_into(self.client.info().best_number))
    }
}

impl<B, BE, C> Starknet<B, BE, C>
where
    B: BlockT,
    C: HeaderBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
    BE: Backend<B>,
{
    pub fn current_block_hash(&self) -> Result<H256, ApiError> {
        let substrate_block_hash = self.client.info().best_hash;

        let block = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .current_block(substrate_block_hash)
            .unwrap_or_default();

        Ok(block.header().hash())
    }

    /// Returns the substrate block corresponding to the given Starknet block id
    fn substrate_block_hash_from_starknet_block(&self, block_id: StarknetBlockId) -> Result<B::Hash, String> {
        match block_id {
            StarknetBlockId::BlockHash(h) => madara_backend_client::load_hash(
                self.client.as_ref(),
                &self.backend,
                H256::from_str(&h).map_err(|e| format!("Failed to convert '{h}' to H256: {e}"))?,
            )
            .map_err(|e| format!("Failed to load Starknet block hash for Substrate block with hash '{h}': {e}"))?,
            StarknetBlockId::BlockNumber(n) => self
                .client
                .hash(UniqueSaturatedInto::unique_saturated_into(n))
                .map_err(|e| format!("Failed to retrieve the hash of block number '{n}': {e}"))?,
            StarknetBlockId::BlockTag(t) => match t {
                mc_rpc_core::BlockTag::Latest => Some(self.client.info().best_hash),
                mc_rpc_core::BlockTag::Pending => None,
            },
        }
        .ok_or("Failed to retrieve the substrate block id".to_string())
    }
}

impl<B, BE, C> StarknetRpcApiServer for Starknet<B, BE, C>
where
    B: BlockT,
    BE: Backend<B> + 'static,
    C: HeaderBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B>,
{
    fn block_number(&self) -> RpcResult<mc_rpc_core::BlockNumber> {
        self.current_block_number()
    }

    fn block_hash_and_number(&self) -> RpcResult<mc_rpc_core::BlockHashAndNumber> {
        let block_number = self.current_block_number()?;
        let block_hash = self.current_block_hash().map_err(|e| {
            error!("Failed to retrieve the current block hash: {}", e);
            StarknetRpcApiError::NoBlocks
        })?;

        Ok(BlockHashAndNumber { block_hash: format!("{:#x}", block_hash), block_number })
    }

    fn get_block_transaction_count(&self, block_id: StarknetBlockId) -> RpcResult<u128> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let block = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .current_block(substrate_block_hash)
            .unwrap_or_default();

        Ok(block.header().transaction_count)
    }

    fn call(&self, request: FunctionCall, block_id: StarknetBlockId) -> RpcResult<Vec<String>> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let runtime_api = self.client.runtime_api();

        let calldata: Result<Vec<U256>, StarknetRpcApiError> = request
            .calldata
            .iter()
            .map(|x| {
                U256::from_str(x).map_err(|e| {
                    error!("Calldata: Failed to convert '{x}' to U256: {e}");
                    StarknetRpcApiError::InvalidCallData
                })
            })
            .collect();

        match calldata {
            Ok(calldata) => {
                let result = runtime_api
                    .call(
                        substrate_block_hash,
                        <[u8; 32]>::from_hex(remove_prefix(&request.contract_address)).map_err(|e| {
                            error!("Address: Failed to convert '{0}' to [u8; 32]: {e}", request.contract_address);
                            StarknetRpcApiError::BlockNotFound
                        })?,
                        string_to_h256(&request.entry_point_selector).map_err(|e| {
                            error!("Entrypoint: Failed to convert '{0}' to H256: {e}", request.entry_point_selector);
                            StarknetRpcApiError::BlockNotFound
                        })?,
                        calldata,
                    )
                    .map_err(|e| {
                        error!("Request parameters error: {e}");
                        StarknetRpcApiError::InternalServerError
                    })?
                    .map_err(|e| {
                        error!("Failed to call function: {:#?}", e);
                        StarknetRpcApiError::ContractError
                    })?;
                Ok(result.iter().map(|x| format!("{:#x}", x)).collect())
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Get the contract class at a given contract address for a given block id
    fn get_class_at(
        &self,
        contract_address: ContractAddress,
        block_id: StarknetBlockId,
    ) -> RpcResult<RPCContractClass> {
        // TODO: Fix it when we deal with cairo 1.
        let _substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let _contract_address_wrapped = <[u8; 32]>::from_hex(remove_prefix(&contract_address)).map_err(|e| {
            error!("Failed to convert '{contract_address}' to array: {e}");
            StarknetRpcApiError::ContractNotFound
        })?;

        // let contract_class: SierraContractClass = self
        //     .overrides
        //     .for_block_hash(self.client.as_ref(), substrate_block_hash)
        //     .contract_class(substrate_block_hash, contract_address_wrapped)
        //     .ok_or_else(|| {
        //         error!("Failed to retrieve contract class at '{contract_address}'");
        //         StarknetRpcApiError::ContractNotFound
        //     })?
        //     .try_into()
        //     .map_err(|e| {
        //         error!(
        //             "Failed to convert `ContractClassWrapper` at '{contract_address}' to
        // `ContractClass`: {e}"
        //         );
        //     StarknetRpcApiError::ContractNotFound
        // })?;

        Ok(RPCContractClass::ContractClass(SierraContractClass::default()))
    }

    /// Get the contract class hash in the given block for the contract deployed at the given
    /// address
    ///
    /// # Arguments
    ///
    /// * `block_id` - The hash of the requested block, or number (height) of the requested block,
    ///   or a block tag
    /// * `contract_address` - The address of the contract whose class hash will be returned
    ///
    /// # Returns
    ///
    /// * `class_hash` - The class hash of the given contract
    fn get_class_hash_at(
        &self,
        block_id: StarknetBlockId,
        contract_address: ContractAddress,
    ) -> RpcResult<FieldElement> {
        let substrate_block_hash = self.substrate_block_hash_from_starknet_block(block_id).map_err(|e| {
            error!("'{e}'");
            StarknetRpcApiError::BlockNotFound
        })?;

        let contract_address_wrapped = <[u8; 32]>::from_hex(remove_prefix(&contract_address)).map_err(|e| {
            error!("Failed to convert '{contract_address}' to array: {e}");
            StarknetRpcApiError::ContractNotFound
        })?;

        let class_hash = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .contract_class_hash_by_address(substrate_block_hash, contract_address_wrapped)
            .ok_or_else(|| {
                error!("Failed to retrieve contract class hash at '{contract_address}'");
                StarknetRpcApiError::ContractNotFound
            })?;

        Ok(StarkFelt::new(class_hash)
            .map_err(|e| {
                error!("Failed to convert contract class hash at '{contract_address}': {e}");
                StarknetRpcApiError::ContractNotFound
            })?
            .to_string())
    }
}

/// Removes the "0x" prefix from a given hexadecimal string
fn remove_prefix(input: &str) -> &str {
    input.strip_prefix("0x").unwrap_or(input)
}

/// Converts a hexadecimal string to an H256 value, padding with zero bytes on the left if necessary
fn string_to_h256(hex_str: &str) -> Result<H256, String> {
    let hex_str = remove_prefix(hex_str);
    let mut padded_hex_str = hex_str.to_string();
    while padded_hex_str.len() < 64 {
        padded_hex_str.insert(0, '0');
    }
    let bytes =
        Vec::from_hex(&padded_hex_str).map_err(|e| format!("Failed to convert hex string to bytes: {:?}", e))?;
    Ok(H256::from_slice(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test case for the string_to_h256 function
    #[test]
    fn test_string_to_h256() {
        // Test case 1: Valid input with 64 characters
        let hex_str_1 = "0x0222882e457847df7ebaf981db2ff8ebb22c19d5b0a6a41dcc13cc2d775fbeb7";
        let expected_result_1 = H256::from_str(hex_str_1).unwrap();
        assert_eq!(string_to_h256(hex_str_1).unwrap(), expected_result_1);

        // Test case 2: Input with leading zeros
        let hex_str_2 = "0x0123456789abcdef";
        let expected_result_2 =
            H256::from_str("0x0000000000000000000000000000000000000000000000000123456789abcdef").unwrap();
        assert_eq!(string_to_h256(hex_str_2).unwrap(), expected_result_2);

        // Test case 3: Input with missing "0x" prefix
        let hex_str_3 = "222882e457847df7ebaf981db2ff8ebb22c19d5b0a6a41dcc13cc2d775fbeb7";
        let expected_result_3 =
            H256::from_str("0x0222882e457847df7ebaf981db2ff8ebb22c19d5b0a6a41dcc13cc2d775fbeb7").unwrap();
        assert_eq!(string_to_h256(hex_str_3).unwrap(), expected_result_3);

        // Test case 4: Input with invalid length
        let hex_str_4 = "0x222882e457847df7ebaf981db2ff8ebb22c19d5b0a6a41dcc13cc2d775fbeb7111111";
        assert!(string_to_h256(hex_str_4).is_err());
    }
}
