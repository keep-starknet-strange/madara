use starknet_api::state::StorageKey;
use starknet_api::{api_core as stcore, block as stb, transaction as sttx};

use super::Felt252Wrapper;

impl From<Felt252Wrapper> for stb::BlockHash {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<stb::BlockHash> for Felt252Wrapper {
    fn from(value: stb::BlockHash) -> Self {
        value.0.into()
    }
}

impl From<Felt252Wrapper> for sttx::TransactionHash {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<sttx::TransactionHash> for Felt252Wrapper {
    fn from(value: sttx::TransactionHash) -> Self {
        value.0.into()
    }
}

impl From<Felt252Wrapper> for stcore::Nonce {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<stcore::Nonce> for Felt252Wrapper {
    fn from(value: stcore::Nonce) -> Self {
        value.0.into()
    }
}

impl From<Felt252Wrapper> for stcore::ClassHash {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<stcore::ClassHash> for Felt252Wrapper {
    fn from(value: stcore::ClassHash) -> Self {
        value.0.into()
    }
}

impl From<Felt252Wrapper> for stcore::CompiledClassHash {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<stcore::CompiledClassHash> for Felt252Wrapper {
    fn from(value: stcore::CompiledClassHash) -> Self {
        value.0.into()
    }
}

impl From<Felt252Wrapper> for stcore::PatriciaKey {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<stcore::PatriciaKey> for Felt252Wrapper {
    fn from(value: stcore::PatriciaKey) -> Self {
        value.0.into()
    }
}

impl From<Felt252Wrapper> for stcore::ContractAddress {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<stcore::ContractAddress> for Felt252Wrapper {
    fn from(value: stcore::ContractAddress) -> Self {
        value.0.into()
    }
}

impl From<Felt252Wrapper> for stcore::EntryPointSelector {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<stcore::EntryPointSelector> for Felt252Wrapper {
    fn from(value: stcore::EntryPointSelector) -> Self {
        value.0.into()
    }
}

impl From<Felt252Wrapper> for sttx::ContractAddressSalt {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<sttx::ContractAddressSalt> for Felt252Wrapper {
    fn from(value: sttx::ContractAddressSalt) -> Self {
        value.0.into()
    }
}

impl From<Felt252Wrapper> for StorageKey {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<StorageKey> for Felt252Wrapper {
    fn from(value: StorageKey) -> Self {
        value.0.0.into()
    }
}

impl From<Felt252Wrapper> for sttx::TransactionVersion {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<sttx::TransactionVersion> for Felt252Wrapper {
    fn from(value: sttx::TransactionVersion) -> Self {
        value.0.into()
    }
}

impl From<Felt252Wrapper> for sttx::EventKey {
    fn from(value: Felt252Wrapper) -> Self {
        Self(value.into())
    }
}

impl From<sttx::EventKey> for Felt252Wrapper {
    fn from(value: sttx::EventKey) -> Self {
        value.0.into()
    }
}
