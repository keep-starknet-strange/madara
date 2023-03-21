use frame_support::traits::{ConstU16, ConstU64, GenesisBuild, Hooks};
use hex::FromHex;
use sp_core::H256;
use sp_runtime::testing::Header;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use {crate as pallet_starknet, frame_system as system, pallet_timestamp};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        KaioshinRandomness: pallet_insecure_randomness_collective_flip,
        Starknet: pallet_starknet,
        Timestamp: pallet_timestamp,
    }
);

impl pallet_insecure_randomness_collective_flip::Config for Test {}

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

impl pallet_starknet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Randomness = KaioshinRandomness;
    type StateRoot = pallet_starknet::state_root::IntermediateStateRoot<Self>;
    type SystemHash = kp_starknet::crypto::hash::pedersen::PedersenHasher;
    type TimestampProvider = Timestamp;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

    // ACCOUNT CONTRACT
    let contract_address_str = "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77";
    let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

    let class_hash_str = "025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
    let class_hash_bytes = <[u8; 32]>::from_hex(class_hash_str).unwrap();

    // TEST CONTRACT
    let other_contract_address_str = "0624EBFb99865079bd58CFCFB925B6F5Ce940D6F6e41E118b8A72B7163fB435c";
    let other_contract_address_bytes = <[u8; 32]>::from_hex(other_contract_address_str).unwrap();

    let other_class_hash_str = "025ec026985a3bf9d0cc1fe17326b245bfdc3ff89b8fde106242a3ea56c5a918";
    let other_class_hash_bytes = <[u8; 32]>::from_hex(other_class_hash_str).unwrap();

    // L1 HANDLER CONTRACT
    let l1_handler_contract_address_str = "0000000000000000000000000000000000000000000000000000000000000001";
    let l1_handler_contract_address_bytes = <[u8; 32]>::from_hex(l1_handler_contract_address_str).unwrap();

    let l1_handler_class_hash_str = "01cb5d0b5b5146e1aab92eb9fc9883a32a33a604858bb0275ac0ee65d885bba8";
    let l1_handler_class_hash_bytes = <[u8; 32]>::from_hex(l1_handler_class_hash_str).unwrap();

    pallet_starknet::GenesisConfig::<Test> {
        contracts: vec![
            (contract_address_bytes, class_hash_bytes),
            (other_contract_address_bytes, other_class_hash_bytes),
            (l1_handler_contract_address_bytes, l1_handler_class_hash_bytes),
        ],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}

pub(crate) fn run_to_block(n: u64) {
    let deployer_account = 1;
    let deployer_origin = RuntimeOrigin::signed(deployer_account);
    for b in System::block_number()..=n {
        System::set_block_number(b);
        Timestamp::set_timestamp(System::block_number() * 6_000);
        Starknet::ping(deployer_origin.clone()).unwrap();
        Starknet::on_finalize(b);
    }
}
