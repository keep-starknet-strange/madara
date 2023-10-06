use std::io::Read;

use starknet_api::api_core::EthAddress;
use starknet_api::hash::StarkFelt;

use crate::codec::SnosCodec;
use crate::conversions::eth_address_to_felt;
use crate::{MessageL1ToL2, MessageL2ToL1, StarknetOsOutput};

trait Decode {
    fn decode<I: Read>(input: &mut I) -> Self;
}

fn segment_decode<T: Decode + SnosCodec, I: Read>(input: &mut I) -> Vec<T> {
    let mut segment_len = u64::decode(input) as usize;
    let mut items: Vec<T> = Vec::new();
    while segment_len > 0 {
        let item = T::decode(input);
        segment_len -= item.size_hint();
        items.push(item);
    }
    items
}

impl Decode for Vec<StarkFelt> {
    fn decode<I: Read>(input: &mut I) -> Self {
        let n_items = u64::decode(input);
        let mut items: Vec<StarkFelt> = Vec::with_capacity(n_items as usize);
        for _ in 0..n_items {
            items.push(StarkFelt::decode(input));
        }
        items
    }
}

impl Decode for u64 {
    fn decode<I: Read>(input: &mut I) -> Self {
        let mut bytes = [0u8; 32];
        input.read_exact(bytes.as_mut_slice()).unwrap();
        u64::from_be_bytes(bytes[24..].try_into().unwrap())
    }
}

impl Decode for StarkFelt {
    fn decode<I: Read>(input: &mut I) -> Self {
        let mut bytes = [0u8; 32];
        input.read_exact(bytes.as_mut_slice()).unwrap();
        Self(bytes)
    }
}

impl Decode for MessageL2ToL1 {
    fn decode<I: Read>(input: &mut I) -> Self {
        Self {
            from_address: StarkFelt::decode(input),
            to_address: StarkFelt::decode(input),
            payload: Vec::<StarkFelt>::decode(input),
        }
    }
}

impl Decode for MessageL1ToL2 {
    fn decode<I: Read>(input: &mut I) -> Self {
        Self {
            from_address: StarkFelt::decode(input),
            to_address: StarkFelt::decode(input),
            nonce: StarkFelt::decode(input),
            selector: StarkFelt::decode(input),
            payload: Vec::<StarkFelt>::decode(input),
        }
    }
}

impl Decode for StarknetOsOutput {
    fn decode<I: Read>(input: &mut I) -> Self {
        Self {
            prev_state_root: StarkFelt::decode(input),
            new_state_root: StarkFelt::decode(input),
            block_number: StarkFelt::decode(input),
            block_hash: StarkFelt::decode(input),
            config_hash: StarkFelt::decode(input),
            messages_to_l1: segment_decode(input),
            messages_to_l2: segment_decode(input),
        }
    }
}

// Starknet::update_state sample invocation from mainnet
// https://etherscan.io/tx/0x9a6f9ee53f0b558f466d4340613740b9483e10c230313aa9c31fd0ba80f1a40f
//
// Calldata:
// "0x0000000000000000000000000000000000000000000000000000000000000060",  programOutput offset (96
// bytes) "0x64f464be0437d366556e4fe7cfc0fc8d2eec0ed531050137ca44052de9c97219",  onchainDataHash
// "0x0000000000000000000000000000000000000000000000000000000000000816",  onchainDataSize
// "0x0000000000000000000000000000000000000000000000000000000000000016",  programOutput length
const SNOS_PROGRAM_OUTPUT_HEX: &str = "\
    00bf8721ac2af6f7f40155c973c2bf5c15b7e0ed790b0865af20bf25ab57e9ff\
    03d46b43f31ccfed7ce09a0e318f1f98f59b28f4527ea01de34382ab8c7f2a26\
    00000000000000000000000000000000000000000000000000000000000441ee\
    0770ab05ba02edc49516cebde84bbffb76da74cdc98fd142c9c703ab871c4c7a\
    017c0bc29d31e9a7d14671610a7626264ce9ce8e3ed066a4775adf9b123de9dd\
    0000000000000000000000000000000000000000000000000000000000000007\
    073314940630fd6dcda0d772d4c972c4e0a9946bef9dabf4ef84eda8ef542b82\
    000000000000000000000000ae0ee0a63a2ce6baeeffe56e7714fb4efe48d419\
    0000000000000000000000000000000000000000000000000000000000000004\
    0000000000000000000000000000000000000000000000000000000000000000\
    000000000000000000000000def47ac573dd080526c2e6dd3bc8b4d66e9c6a77\
    00000000000000000000000000000000000000000000000000009184e72a0000\
    0000000000000000000000000000000000000000000000000000000000000000\
    0000000000000000000000000000000000000000000000000000000000000008\
    000000000000000000000000ae0ee0a63a2ce6baeeffe56e7714fb4efe48d419\
    073314940630fd6dcda0d772d4c972c4e0a9946bef9dabf4ef84eda8ef542b82\
    000000000000000000000000000000000000000000000000000000000014de2c\
    02d757788a8d8d6f21d1cd40bce38a8222d70654214e96ff95d8086e684fbee5\
    0000000000000000000000000000000000000000000000000000000000000003\
    015342c9b50c5eed063ef19efb9a57ad10c30d1d39f1f1977f48bcc7199e91e0\
    0000000000000000000000000000000000000000000000000429d069189e0000\
    0000000000000000000000000000000000000000000000000000000000000000";

#[test]
fn test_snos_output_codec() {
    let program_output = hex::decode(SNOS_PROGRAM_OUTPUT_HEX).unwrap();
    let snos_output = StarknetOsOutput::decode(&mut program_output.as_slice());

    let mut actual: Vec<u8> = Vec::new();
    snos_output.into_vec().into_iter().for_each(|felt| actual.extend_from_slice(felt.0.as_slice()));

    assert_eq!(program_output, actual);
}

#[test]
fn test_eth_address_cast() {
    let felt = StarkFelt::try_from("0x000000000000000000000000ae0ee0a63a2ce6baeeffe56e7714fb4efe48d419").unwrap();
    let eth_address = EthAddress::try_from(felt).unwrap();
    let actual = eth_address_to_felt(&eth_address);
    assert_eq!(felt, actual);
}
