use core::str::FromStr;

use blockifier::test_utils::{get_contract_class, ACCOUNT_CONTRACT_PATH};
use frame_support::parameter_types;
use frame_support::traits::{ConstU16, ConstU64, GenesisBuild, Hooks};
use frame_support::weights::IdentityFee;
use hex::FromHex;
use mp_starknet::execution::ContractClassWrapper;
use pallet_transaction_payment::{ConstFeeMultiplier, Multiplier};
use sp_core::{ConstU8, H256, U256};
use sp_runtime::testing::Header;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup, One};
use starknet_api::api_core::{calculate_contract_address as _calculate_contract_address, ClassHash, ContractAddress};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, ContractAddressSalt};
use starknet_api::StarknetApiError;
use {crate as pallet_starknet, frame_system as system};

use crate::StarknetFee;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const ARGENT_PROXY_CLASS_HASH_V0: &str = "0x025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
pub const ARGENT_ACCOUNT_CLASS_HASH_V0: &str = "0x033434ad846cdd5f23eb73ff09fe6fddd568284a0fb7d1be20ee482f044dabe2";
pub const BLOCKIFIER_ACCOUNT_CLASS: &str = "0x03bcec8de953ba8e305e2ce2db52c91504aefa7c56c91211873b4d6ba36e8c32";
pub const TEST_CLASS_HASH: &str = "0x00000000000000000000000000000000000000000000000000000000DEADBEEF";
pub const TEST_ACCOUNT_SALT: &str = "0x0780f72e33c1508df24d8f00a96ecc6e08a850ecb09f7e6dff6a81624c0ef46a";

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Starknet: pallet_starknet,
        Timestamp: pallet_timestamp,
        TransactionPayment: pallet_transaction_payment,
    }
);

impl pallet_timestamp::Config for Test {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<{ 6_000 / 2 }>;
    type WeightInfo = ();
}

