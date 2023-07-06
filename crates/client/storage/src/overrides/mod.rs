use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use blockifier::execution::contract_class::ContractClass;
use frame_support::{Identity, StorageHasher};
use mp_starknet::execution::types::{ClassHashWrapper, ContractAddressWrapper, Felt252Wrapper};
use mp_starknet::storage::StarknetStorageSchemaVersion;
use mp_starknet::transaction::types::EventWrapper;
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use pallet_starknet::types::NonceWrapper;
use sc_client_api::{Backend, HeaderBackend, StorageProvider};
use sp_api::ProvideRuntimeApi;
use sp_io::hashing::twox_128;
use sp_runtime::traits::Block as BlockT;

mod schema_v1_override;
use starknet_core::types::FieldElement;

pub use self::schema_v1_override::SchemaV1Override;
use crate::onchain_storage_schema;

/// A handle containing multiple entities implementing `StorageOverride`
pub struct OverrideHandle<B: BlockT> {
    /// Contains one implementation of `StorageOverride` by version of the pallet storage schema
    pub schemas: BTreeMap<StarknetStorageSchemaVersion, Box<dyn StorageOverride<B>>>,
    /// A non-failing way to retrieve the storage data
    pub fallback: Box<dyn StorageOverride<B>>,
}

#[allow(clippy::borrowed_box)]
impl<B: BlockT> OverrideHandle<B> {
    pub fn for_schema_version(&self, schema_version: &StarknetStorageSchemaVersion) -> &Box<dyn StorageOverride<B>> {
        match self.schemas.get(schema_version) {
            Some(storage_override) => storage_override,
            None => &self.fallback,
        }
    }
}

#[allow(clippy::borrowed_box)]
impl<B: BlockT> OverrideHandle<B> {
    pub fn for_block_hash<C: HeaderBackend<B> + StorageProvider<B, BE>, BE: Backend<B>>(
        &self,
        client: &C,
        block_hash: B::Hash,
    ) -> &Box<dyn StorageOverride<B>> {
        let schema_version = onchain_storage_schema(client, block_hash);
        self.for_schema_version(&schema_version)
    }
}

/// Something that can fetch Starknet-related data. This trait is quite similar to the runtime API,
/// and indeed the implementation of it uses the runtime API.
/// Having this trait is useful because it allows optimized implementations that fetch data from a
/// State Backend with some assumptions about pallet-starknet's storage schema. Using such an
/// optimized implementation avoids spawning a runtime and the overhead associated with it.
pub trait StorageOverride<B: BlockT>: Send + Sync {
    /// get storage
    fn get_storage_by_storage_key(
        &self,
        block_hash: B::Hash,
        address: ContractAddressWrapper,
        key: FieldElement,
    ) -> Option<Felt252Wrapper>;

    /// Return the class hash at the provided address for the provided block.
    fn contract_class_hash_by_address(
        &self,
        block_hash: B::Hash,
        address: ContractAddressWrapper,
    ) -> Option<ClassHashWrapper>;
    /// Return the contract class at the provided address for the provided block.
    fn contract_class_by_address(&self, block_hash: B::Hash, address: ContractAddressWrapper) -> Option<ContractClass>;
    /// Return the contract class for a provided class_hash and block hash.
    fn contract_class_by_class_hash(
        &self,
        block_hash: B::Hash,
        contract_class_hash: ClassHashWrapper,
    ) -> Option<ContractClass>;
    /// Returns the nonce for a provided contract address and block hash.
    fn nonce(&self, block_hash: B::Hash, address: ContractAddressWrapper) -> Option<NonceWrapper>;
    /// Returns the events for a provided block hash.
    fn events(&self, block_hash: B::Hash) -> Option<Vec<EventWrapper>>;
    /// Returns the storage value for a provided key and block hash.
    fn chain_id(&self, block_hash: B::Hash) -> Option<Felt252Wrapper>;
}

/// Returns the storage prefix given the pallet module name and the storage name
fn storage_prefix_build(module: &[u8], storage: &[u8]) -> Vec<u8> {
    [twox_128(module), twox_128(storage)].concat().to_vec()
}

/// Returns the storage key for single key maps using the Identity storage hasher.
fn storage_key_build(prefix: Vec<u8>, key: &[u8]) -> Vec<u8> {
    [prefix, Identity::hash(key)].concat()
}

