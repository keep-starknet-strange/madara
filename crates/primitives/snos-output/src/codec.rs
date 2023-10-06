use alloc::vec::Vec;

use starknet_api::hash::StarkFelt;

use crate::{MessageL1ToL2, MessageL2ToL1, StarknetOsOutput};

/// This codec allows to convert structured OS program output into array of felts
/// 
/// In order to prepare parameters for the Starknet contract `updateState` method:
///     1. Cast the output to dynamic uint256[] array
///     2. Get onchain data hash & size
///     3. ABI encode parameters 
///  
/// NOTE: Field element (252 bit) is encoded as an EVM word (256 bit) and vice versa
/// EVM developer should be aware of that and prevent data loss by not using the higest 4 bits
pub trait SnosCodec: Sized {
    fn felt_len(&self) -> usize;
    fn append_to(self, output: &mut Vec<StarkFelt>);

    fn as_vec(self) -> Vec<StarkFelt> {
        let mut output: Vec<StarkFelt> = Vec::with_capacity(self.felt_len());
        self.append_to(&mut output);
        output
    }
}

fn segment_append_to<T: SnosCodec>(segment: Vec<T>, output: &mut Vec<StarkFelt>) {
    let segment_size = segment.iter().map(|msg| msg.felt_len() as u64).sum::<u64>();
    output.push(segment_size.into());
    for item in segment {
        item.append_to(output);
    }
}

impl SnosCodec for StarknetOsOutput {
    fn felt_len(&self) -> usize {
        7 + self.messages_to_l1.iter().map(|msg| msg.felt_len()).sum::<usize>()
            + self.messages_to_l2.iter().map(|msg| msg.felt_len()).sum::<usize>()
    }

    fn append_to(self, output: &mut Vec<StarkFelt>) {
        output.push(self.prev_state_root);
        output.push(self.new_state_root);
        output.push(self.block_number);
        output.push(self.block_hash);
        output.push(self.config_hash);
        segment_append_to(self.messages_to_l1, output);
        segment_append_to(self.messages_to_l2, output);
    }
}

impl SnosCodec for MessageL2ToL1 {
    fn felt_len(&self) -> usize {
        3 + self.payload.len()
    }

    fn append_to(self, output: &mut Vec<StarkFelt>) {
        output.push(self.from_address);
        output.push(self.to_address);
        output.push((self.payload.len() as u64).into());
        let mut payload = self.payload;
        output.append(&mut payload);
    }
}

impl SnosCodec for MessageL1ToL2 {
    fn felt_len(&self) -> usize {
        5 + self.payload.len()
    }

    fn append_to(self, output: &mut Vec<StarkFelt>) {
        output.push(self.from_address);
        output.push(self.to_address);
        output.push(self.nonce);
        output.push(self.selector);
        output.push((self.payload.len() as u64).into());
        let mut payload = self.payload;
        output.append(&mut payload);
    }
}
