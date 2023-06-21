use blockifier::block_context::BlockContext;
use sp_core::U256;
use starknet_api::api_core::{ChainId, ContractAddress};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::hash::StarkFelt;
use starknet_api::stdlib::collections::HashMap;

use crate::execution::types::{ContractAddressWrapper, Felt252Wrapper};
use crate::traits::hash::HasherT;

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    Default,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
/// Starknet header definition.
pub struct Header {
    /// The hash of this block’s parent.
    pub parent_block_hash: Felt252Wrapper,
    /// The number (height) of this block.
    pub block_number: u64,
    /// The state commitment after this block.
    pub global_state_root: Felt252Wrapper,
    /// The Starknet address of the sequencer who created this block.
    pub sequencer_address: ContractAddressWrapper,
    /// The time the sequencer created this block before executing transactions
    pub block_timestamp: u64,
    /// The number of transactions in a block
    pub transaction_count: u128,
    /// A commitment to the transactions included in the block
    pub transaction_commitment: Felt252Wrapper,
    /// The number of events
    pub event_count: u128,
    /// A commitment to the events produced in this block
    pub event_commitment: Felt252Wrapper,
    /// The version of the Starknet protocol used when creating this block
    pub protocol_version: Option<u8>,
    /// Extraneous data that might be useful for running transactions
    pub extra_data: Option<U256>,
}

impl Header {
    /// Creates a new header.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        parent_block_hash: Felt252Wrapper,
        block_number: u64,
        global_state_root: Felt252Wrapper,
        sequencer_address: ContractAddressWrapper,
        block_timestamp: u64,
        transaction_count: u128,
        transaction_commitment: Felt252Wrapper,
        event_count: u128,
        event_commitment: Felt252Wrapper,
        protocol_version: Option<u8>,
        extra_data: Option<U256>,
    ) -> Self {
        Self {
            parent_block_hash,
            block_number,
            global_state_root,
            sequencer_address,
            block_timestamp,
            transaction_count,
            transaction_commitment,
            event_count,
            event_commitment,
            protocol_version,
            extra_data,
        }
    }

    /// Converts to a blockifier BlockContext
    pub fn into_block_context(self, fee_token_address: ContractAddressWrapper, chain_id: ChainId) -> BlockContext {
        // Convert from ContractAddressWrapper to ContractAddress
        let sequencer_address =
            ContractAddress::try_from(StarkFelt::new(self.sequencer_address.into()).unwrap()).unwrap();
        // Convert from ContractAddressWrapper to ContractAddress
        let fee_token_address = ContractAddress::try_from(StarkFelt::new(fee_token_address.into()).unwrap()).unwrap();

        BlockContext {
            chain_id,
            block_number: BlockNumber(self.block_number),
            block_timestamp: BlockTimestamp(self.block_timestamp),
            sequencer_address,
            vm_resource_fee_cost: HashMap::default(),
            fee_token_address,
            invoke_tx_max_n_steps: 1000000,
            validate_max_n_steps: 1000000,
            // FIXME: https://github.com/keep-starknet-strange/madara/issues/329
            gas_price: 10,
        }
    }

    /// Compute the hash of the header.
    #[must_use]
    pub fn hash<H: HasherT>(&self, hasher: H) -> Felt252Wrapper {
        let protocol_version = self.protocol_version.unwrap_or_default().into();

        let data: &[Felt252Wrapper] = &[
            self.block_number.into(), // TODO: remove unwrap
            self.global_state_root,
            self.sequencer_address,
            self.block_timestamp.into(),
            self.transaction_count.into(),
            self.transaction_commitment,
            self.event_count.into(),
            self.event_commitment,
            protocol_version,
            Felt252Wrapper::ZERO,
            self.parent_block_hash,
        ];

        <H as HasherT>::hash_elements(&hasher, data)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::crypto::hash::pedersen::PedersenHasher;
    #[test]
    fn test_header_hash() {
        // Values taken from genesis block on mainnet
        let hasher = PedersenHasher::default();

        let block_number = 86000;
        let block_timestamp = 1687235884;
        let global_state_root =
            Felt252Wrapper::from_hex_be("0x006727a7aae8c38618a179aeebccd6302c67ad5f8528894d1dde794e9ae0bbfa").unwrap();
        let parent_block_hash =
            Felt252Wrapper::from_hex_be("0x045543088ce763aba7db8f6bfb33e33cc50af5c2ed5a26d38d5071c352a49c1d").unwrap();
        let sequencer_address =
            Felt252Wrapper::from_hex_be("0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8").unwrap();
        let transaction_count = 197;
        let transaction_commitment =
            Felt252Wrapper::from_hex_be("0x70369cef825889dc005916dba67332b71f270b7af563d0433cee3342dda527d").unwrap();
        let event_count = 1430;
        let event_commitment =
            Felt252Wrapper::from_hex_be("0x2043ba1ef46882ce1dbb17b501fffa4b71f87f618e8f394e9605959d92efdf6").unwrap();
        let protocol_version = None;
        let extra_data = None;

        let header = Header::new(
            parent_block_hash,
            block_number,
            global_state_root,
            sequencer_address,
            block_timestamp,
            transaction_count,
            transaction_commitment,
            event_count,
            event_commitment,
            protocol_version,
            extra_data,
        );

        let expected_hash =
            Felt252Wrapper::from_hex_be("0x001d126ca058c7e546d59cf4e10728e4b023ca0fb368e8abcabf0b5335f4487a").unwrap();

        assert_eq!(header.hash(hasher), expected_hash);
    }

    #[test]
    fn test_to_block_context() {
        let sequencer_address = Felt252Wrapper::from_hex_be("0xFF").unwrap();
        // Create a block header.
        let block_header = Header { block_number: 1, block_timestamp: 1, sequencer_address, ..Default::default() };
        // Create a fee token address.
        let fee_token_address = Felt252Wrapper::from_hex_be("AA").unwrap();
        // Create a chain id.
        let chain_id = ChainId("0x1".to_string());
        // Try to serialize the block header.
        let block_context = block_header.into_block_context(fee_token_address, chain_id);
        let expected_sequencer_address =
            ContractAddress::try_from(StarkFelt::new(sequencer_address.into()).unwrap()).unwrap();
        let expected_fee_token_address =
            ContractAddress::try_from(StarkFelt::new(fee_token_address.into()).unwrap()).unwrap();
        // Check that the block context was serialized correctly.
        assert_eq!(block_context.block_number, BlockNumber(1));
        assert_eq!(block_context.block_timestamp, BlockTimestamp(1));
        assert_eq!(block_context.sequencer_address, expected_sequencer_address);
        assert_eq!(block_context.fee_token_address, expected_fee_token_address);
    }
}
