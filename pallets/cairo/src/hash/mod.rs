use kp_starknet::crypto::hash::{hash, HashType};

pub trait Hasher {
    fn hash(data: &[u8]) -> [u8; 32];
}

#[derive(PartialEq, Eq, Clone)]
pub struct PoseidonHash;
#[derive(PartialEq, Eq, Clone)]
pub struct PedersenHash;

impl Hasher for PoseidonHash {
    fn hash(data: &[u8]) -> [u8; 32] {
        hash(HashType::Poseidon, data)
    }
}

impl Hasher for PedersenHash {
    fn hash(_data: &[u8]) -> [u8; 32] {
        hash(HashType::Pedersen, _data)
    }
}
