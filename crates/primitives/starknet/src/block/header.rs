use scale_codec::Encode;
use sp_core::{H256, U256};

use crate::execution::types::{ContractAddressWrapper, Felt252Wrapper};
use crate::traits::hash::Hasher;

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

    /// Compute the hash of the header.
    #[must_use]
    pub fn hash<H: Hasher>(&self, hasher: H) -> H256 {
        H256::from_slice(<H as Hasher>::hash(&hasher, &self.block_number.encode()).as_slice())
    }
}

#[test]
fn test_header_hash() {
    let parent_block_hash = H256::from([1; 32]).into();
    let block_number = U256::from(42);
    let global_state_root = Felt252Wrapper(U256::from(12345));
    let sequencer_address = [2; 32].into();
    let block_timestamp = 1620037184;
    let transaction_count = 2;
    let transaction_commitment = H256::from([3; 32]).into();
    let event_count = 1;
    let event_commitment = H256::from([4; 32]).into();
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

    let expected_hash = H256::from_slice(hasher.hash(&block_number.encode()).as_slice());

    assert_eq!(header.hash(hasher), expected_hash);
}
