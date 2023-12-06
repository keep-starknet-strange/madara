pub mod client;
pub mod errors;

use async_trait::async_trait;
pub use client::StarknetContractClient;
use ethers::types::U256;
use mp_snos_output::{SnosCodec, StarknetOsOutput};
use sp_runtime::traits::Block;
use starknet_api::hash::StarkFelt;

use crate::{Result, SettlementProvider, StarknetSpec, StarknetState};

pub fn convert_u256_to_felt<B: Block>(word: U256) -> Result<StarkFelt, B> {
    let mut bytes = [0u8; 32];
    word.to_big_endian(bytes.as_mut_slice());
    Ok(StarkFelt::new(bytes)?)
}

pub fn convert_felt_to_u256(felt: StarkFelt) -> U256 {
    U256::from_big_endian(felt.bytes())
}

#[async_trait]
impl<B: Block> SettlementProvider<B> for StarknetContractClient {
    async fn is_initialized(&self) -> Result<bool, B> {
        Ok(U256::zero() != self.program_hash().await?)
    }

    async fn get_chain_spec(&self) -> Result<StarknetSpec, B> {
        Ok(StarknetSpec {
            program_hash: convert_u256_to_felt(self.program_hash().await?)?,
            config_hash: convert_u256_to_felt(self.config_hash().await?)?,
        })
    }

    async fn get_state(&self) -> Result<StarknetState, B> {
        Ok(StarknetState {
            block_number: convert_u256_to_felt(self.state_block_number().await?.into_raw())?,
            state_root: convert_u256_to_felt(self.state_root().await?)?,
        })
    }

    async fn update_state(&self, program_output: StarknetOsOutput) -> Result<(), B> {
        let program_output: Vec<U256> =
            program_output.into_encoded_vec().into_iter().map(convert_felt_to_u256).collect();

        let tx_receipt = self.update_state(program_output).await?;
        log::trace!("[settlement] State was successfully updated: {:#?}", tx_receipt);

        Ok(())
    }
}