/// A wrapper type for the Runtime API.
///
/// This type implements `StorageOverride`, so it can be used when calling the runtime API is
/// desired but a `dyn StorageOverride` is required.
pub struct RuntimeApiStorageOverride<B: BlockT, C> {
    client: Arc<C>,
    _marker: PhantomData<B>,
}

impl<B: BlockT, C> RuntimeApiStorageOverride<B, C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client, _marker: PhantomData }
    }
}

impl<B, C> StorageOverride<B> for RuntimeApiStorageOverride<B, C>
where
    B: BlockT,
    C: ProvideRuntimeApi<B> + Send + Sync,
    C::Api: StarknetRuntimeApi<B>,
{
    fn get_storage_by_storage_key(
        &self,
        block_hash: <B as BlockT>::Hash,
        address: ContractAddressWrapper,
        key: FieldElement,
    ) -> Option<Felt252Wrapper> {
        let api = self.client.runtime_api();

        match api.get_storage_at(block_hash, address, key.into()) {
            Ok(Ok(storage)) => Some(storage),
            Ok(Err(_)) => None,
            Err(_) => None,
        }
    }

    fn contract_class_by_address(
        &self,
        block_hash: <B as BlockT>::Hash,
        address: ContractAddressWrapper,
    ) -> Option<ContractClass> {
        let api = self.client.runtime_api();
        let contract_class_hash = api.contract_class_hash_by_address(block_hash, address).ok()?;

        match contract_class_hash {
            None => None,
            Some(contract_class_hash) => api.contract_class_by_class_hash(block_hash, contract_class_hash).ok()?,
        }
    }

    // Use the runtime api to fetch the class hash at the provided address for the provided block.
    // # Arguments
    //
    // * `block_hash` - The block hash
    // * `address` - The address to fetch the class hash for
    //
    // # Returns
    // * `Some(class_hash)` - The class hash at the provided address for the provided block
    fn contract_class_hash_by_address(
        &self,
        block_hash: <B as BlockT>::Hash,
        address: ContractAddressWrapper,
    ) -> Option<ClassHashWrapper> {
        let api = self.client.runtime_api();
        api.contract_class_hash_by_address(block_hash, address).ok()?
    }

    /// Return the contract class for a provided class_hash and block hash.
    ///
    /// # Arguments
    ///
    /// * `block_hash` - The block hash
    /// * `contract_class_hash` - The class hash to fetch the contract class for
    ///
    /// # Returns
    /// * `Some(contract_class)` - The contract class for the provided class hash and block hash
    fn contract_class_by_class_hash(
        &self,
        block_hash: <B as BlockT>::Hash,
        contract_class_hash: ClassHashWrapper,
    ) -> Option<ContractClass> {
        self.client.runtime_api().contract_class_by_class_hash(block_hash, contract_class_hash).ok()?
    }

    /// Return the nonce for a provided contract address and block hash.
    ///
    /// # Arguments
    ///
    /// * `block_hash` - The block hash
    /// * `contract_address` - The contract address to fetch the nonce for
    ///
    /// # Returns
    /// * `Some(nonce)` - The nonce for the provided contract address and block hash
    fn nonce(&self, block_hash: <B as BlockT>::Hash, contract_address: ContractAddressWrapper) -> Option<NonceWrapper> {
        self.client.runtime_api().nonce(block_hash, contract_address).ok()
    }

    /// Return the events for a provided block hash.
    ///
    /// # Arguments
    ///
    /// * `block_hash` - The block hash
    ///
    /// # Returns
    /// * `Some(events)` - The events for the provided block hash
    fn events(&self, block_hash: <B as BlockT>::Hash) -> Option<Vec<EventWrapper>> {
        self.client.runtime_api().events(block_hash).ok()
    }

    /// Return the chain id for a provided block hash.
    ///
    /// # Arguments
    ///
    /// * `block_hash` - The block hash
    ///
    /// # Returns
    /// * `Some(chain_id)` - The chain id for the provided block hash
    fn chain_id(&self, block_hash: <B as BlockT>::Hash) -> Option<Felt252Wrapper> {
        self.client.runtime_api().chain_id(block_hash).ok()
    }
}
