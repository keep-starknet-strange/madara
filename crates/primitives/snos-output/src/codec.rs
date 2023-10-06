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
    fn size_hint(&self) -> usize;
    fn encode_to(self, output: &mut Vec<StarkFelt>);

    fn into_vec(self) -> Vec<StarkFelt> {
        let mut output: Vec<StarkFelt> = Vec::with_capacity(self.size_hint());
        self.encode_to(&mut output);
        output
    }
}

impl SnosCodec for StarkFelt {
    fn size_hint(&self) -> usize {
        1
    }

    fn encode_to(self, output: &mut Vec<StarkFelt>) {
        output.push(self);
    }
}

impl<T: SnosCodec> SnosCodec for Vec<T> {
    fn size_hint(&self) -> usize {
        self.iter().map(|elt| elt.size_hint()).sum()
    }

    fn encode_to(self, output: &mut Vec<StarkFelt>) {
        let segment_size = self.size_hint() as u64;
        output.push(segment_size.into());
        for elt in self.into_iter() {
            elt.encode_to(output);
        }
    }
}

impl SnosCodec for MessageL2ToL1 {
    fn size_hint(&self) -> usize {
        3 + self.payload.len()
    }

    fn encode_to(self, output: &mut Vec<StarkFelt>) {
        output.push(self.from_address);
        output.push(self.to_address);
        self.payload.encode_to(output);
    }
}

impl SnosCodec for MessageL1ToL2 {
    fn size_hint(&self) -> usize {
        5 + self.payload.len()
    }

    fn encode_to(self, output: &mut Vec<StarkFelt>) {
        output.push(self.from_address);
        output.push(self.to_address);
        output.push(self.nonce);
        output.push(self.selector);
        self.payload.encode_to(output);
    }
}

impl SnosCodec for StarknetOsOutput {
    fn size_hint(&self) -> usize {
        7 + self.messages_to_l1.iter().map(|msg| msg.size_hint()).sum::<usize>()
            + self.messages_to_l2.iter().map(|msg| msg.size_hint()).sum::<usize>()
    }

    fn encode_to(self, output: &mut Vec<StarkFelt>) {
        output.push(self.prev_state_root);
        output.push(self.new_state_root);
        output.push(self.block_number);
        output.push(self.block_hash);
        output.push(self.config_hash);
        self.messages_to_l1.encode_to(output);
        self.messages_to_l2.encode_to(output);
    }
}
