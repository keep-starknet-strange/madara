use blockifier::blockifier::block::GasPrices;
use mp_felt::Felt252Wrapper;
use mp_hashers::pedersen::PedersenHasher;
use mp_hashers::HasherT;
use sp_core::U256;
use starknet_api::core::ContractAddress;
use starknet_api::hash::StarkHash;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
// #[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
/// Starknet header definition.
pub struct Header {
    /// The hash of this block’s parent.
    pub parent_block_hash: StarkHash,
    /// The number (height) of this block.
    pub block_number: u64,
    /// The Starknet address of the sequencer who created this block.
    pub sequencer_address: ContractAddress,
    /// The time the sequencer created this block before executing transactions
    pub block_timestamp: u64,
    /// The number of transactions in a block
    pub transaction_count: u128,
    /// The number of events
    pub event_count: u128,
    /// The version of the Starknet protocol used when creating this block
    pub protocol_version: u8,
    /// Gas prices for this block
    pub l1_gas_price: GasPrices,
    /// Extraneous data that might be useful for running transactions
    pub extra_data: Option<U256>,
}

impl Header {
    /// Creates a new header.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        parent_block_hash: StarkHash,
        block_number: u64,
        sequencer_address: ContractAddress,
        block_timestamp: u64,
        transaction_count: u128,
        event_count: u128,
        protocol_version: u8,
        gas_prices: GasPrices,
        extra_data: Option<U256>,
    ) -> Self {
        Self {
            parent_block_hash,
            block_number,
            sequencer_address,
            block_timestamp,
            transaction_count,
            event_count,
            protocol_version,
            l1_gas_price: gas_prices,
            extra_data,
        }
    }

    /// Compute the hash using the Pedersen hasher according to [the Starknet protocol specification](https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/header/#block_hash).  
    pub fn hash(&self) -> Felt252Wrapper {
        let data = [
            self.block_number.into(),
            self.sequencer_address.0.0.into(),
            self.block_timestamp.into(),
            self.transaction_count.into(),
            self.event_count.into(),
            self.protocol_version.into(),
            Felt252Wrapper::ZERO,
            self.parent_block_hash.into(),
        ];

        PedersenHasher::compute_hash_on_wrappers(data.into_iter())
    }
}
