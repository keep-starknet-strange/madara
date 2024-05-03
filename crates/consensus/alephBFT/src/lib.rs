#![feature(async_fn_in_trait)]
use aleph_bft::{Data, Hasher, Index, NetworkData, NodeIndex, PartialMultisignature, Signature};
use parity_scale_codec::{Decode, Encode};

pub trait DataProvider<Data> {
    async fn get_data(&mut self) -> Option<Data>;
}

pub trait FinalizationHandler<Data> {
    fn data_finalized(&mut self, data: Data, creator: NodeIndex);
}

pub trait Network<H, D, S, MS>: Send
where
    H: Hasher,
    D: Data,
    S: Encode + Decode,
    MS: PartialMultisignature,
{
    fn send(&self, data: NetworkData<H, D, S, MS>, recipient: Recipient);
    async fn next_event(&mut self) -> Option<NetworkData<H, D, S, MS>>;
}

pub enum Recipient {
    Everyone,
    Node(NodeIndex),
}

pub trait Keychain: Index + Clone + Send + Sync + 'static {
    type Signature: Signature;
    fn sign(&self, msg: &[u8]) -> Self::Signature;
    fn verify(&self, msg: &[u8], sgn: &Self::Signature, index: NodeIndex) -> bool;
}