impl system::Config for Test {
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

impl pallet_starknet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type StateRoot = pallet_starknet::state_root::IntermediateStateRoot<Self>;
    type SystemHash = mp_starknet::crypto::hash::pedersen::PedersenHasher;
    type TimestampProvider = Timestamp;
    type UnsignedPriority = UnsignedPriority;
}
parameter_types! {
    pub FeeMultiplier: Multiplier = Multiplier::one();
}
// Provides the logic needed to handle transaction fees
impl pallet_transaction_payment::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = StarknetFee;
    type OperationalFeeMultiplier = ConstU8<5>;
    type WeightToFee = IdentityFee<u128>;
    type LengthToFee = IdentityFee<u128>;
    type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

    // ARGENT CLASSES
    let proxy_class_hash = <[u8; 32]>::from_hex(ARGENT_PROXY_CLASS_HASH_V0.strip_prefix("0x").unwrap()).unwrap();
    let account_class_hash = <[u8; 32]>::from_hex(ARGENT_ACCOUNT_CLASS_HASH_V0.strip_prefix("0x").unwrap()).unwrap();

    let blockifier_account_address =
        <[u8; 32]>::from_hex("02356b628d108863baf8644c945d97bad70190af5957031f4852d00d0f690a77").unwrap();
    let blockifier_account_class_hash =
        <[u8; 32]>::from_hex(BLOCKIFIER_ACCOUNT_CLASS.strip_prefix("0x").unwrap()).unwrap();

    // TEST CLASSES
    let argent_proxy_class = get_contract_class(include_bytes!("../../../../resources/argent_proxy_v0.json"));
    let argent_account_class = get_contract_class(include_bytes!("../../../../resources/argent_account_v0.json"));
    let test_class = get_contract_class(include_bytes!("../../../../resources/test.json"));
    let l1_handler_class = get_contract_class(include_bytes!("../../../../resources/l1_handler.json"));
    let blockifier_account_class = get_contract_class(ACCOUNT_CONTRACT_PATH);
    let simple_account_class = get_contract_class(include_bytes!("../../../../resources/account/account.json"));
    let erc20_class = get_contract_class(include_bytes!("../../../../resources/erc20/erc20.json"));
    let simple_account_address =
        <[u8; 32]>::from_hex("000000000000000000000000000000000000000000000000000000000000000F").unwrap();
    let simple_account_class_hash =
        <[u8; 32]>::from_hex("000000000000000000000000000000000000000000000000000000000000000E").unwrap();

    // ACCOUNT CONTRACT
    // - ref testnet tx(0x06cfa9b097bec7a811e791b4c412b3728fb4cd6d3b84ae57db3a10c842b00740)
    let (account_addr, _, _) = account_helper(TEST_ACCOUNT_SALT);

    // TEST CONTRACT
    let other_contract_address_bytes =
        <[u8; 32]>::from_hex("024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap();
    let other_class_hash_bytes = <[u8; 32]>::from_hex(TEST_CLASS_HASH.strip_prefix("0x").unwrap()).unwrap();

    // L1 HANDLER CONTRACT
    let l1_handler_contract_address_bytes =
        <[u8; 32]>::from_hex("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
    let l1_handler_class_hash_bytes =
        <[u8; 32]>::from_hex("01cb5d0b5b5146e1aab92eb9fc9883a32a33a604858bb0275ac0ee65d885bba8").unwrap();

    // FEE CONTRACT
    let token_class_hash_str = "0000000000000000000000000000000000000000000000000000000000010000";
    let token_class_hash_bytes = <[u8; 32]>::from_hex(token_class_hash_str).unwrap();
    let fee_token_address =
        <[u8; 32]>::from_hex("00000000000000000000000000000000000000000000000000000000000000AA").unwrap();

    pallet_starknet::GenesisConfig::<Test> {
        contracts: vec![
            (account_addr, proxy_class_hash),
            (other_contract_address_bytes, other_class_hash_bytes),
            (l1_handler_contract_address_bytes, l1_handler_class_hash_bytes),
            (blockifier_account_address, blockifier_account_class_hash),
            (simple_account_address, simple_account_class_hash),
            (fee_token_address, token_class_hash_bytes),
        ],
        contract_classes: vec![
            (proxy_class_hash, ContractClassWrapper::from(argent_proxy_class)),
            (account_class_hash, ContractClassWrapper::from(argent_account_class)),
            (other_class_hash_bytes, ContractClassWrapper::from(test_class)),
            (l1_handler_class_hash_bytes, ContractClassWrapper::from(l1_handler_class)),
            (blockifier_account_class_hash, ContractClassWrapper::from(blockifier_account_class)),
            (simple_account_class_hash, ContractClassWrapper::from(simple_account_class)),
            (token_class_hash_bytes, ContractClassWrapper::from(erc20_class)),
        ],
        fee_token_address,
        storage: vec![
            (
                (
                    fee_token_address,
                    // pedersen(sn_keccak(b"ERC20_balances"), 0x0F) which is the key in the starknet contract for
                    // ERC20_balances(0x0F).low
                    H256::from_str("0x078e4fa4db2b6f3c7a9ece31571d47ac0e853975f90059f7c9df88df974d9093").unwrap(),
                ),
                U256::from(u128::MAX),
            ),
            (
                (
                    fee_token_address,
                    // pedersen(sn_keccak(b"ERC20_balances"), 0x0F) + 1 which is the key in the starknet contract for
                    // ERC20_balances(0x0F).high
                    H256::from_str("0x078e4fa4db2b6f3c7a9ece31571d47ac0e853975f90059f7c9df88df974d9094").unwrap(),
                ),
                U256::from(u128::MAX),
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

pub fn account_helper(salt: &str) -> ([u8; 32], [u8; 32], Vec<&str>) {
    let account_class_hash = H256::from_str(ARGENT_PROXY_CLASS_HASH_V0).unwrap();
    let account_salt = H256::from_str(salt).unwrap();

    let cd_raw = vec![
        ARGENT_ACCOUNT_CLASS_HASH_V0,
        "0x79dc0da7c54b95f10aa182ad0a46400db63156920adb65eca2654c0945a463",
        "0x2",
        salt,
        "0x0",
    ];

    let addr = calculate_contract_address(account_salt, account_class_hash, cd_raw.clone()).unwrap();
    (addr.0.0.0, account_class_hash.to_fixed_bytes(), cd_raw)
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
