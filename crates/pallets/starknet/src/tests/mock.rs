use core::str::FromStr;

use frame_support::parameter_types;
use frame_support::traits::{ConstU16, ConstU64, GenesisBuild, Hooks};
use mp_starknet::execution::types::{ContractClassWrapper, Felt252Wrapper};
use sp_core::H256;
use sp_runtime::testing::Header;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use starknet_api::api_core::{calculate_contract_address as _calculate_contract_address, ClassHash, ContractAddress};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, ContractAddressSalt};
use starknet_api::StarknetApiError;
use starknet_core::types::FieldElement;
use starknet_core::utils::get_storage_var_address;
use {crate as pallet_starknet, frame_system as system};

use super::constants::*;
use super::utils::get_contract_class;
use crate::types::ContractStorageKeyWrapper;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<MockRuntime>;
type Block = frame_system::mocking::MockBlock<MockRuntime>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum MockRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Starknet: pallet_starknet,
        Timestamp: pallet_timestamp,
    }
);

impl pallet_timestamp::Config for MockRuntime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<{ 6_000 / 2 }>;
    type WeightInfo = ();
}

impl system::Config for MockRuntime {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const UnsignedPriority: u64 = 1 << 20;
}

impl pallet_starknet::Config for MockRuntime {
    type RuntimeEvent = RuntimeEvent;
    type StateRoot = pallet_starknet::state_root::IntermediateStateRoot<Self>;
    type SystemHash = mp_starknet::crypto::hash::pedersen::PedersenHasher;
    type TimestampProvider = Timestamp;
    type UnsignedPriority = UnsignedPriority;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<MockRuntime>().unwrap();

    // ARGENT CLASSES
    let proxy_class_hash = Felt252Wrapper::from_hex_be(ARGENT_PROXY_CLASS_HASH_V0).unwrap();
    let account_class_hash_v0 = Felt252Wrapper::from_hex_be(ARGENT_ACCOUNT_CLASS_HASH_V0).unwrap();

    let blockifier_account_address = Felt252Wrapper::from_hex_be(BLOCKIFIER_ACCOUNT_ADDRESS).unwrap();
    let blockifier_account_class_hash = Felt252Wrapper::from_hex_be(BLOCKIFIER_ACCOUNT_CLASS).unwrap();

    // TEST CLASSES
    let argent_proxy_class = get_contract_class("argent_proxy_v0.json");
    let argent_account_class_v0 = get_contract_class("argent_account_v0.json");
    let openzeppelin_account_class = get_contract_class("account/openzeppelin/account.json");
    let argent_account_class = get_contract_class("account/argent/account.json");
    let braavos_account_class = get_contract_class("account/braavos/account.json");
    let braavos_proxy_class = get_contract_class("account/braavos/openzeppelin_deps/proxy.json");
    let test_class = get_contract_class("test.json");
    let l1_handler_class = get_contract_class("l1_handler.json");
    let blockifier_account_class = get_contract_class("account/simple/account.json");
    let simple_account_class = get_contract_class("account/simple/account.json");
    let inner_call_account_class = get_contract_class("account/unauthorized_inner_call/account.json");
    let erc20_class = get_contract_class("erc20/erc20.json");

    // ACCOUNT CONTRACT
    // - ref testnet tx(0x06cfa9b097bec7a811e791b4c412b3728fb4cd6d3b84ae57db3a10c842b00740)
    let (account_addr, _, _) = account_helper(TEST_ACCOUNT_SALT, AccountType::ArgentV0);

    // OPENZEPPELIN ACCOUNT CONTRACT
    let openzeppelin_class_hash = Felt252Wrapper::from_hex_be(OPENZEPPELIN_ACCOUNT_CLASS_HASH).unwrap();
    let openzeppelin_account_address = get_account_address(AccountType::Openzeppelin);

    // ARGENT ACCOUNT CONTRACT
    let argent_class_hash = Felt252Wrapper::from_hex_be(ARGENT_ACCOUNT_CLASS_HASH).unwrap();
    let argent_account_address = get_account_address(AccountType::Argent);

    // BRAAVOS ACCOUNT CONTRACT
    let braavos_class_hash = Felt252Wrapper::from_hex_be(BRAAVOS_ACCOUNT_CLASS_HASH).unwrap();
    let braavos_account_address = get_account_address(AccountType::Braavos);
    let braavos_proxy_class_hash = Felt252Wrapper::from_hex_be(BRAAVOS_PROXY_CLASS_HASH).unwrap();
    let braavos_proxy_address = get_account_address(AccountType::BraavosProxy);

    // UNAUTHORIZED INNER CALL ACCOUNT CONTRACT
    let inner_call_account_class_hash =
        Felt252Wrapper::from_hex_be(UNAUTHORIZED_INNER_CALL_ACCOUNT_CLASS_HASH).unwrap();
    let inner_call_account_address = get_account_address(AccountType::InnerCall);

    // SIMPLE ACCOUNT CONTRACT
    let simple_account_class_hash = Felt252Wrapper::from_hex_be(SIMPLE_ACCOUNT_CLASS_HASH).unwrap();
    let simple_account_address = get_account_address(AccountType::NoValidate);

