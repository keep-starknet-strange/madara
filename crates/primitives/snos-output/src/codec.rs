use alloc::vec::Vec;

use parity_scale_codec::{Decode, Encode, Input, Output};
use starknet_api::{api_core::{ContractAddress, EthAddress}, hash::StarkFelt};

use crate::{MessageL1ToL2, MessageL2ToL1, StarknetOsOutput};

// Field element (252 bit) is encoded as an EVM word (256 bit) and vice versa
// EVM developer should be aware of that and prevent data loss by not using the higest 4 bits

fn vec_encode_to_byte32_words<E: Encode, T: Output + ?Sized>(items: &Vec<E>, dest: &mut T) {
    dest.write(&[0u8; 24]); // 24 bytes padding
    (items.len() as u64).encode_to(dest); // 8 bytes

    for item in items.iter() {
        item.encode_to(dest); // 32 bytes
    }
}

fn skip_bytes<I: Input>(input: &mut I, count: usize) -> Result<(), parity_scale_codec::Error> {
    for _ in 0..count {
        input.read_byte()?;
    }
    Ok(())
}

impl Encode for MessageL2ToL1 {
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        self.from_address.encode_to(dest); // 32 bytes

        dest.write(&[0u8; 12]); // 12 bytes padding
        self.to_address.encode_to(dest); // 20 bytes

        vec_encode_to_byte32_words(&self.payload, dest);
    }

    fn size_hint(&self) -> usize {
        // from_address
        // to_address
        // payload_size
        // payload
        32 * (self.payload.len() + 3)
    }
}

impl Decode for MessageL2ToL1 {
    fn decode<I: Input>(input: &mut I) -> Result<Self, parity_scale_codec::Error> {
        let from_address = ContractAddress::decode(input)?;

        skip_bytes(input, 12)?;
        let to_address = EthAddress::decode(input)?;

        skip_bytes(input, 24)?;
        let payload_len = u64::decode(input)?;

        let mut payload: Vec<StarkFelt> = Vec::with_capacity(payload_len as usize);
        for _ in 0..payload_len {
            payload.push(StarkFelt::decode(input)?);
        }

        Ok(Self {
            from_address,
            to_address,
            payload
        })
    }
}

impl Encode for MessageL1ToL2 {
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        dest.write(&[0u8; 12]); // 12 bytes padding
        self.from_address.encode_to(dest); // 20 bytes

        self.to_address.encode_to(dest); // 32 bytes
        self.nonce.encode_to(dest); // 32 bytes
        self.selector.encode_to(dest); // 32 bytes

        vec_encode_to_byte32_words(&self.payload, dest);
    }

    fn size_hint(&self) -> usize {
        // from_address
        // to_address
        // nonce
        // selector
        // payload_size
        // payload
        32 * (self.payload.len() + 5)
    }
}

impl Decode for MessageL1ToL2 {
    fn decode<I: Input>(input: &mut I) -> Result<Self, parity_scale_codec::Error> {
        todo!()
    }
}

impl Encode for StarknetOsOutput {
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        self.prev_state_root.encode_to(dest); // 32 bytes
        self.new_state_root.encode_to(dest); // 32 bytes

        dest.write(&[0u8; 24]); // 24 bytes padding
        self.block_number.encode_to(dest); // 8 bytes

        self.config_hash.encode_to(dest); // 32 bytes

        vec_encode_to_byte32_words(&self.messages_to_l1, dest);
        vec_encode_to_byte32_words(&self.messages_to_l2, dest);
    }

    fn size_hint(&self) -> usize {
        // prev_state_root
        // next_state_root
        // block_number
        // config_hash
        // segment_size
        // messages_to_l1
        // segment_size
        // messages_to_l2
        32 * 6
            + self.messages_to_l1.iter().map(|msg| msg.size_hint()).sum::<usize>()
            + self.messages_to_l2.iter().map(|msg| msg.size_hint()).sum::<usize>()
    }
}

impl Decode for StarknetOsOutput {
    fn decode<I: Input>(input: &mut I) -> Result<Self, parity_scale_codec::Error> {
        todo!()
    }
}
