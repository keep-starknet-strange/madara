use blockifier::block_context::BlockContext;
use scale_codec::Encode;
use sp_core::U256;
use starknet_api::api_core::{ChainId, ContractAddress};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::hash::StarkFelt;
use starknet_api::stdlib::collections::HashMap;

use crate::alloc::string::ToString;
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
    pub block_number: U256,
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
        block_number: U256,
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
    pub fn into_block_context(self, fee_token_address: ContractAddressWrapper) -> BlockContext {
        // Convert from ContractAddressWrapper to ContractAddress
        let sequencer_address =
            ContractAddress::try_from(StarkFelt::new(self.sequencer_address.into()).unwrap()).unwrap();
        // Convert from ContractAddressWrapper to ContractAddress
        let fee_token_address = ContractAddress::try_from(StarkFelt::new(fee_token_address.into()).unwrap()).unwrap();

        BlockContext {
            chain_id: ChainId("SN_GOERLI".to_string()),
            block_number: BlockNumber(self.block_number.as_u64()),
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
        <H as HasherT>::hash(&hasher, &self.block_number.encode())
    }
}

#[test]
fn test_header_hash() {
    let parent_block_hash = Felt252Wrapper::try_from(&[1; 32]).unwrap();
    let block_number = U256::from(42);
    let global_state_root = Felt252Wrapper::from(12345_u128);
    let sequencer_address = Felt252Wrapper::try_from(&[2; 32]).unwrap();
    let block_timestamp = 1620037184;
    let transaction_count = 2;
    let transaction_commitment = Felt252Wrapper::try_from(&[3; 32]).unwrap();
    let event_count = 1;
    let event_commitment = Felt252Wrapper::try_from(&[4; 32]).unwrap();
    let protocol_version = Some(1);
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

    let hasher = crate::crypto::hash::pedersen::PedersenHasher::default();

    let expected_hash = hasher.hash(&block_number.encode());

    assert_eq!(header.hash(hasher), expected_hash);
}

#[test]
fn test_to_block_context() {
    let sequencer_address = Felt252Wrapper::from_hex_be("0xFF").unwrap();
    // Create a block header.
    let block_header = Header { block_number: 1.into(), block_timestamp: 1, sequencer_address, ..Default::default() };
    // Create a fee token address.
    let fee_token_address = Felt252Wrapper::from_hex_be("AA").unwrap();
    // Try to serialize the block header.
    let block_context = block_header.into_block_context(fee_token_address);
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