    // TEST CONTRACT
    let other_contract_address = Felt252Wrapper::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap();
    let other_class_hash = Felt252Wrapper::from_hex_be(TEST_CLASS_HASH).unwrap();

    // L1 HANDLER CONTRACT
    let l1_handler_contract_address = Felt252Wrapper::from_hex_be(L1_HANDLER_CONTRACT_ADDRESS).unwrap();
    let l1_handler_class_hash = Felt252Wrapper::from_hex_be(L1_HANDLER_CLASS_HASH).unwrap();

    // FEE CONTRACT
    let token_class_hash = Felt252Wrapper::from_hex_be(TOKEN_CONTRACT_CLASS_HASH).unwrap();
    let fee_token_address = Felt252Wrapper::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();

    pallet_starknet::GenesisConfig::<MockRuntime> {
        contracts: vec![
            (account_addr, proxy_class_hash),
            (other_contract_address, other_class_hash),
            (l1_handler_contract_address, l1_handler_class_hash),
            (blockifier_account_address, blockifier_account_class_hash),
            (openzeppelin_account_address, openzeppelin_class_hash),
            (argent_account_address, argent_class_hash),
            (braavos_account_address, braavos_class_hash),
            (braavos_proxy_address, braavos_proxy_class_hash),
            (simple_account_address, simple_account_class_hash),
            (inner_call_account_address, inner_call_account_class_hash),
            (fee_token_address, token_class_hash),
        ],
        contract_classes: vec![
            (proxy_class_hash, ContractClassWrapper::try_from(argent_proxy_class).unwrap()),
            (account_class_hash_v0, ContractClassWrapper::try_from(argent_account_class_v0).unwrap()),
            (other_class_hash, ContractClassWrapper::try_from(test_class).unwrap()),
            (l1_handler_class_hash, ContractClassWrapper::try_from(l1_handler_class).unwrap()),
            (blockifier_account_class_hash, ContractClassWrapper::try_from(blockifier_account_class).unwrap()),
            (openzeppelin_class_hash, ContractClassWrapper::try_from(openzeppelin_account_class).unwrap()),
            (argent_class_hash, ContractClassWrapper::try_from(argent_account_class).unwrap()),
            (braavos_class_hash, ContractClassWrapper::try_from(braavos_account_class).unwrap()),
            (braavos_proxy_class_hash, ContractClassWrapper::try_from(braavos_proxy_class).unwrap()),
            (simple_account_class_hash, ContractClassWrapper::try_from(simple_account_class).unwrap()),
            (inner_call_account_class_hash, ContractClassWrapper::try_from(inner_call_account_class).unwrap()),
            (token_class_hash, ContractClassWrapper::try_from(erc20_class).unwrap()),
        ],
        fee_token_address,
        storage: vec![
            (
                get_storage_key(&fee_token_address, "ERC20_balances", &[simple_account_address], 0),
                Felt252Wrapper::from(u128::MAX),
            ),
            (
                get_storage_key(&fee_token_address, "ERC20_balances", &[simple_account_address], 1),
                Felt252Wrapper::from(u128::MAX),
            ),
            (
                get_storage_key(&fee_token_address, "ERC20_balances", &[blockifier_account_address], 0),
                Felt252Wrapper::from(u128::MAX),
            ),
            (
                get_storage_key(&fee_token_address, "ERC20_balances", &[blockifier_account_address], 1),
                Felt252Wrapper::from(u128::MAX),
            ),
            (
                get_storage_key(&fee_token_address, "ERC20_balances", &[openzeppelin_account_address], 0),
                Felt252Wrapper::from(u128::MAX),
            ),
            (
                get_storage_key(&fee_token_address, "ERC20_balances", &[openzeppelin_account_address], 1),
                Felt252Wrapper::from(u128::MAX),
            ),
            (
                get_storage_key(&fee_token_address, "ERC20_balances", &[argent_account_address], 0),
                Felt252Wrapper::from(u128::MAX),
            ),
            (
                get_storage_key(&fee_token_address, "ERC20_balances", &[argent_account_address], 1),
                Felt252Wrapper::from(u128::MAX),
            ),
            (
                get_storage_key(&fee_token_address, "ERC20_balances", &[braavos_account_address], 0),
                Felt252Wrapper::from(u128::MAX),
            ),
            (
                get_storage_key(&fee_token_address, "ERC20_balances", &[braavos_account_address], 1),
                Felt252Wrapper::from(u128::MAX),
            ),
            (
                get_storage_key(&openzeppelin_account_address, "Account_public_key", &[], 0),
                Felt252Wrapper::from_hex_be(ACCOUNT_PUBLIC_KEY).unwrap(),
            ),
            (
                get_storage_key(&argent_account_address, "_signer", &[], 0),
                Felt252Wrapper::from_hex_be(ACCOUNT_PUBLIC_KEY).unwrap(),
            ),
            (
                get_storage_key(&braavos_account_address, "Account_signers", &[Felt252Wrapper::ZERO], 0),
                Felt252Wrapper::from_hex_be(ACCOUNT_PUBLIC_KEY).unwrap(),
            ),
        ],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}

/// Run to block n.
/// The function will repeatedly create and run blocks until the block number is equal to `n`.
/// # Arguments
/// * `n` - The block number to run to.
pub(crate) fn run_to_block(n: u64) {
    let deployer_origin = RuntimeOrigin::none();
    for b in System::block_number()..=n {
        System::set_block_number(b);
        Timestamp::set_timestamp(System::block_number() * 6_000);
        Starknet::ping(deployer_origin.clone()).unwrap();
        Starknet::on_finalize(b);
    }
}

/// Returns the storage key for a given storage name, keys and offset.
/// Calculates pedersen(sn_keccak(storage_name), keys) + storage_key_offset which is the key in the
/// starknet contract for storage_name(key_1, key_2, ..., key_n).
/// https://docs.starknet.io/documentation/architecture_and_concepts/Contracts/contract-storage/#storage_variables
pub fn get_storage_key(
    address: &Felt252Wrapper,
    storage_name: &str,
    keys: &[Felt252Wrapper],
    storage_key_offset: u64,
) -> ContractStorageKeyWrapper {
    let storage_key_offset = H256::from_low_u64_be(storage_key_offset);
    let mut storage_key = get_storage_var_address(
        storage_name,
        keys.iter().map(|x| FieldElement::from(*x)).collect::<Vec<_>>().as_slice(),
    )
    .unwrap();
    storage_key += FieldElement::from_bytes_be(&storage_key_offset.to_fixed_bytes()).unwrap();
    (*address, storage_key.into())
}

pub enum AccountType {
    Argent,
    ArgentV0,
    Openzeppelin,
    Braavos,
    BraavosProxy,
    NoValidate,
    InnerCall,
}

/// Returns the account class hash, the contract data and the salt for an account type
pub fn account_helper(salt: &str, account_type: AccountType) -> (Felt252Wrapper, Felt252Wrapper, Vec<&str>) {
    let (account_class_hash, cd_raw) = match account_type {
        AccountType::Argent => (H256::from_str(ARGENT_ACCOUNT_CLASS_HASH).unwrap(), vec![]),
        AccountType::ArgentV0 => (
            H256::from_str(ARGENT_PROXY_CLASS_HASH_V0).unwrap(),
            vec![
                ARGENT_ACCOUNT_CLASS_HASH_V0,
                "0x79dc0da7c54b95f10aa182ad0a46400db63156920adb65eca2654c0945a463",
                "0x2",
                salt,
                "0x0",
            ],
        ),
        AccountType::Braavos => (H256::from_str(BRAAVOS_ACCOUNT_CLASS_HASH).unwrap(), vec![]),
        AccountType::BraavosProxy => (
            H256::from_str(BRAAVOS_PROXY_CLASS_HASH).unwrap(),
            vec![
                BRAAVOS_ACCOUNT_CLASS_HASH, // Braavos account class hash
                "0x02dd76e7ad84dbed81c314ffe5e7a7cacfb8f4836f01af4e913f275f89a3de1a", // 'initializer' selector
            ],
        ),
        AccountType::Openzeppelin => (H256::from_str(OPENZEPPELIN_ACCOUNT_CLASS_HASH).unwrap(), vec![]),
        AccountType::NoValidate => (H256::from_str(SIMPLE_ACCOUNT_CLASS_HASH).unwrap(), vec![]),
        AccountType::InnerCall => (H256::from_str(UNAUTHORIZED_INNER_CALL_ACCOUNT_CLASS_HASH).unwrap(), vec![]),
    };
    let account_salt = H256::from_str(salt).unwrap();

    let addr = calculate_contract_address(account_salt, account_class_hash, cd_raw.clone()).unwrap();
    (addr.0.0.into(), account_class_hash.try_into().unwrap(), cd_raw)
}

/// Returns the account address for an account type
pub fn get_account_address(account_type: AccountType) -> Felt252Wrapper {
    account_helper(TEST_ACCOUNT_SALT, account_type).0
}

/// Calculate the address of a contract.
/// # Arguments
/// * `salt` - The salt of the contract.
/// * `class_hash` - The hash of the contract class.
/// * `constructor_calldata` - The calldata of the constructor.
/// # Returns
/// The address of the contract.
/// # Errors
/// If the contract address cannot be calculated.
pub fn calculate_contract_address(
    salt: H256,
    class_hash: H256,
    constructor_calldata: Vec<&str>,
) -> Result<ContractAddress, StarknetApiError> {
    _calculate_contract_address(
        ContractAddressSalt(StarkFelt::new(salt.0)?),
        ClassHash(StarkFelt::new(class_hash.0)?),
        &Calldata(
            constructor_calldata
                .clone()
                .into_iter()
                .map(|x| StarkFelt::try_from(x).unwrap())
                .collect::<Vec<StarkFelt>>()
                .into(),
        ),
        ContractAddress::default(),
    )
}
