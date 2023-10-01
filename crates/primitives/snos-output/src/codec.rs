use alloc::vec::Vec;

use parity_scale_codec::{Decode, Encode, Input, Output};
use starknet_api::api_core::{ContractAddress, EntryPointSelector, EthAddress, Nonce};
use starknet_api::hash::StarkHash;

use crate::{MessageL1ToL2, MessageL2ToL1, StarknetOsOutput};

// Field element (252 bit) is encoded as an EVM word (256 bit) and vice versa
// EVM developer should be aware of that and prevent data loss by not using the higest 4 bits

fn skip_bytes<I: Input>(input: &mut I, count: usize) -> Result<(), parity_scale_codec::Error> {
    for _ in 0..count {
        input.read_byte()?;
    }
    Ok(())
}

fn u64_encode<O: Output + ?Sized>(value: u64, dest: &mut O) {
    dest.write(&[0u8; 24]); // 24 bytes padding
    dest.write(value.to_be_bytes().as_slice());
}

fn u64_decode<I: Input>(input: &mut I) -> Result<u64, parity_scale_codec::Error> {
    skip_bytes(input, 24)?;
    let mut bytes = [0u8; 8];
    input.read(bytes.as_mut_slice())?;
    Ok(u64::from_be_bytes(bytes))
}

fn vec_encode<T: Encode, O: Output + ?Sized>(items: &[T], dest: &mut O) {
    u64_encode(items.len() as u64, dest);
    for item in items.iter() {
        item.encode_to(dest); // 32 bytes
    }
}

fn vec_decode<T: Decode, I: Input>(input: &mut I) -> Result<Vec<T>, parity_scale_codec::Error> {
    let n_items = u64_decode(input)?;
    let mut items: Vec<T> = Vec::with_capacity(n_items as usize);
    for _ in 0..n_items {
        items.push(T::decode(input)?);
    }
    Ok(items)
}

fn segment_encode<T: Encode, O: Output + ?Sized>(items: &[T], dest: &mut O) {
    // Number of byte32 words
    let segment_size = items.iter().map(|item| item.size_hint() as u64).sum::<u64>() / 32;
    u64_encode(segment_size, dest);
    for item in items.iter() {
        item.encode_to(dest);
    }
}

fn segment_decode<T: Encode + Decode, I: Input>(input: &mut I) -> Result<Vec<T>, parity_scale_codec::Error> {
    let mut segment_len = 32 * u64_decode(input)? as usize;
    let mut items: Vec<T> = Vec::with_capacity(segment_len / 3); // minimum capacity for empty l2 > l1 messages

    if let Some(remaining_len) = input.remaining_len()? {
        if segment_len > remaining_len {
            return Err("Segment size is greater than remaining read buffer length".into());
        }
    }

    while segment_len > 0 {
        let item = T::decode(input)?;
        segment_len -= item.size_hint();
        items.push(item);
    }

    Ok(items)
}

fn eth_address_encode<O: Output + ?Sized>(address: &EthAddress, dest: &mut O) {
    dest.write(&[0u8; 12]); // 12 bytes padding
    address.encode_to(dest); // 20 bytes
}

fn eth_address_decode<I: Input>(input: &mut I) -> Result<EthAddress, parity_scale_codec::Error> {
    skip_bytes(input, 12)?;
    EthAddress::decode(input)
}

impl Encode for MessageL2ToL1 {
    fn encode_to<O: Output + ?Sized>(&self, dest: &mut O) {
        self.from_address.encode_to(dest);
        eth_address_encode(&self.to_address, dest);
        vec_encode(&self.payload, dest);
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
        Ok(Self {
            from_address: ContractAddress::decode(input)?,
            to_address: eth_address_decode(input)?,
            payload: vec_decode(input)?,
        })
    }
}

impl Encode for MessageL1ToL2 {
    fn encode_to<O: Output + ?Sized>(&self, dest: &mut O) {
        eth_address_encode(&self.from_address, dest);
        self.to_address.encode_to(dest);
        self.nonce.encode_to(dest);
        self.selector.encode_to(dest);
        vec_encode(&self.payload, dest);
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
        Ok(Self {
            from_address: eth_address_decode(input)?,
            to_address: ContractAddress::decode(input)?,
            nonce: Nonce::decode(input)?,
            selector: EntryPointSelector::decode(input)?,
            payload: vec_decode(input)?,
        })
    }
}

impl Encode for StarknetOsOutput {
    fn encode_to<O: Output + ?Sized>(&self, dest: &mut O) {
        self.prev_state_root.encode_to(dest);
        self.new_state_root.encode_to(dest);
        u64_encode(self.block_number, dest);
        self.block_hash.encode_to(dest);
        self.config_hash.encode_to(dest);
        segment_encode(&self.messages_to_l1, dest);
        segment_encode(&self.messages_to_l2, dest);
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
        Ok(Self {
            prev_state_root: StarkHash::decode(input)?,
            new_state_root: StarkHash::decode(input)?,
            block_number: u64_decode(input)?,
            block_hash: StarkHash::decode(input)?,
            config_hash: StarkHash::decode(input)?,
            messages_to_l1: segment_decode(input)?,
            messages_to_l2: segment_decode(input)?,
        })
    }
}
