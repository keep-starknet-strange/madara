use core::str::FromStr;

use blockifier::execution::contract_class::ContractClass;
use frame_support::traits::{ConstU16, ConstU64, GenesisBuild, Hooks};
use frame_support::{bounded_vec, parameter_types};
use hex::FromHex;
use mp_starknet::execution::types::ContractClassWrapper;
use mp_starknet::transaction::types::MaxArraySize;
use sp_core::{H256, U256};
use sp_runtime::testing::Header;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use sp_runtime::BoundedVec;
use starknet_api::api_core::{calculate_contract_address as _calculate_contract_address, ClassHash, ContractAddress};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, ContractAddressSalt};
use starknet_api::StarknetApiError;
use starknet_crypto::{sign, FieldElement};
use {crate as pallet_starknet, frame_system as system};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<MockRuntime>;
type Block = frame_system::mocking::MockBlock<MockRuntime>;

pub const ACCOUNT_PUBLIC_KEY: &str = "0x03603a2692a2ae60abb343e832ee53b55d6b25f02a3ef1565ec691edc7a209b2";
const ACCOUNT_PRIVATE_KEY: &str = "0x00c1cf1490de1352865301bb8705143f3ef938f97fdf892f1090dcb5ac7bcd1d";
const K: &str = "0x0000000000000000000000000000000000000000000000000000000000000001";

pub const ARGENT_PROXY_CLASS_HASH_V0: &str = "0x025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918";
pub const ARGENT_ACCOUNT_CLASS_HASH: &str = "06f0d6f6ae72e1a507ff4b65181291642889742dbf8f1a53e9ec1c595d01ba7d";
pub const ARGENT_ACCOUNT_CLASS_HASH_V0: &str = "0x033434ad846cdd5f23eb73ff09fe6fddd568284a0fb7d1be20ee482f044dabe2";
pub const OPENZEPPELIN_ACCOUNT_CLASS_HASH: &str = "006280083f8c2a2db9f737320d5e3029b380e0e820fe24b8d312a6a34fdba0cd";
pub const BRAAVOS_ACCOUNT_CLASS_HASH: &str = "0244ca3d9fe8b47dd565a6f4270d979ba31a7d6ff2c3bf8776198161505e8b52";
pub const BRAAVOS_PROXY_CLASS_HASH: &str = "06a89ae7bd72c96202c040341c1ee422474b562e1d73c6848f08cae429c33262";
pub const BLOCKIFIER_ACCOUNT_CLASS: &str = "0x03bcec8de953ba8e305e2ce2db52c91504aefa7c56c91211873b4d6ba36e8c32";
pub const SIMPLE_ACCOUNT_CLASS_HASH: &str = "0x0279d77db761fba82e0054125a6fdb5f6baa6286fa3fb73450cc44d193c2d37f";
pub const UNAUTHORIZED_INNER_CALL_ACCOUNT_CLASS_HASH: &str =
    "0x071aaf68d30c3e52e1c4b7d1209b0e09525939c31bb0275919dffd4cd53f57c4";
pub const TEST_CLASS_HASH: &str = "0x00000000000000000000000000000000000000000000000000000000DEADBEEF";
pub const TEST_ACCOUNT_SALT: &str = "0x0780f72e33c1508df24d8f00a96ecc6e08a850ecb09f7e6dff6a81624c0ef46a";
pub const TOKEN_CONTRACT_CLASS_HASH: &str = "0x06232eeb9ecb5de85fc927599f144913bfee6ac413f2482668c9f03ce4d07922";
pub const BLOCKIFIER_ACCOUNT_ADDRESS: &str = "0x02356b628d108863baf8644c945d97bad70190af5957031f4852d00d0f690a77";
pub const FEE_TOKEN_ADDRESS: &str = "0x00000000000000000000000000000000000000000000000000000000000000AA";

pub fn get_contract_class(contract_content: &'static [u8]) -> ContractClass {
    serde_json::from_slice(contract_content).unwrap()
}

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
    let proxy_class_hash = H256::from_str(ARGENT_PROXY_CLASS_HASH_V0).unwrap().to_fixed_bytes();
    let account_class_hash = H256::from_str(ARGENT_ACCOUNT_CLASS_HASH_V0).unwrap().to_fixed_bytes();

    let blockifier_account_address = H256::from_str(BLOCKIFIER_ACCOUNT_ADDRESS).unwrap().to_fixed_bytes();
    let blockifier_account_class_hash = H256::from_str(BLOCKIFIER_ACCOUNT_CLASS).unwrap().to_fixed_bytes();

    // TEST CLASSES
    let argent_proxy_class = get_contract_class(include_bytes!("../../../../../resources/argent_proxy_v0.json"));
    let argent_account_class_v0 = get_contract_class(include_bytes!("../../../../../resources/argent_account_v0.json"));
    let openzeppelin_account_class =
        get_contract_class(include_bytes!("../../../../../resources/account/openzeppelin/account.json"));
    let argent_account_class =
        get_contract_class(include_bytes!("../../../../../resources/account/argent/account.json"));
    let braavos_account_class =
        get_contract_class(include_bytes!("../../../../../resources/account/braavos/account.json"));
    let braavos_proxy_class =
        get_contract_class(include_bytes!("../../../../../resources/account/braavos/openzepellin_deps/proxy.json"));
    let test_class = get_contract_class(include_bytes!("../../../../../resources/test.json"));
    let l1_handler_class = get_contract_class(include_bytes!("../../../../../resources/l1_handler.json"));
    let blockifier_account_class =
        get_contract_class(include_bytes!("../../../../../resources/account/simple/account.json"));
    let simple_account_class =
        get_contract_class(include_bytes!("../../../../../resources/account/simple/account.json"));
    let inner_call_account_class =
        get_contract_class(include_bytes!("../../../../../resources/account/unauthorized_inner_call/account.json"));
    let erc20_class = get_contract_class(include_bytes!("../../../../../resources/erc20/erc20.json"));

    // ACCOUNT CONTRACT
    // - ref testnet tx(0x06cfa9b097bec7a811e791b4c412b3728fb4cd6d3b84ae57db3a10c842b00740)
    let (account_addr, _, _) = account_helper(TEST_ACCOUNT_SALT, AccountType::ArgentV0);

    // OPENZEPPELIN ACCOUNT CONTRACT
    let openzeppelin_class_hash_bytes = <[u8; 32]>::from_hex(OPENZEPPELIN_ACCOUNT_CLASS_HASH).unwrap();
    let openzeppelin_account_address = get_account_address(AccountType::Openzeppelin);

    // ARGENT ACCOUNT CONTRACT
    let argent_class_hash_bytes = <[u8; 32]>::from_hex(ARGENT_ACCOUNT_CLASS_HASH).unwrap();
    let argent_account_address = get_account_address(AccountType::Argent);

    // BRAAVOS ACCOUNT CONTRACT
    let braavos_class_hash_bytes = <[u8; 32]>::from_hex(BRAAVOS_ACCOUNT_CLASS_HASH).unwrap();
    let braavos_account_address = get_account_address(AccountType::Braavos);
    let braavos_proxy_class_hash_bytes = <[u8; 32]>::from_hex(BRAAVOS_PROXY_CLASS_HASH).unwrap();
    let braavos_proxy_address = get_account_address(AccountType::BraavosProxy);

    // UNAUTHORIZED INNER CALL ACCOUNT CONTRACT
    let inner_call_account_class_hash =
        <[u8; 32]>::from_hex(UNAUTHORIZED_INNER_CALL_ACCOUNT_CLASS_HASH.strip_prefix("0x").unwrap()).unwrap();
    let inner_call_account_address = get_account_address(AccountType::InnerCall);

    // SIMPLE ACCOUNT CONTRACT
    let simple_account_class_hash =
        <[u8; 32]>::from_hex(SIMPLE_ACCOUNT_CLASS_HASH.strip_prefix("0x").unwrap()).unwrap();
    let simple_account_address = get_account_address(AccountType::NoValidate);

    // TEST CONTRACT
    let other_contract_address =
        <[u8; 32]>::from_hex("024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap();
    let other_class_hash = H256::from_str(TEST_CLASS_HASH).unwrap().to_fixed_bytes();

    // L1 HANDLER CONTRACT
    let l1_handler_contract_address =
        <[u8; 32]>::from_hex("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
    let l1_handler_class_hash =
        <[u8; 32]>::from_hex("01cb5d0b5b5146e1aab92eb9fc9883a32a33a604858bb0275ac0ee65d885bba8").unwrap();

    // FEE CONTRACT
    let token_class_hash = H256::from_str(TOKEN_CONTRACT_CLASS_HASH).unwrap().to_fixed_bytes();
    let fee_token_address = H256::from_str(FEE_TOKEN_ADDRESS).unwrap().to_fixed_bytes();

    pallet_starknet::GenesisConfig::<MockRuntime> {
        contracts: vec![
            (account_addr, proxy_class_hash),
            (other_contract_address, other_class_hash),
            (l1_handler_contract_address, l1_handler_class_hash),
            (blockifier_account_address, blockifier_account_class_hash),
            (openzeppelin_account_address, openzeppelin_class_hash_bytes),
            (argent_account_address, argent_class_hash_bytes),
            (braavos_account_address, braavos_class_hash_bytes),
            (braavos_proxy_address, braavos_proxy_class_hash_bytes),
            (simple_account_address, simple_account_class_hash),
            (inner_call_account_address, inner_call_account_class_hash),
            (fee_token_address, token_class_hash),
        ],
        contract_classes: vec![
            (proxy_class_hash, ContractClassWrapper::try_from(argent_proxy_class).unwrap()),
            (account_class_hash, ContractClassWrapper::try_from(argent_account_class).unwrap()),
            (other_class_hash, ContractClassWrapper::try_from(test_class).unwrap()),
            (l1_handler_class_hash, ContractClassWrapper::try_from(l1_handler_class).unwrap()),
            (blockifier_account_class_hash, ContractClassWrapper::try_from(blockifier_account_class).unwrap()),
            (openzeppelin_class_hash_bytes, ContractClassWrapper::try_from(openzeppelin_account_class).unwrap()),
            (argent_class_hash_bytes, ContractClassWrapper::try_from(argent_account_class).unwrap()),
            (braavos_class_hash_bytes, ContractClassWrapper::try_from(braavos_account_class).unwrap()),
            (braavos_proxy_class_hash_bytes, ContractClassWrapper::try_from(braavos_proxy_class).unwrap()),
            (simple_account_class_hash, ContractClassWrapper::try_from(simple_account_class).unwrap()),
            (inner_call_account_class_hash, ContractClassWrapper::try_from(inner_call_account_class).unwrap()),
            (token_class_hash, ContractClassWrapper::try_from(erc20_class).unwrap()),
        ],
        fee_token_address,
        storage: vec![
            (
                (
                    fee_token_address,
                    // pedersen(sn_keccak(b"ERC20_balances"),
                    // 0x01a3339ec92ac1061e3e0f8e704106286c642eaf302e94a582e5f95ef5e6b4d0) which is the key in the
                    // starknet contract for
                    // ERC20_balances(0x01a3339ec92ac1061e3e0f8e704106286c642eaf302e94a582e5f95ef5e6b4d0).low
                    H256::from_str("0x03701645da930cd7f63318f7f118a9134e72d64ab73c72ece81cae2bd5fb403f").unwrap(),
                ),
                U256::from(u128::MAX),
            ),
            (
                (
                    fee_token_address,
                    // pedersen(sn_keccak(b"ERC20_balances"),
                    // 0x01a3339ec92ac1061e3e0f8e704106286c642eaf302e94a582e5f95ef5e6b4d0) + 1 which is the key in the
                    // starknet contract for
                    // ERC20_balances(0x01a3339ec92ac1061e3e0f8e704106286c642eaf302e94a582e5f95ef5e6b4d0).high
                    H256::from_str("0x03701645da930cd7f63318f7f118a9134e72d64ab73c72ece81cae2bd5fb4040").unwrap(),
                ),
                U256::from(u128::MAX),
            ),
            (
                (
                    fee_token_address,
                    // pedersen(sn_keccak(b"ERC20_balances"),
                    // 0x02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77) which is the key in the
                    // starknet contract for
                    // ERC20_balances(0x02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77).low (this
                    // address corresponds to the sender address of the invoke tx from json)
                    H256::from_str("0x06afaa15cba5e9ea552a55fec494d2d859b4b73506794bf5afbb3d73c1fb00aa").unwrap(),
                ),
                U256::from(u128::MAX),
            ),
            (
                (
                    fee_token_address,
                    // pedersen(sn_keccak(b"ERC20_balances"),
                    // 0x02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77) + 1 which is the key in the
                    // starknet contract for
                    // ERC20_balances(0x02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77).high (this
                    // address corresponds to the sender address of the invoke tx from json)
                    H256::from_str("0x06afaa15cba5e9ea552a55fec494d2d859b4b73506794bf5afbb3d73c1fb00ab").unwrap(),
                ),
                U256::from(u128::MAX),
            ),
            (
                (
                    fee_token_address,
                    // pedersen(sn_keccak(b"ERC20_balances"),
                    // 0x04b6c8fa64a0ce8c8eae8e3d421d74fcb77a87ecb771c882ac5bacdccd598012) which is the key in the
                    // starknet contract for
                    // ERC20_balances(0x04b6c8fa64a0ce8c8eae8e3d421d74fcb77a87ecb771c882ac5bacdccd598012).low (this
                    // address corresponds to the sender address of the invoke tx from json)
                    H256::from_str("0x0654f5a2b807de48bfebf66b84267f110e2577edec66de3fb711a04d491e29bc").unwrap(),
                ),
                U256::from(u128::MAX),
            ),
            (
                (
                    fee_token_address,
                    // pedersen(sn_keccak(b"ERC20_balances"),
                    // 0x04b6c8fa64a0ce8c8eae8e3d421d74fcb77a87ecb771c882ac5bacdccd598012) + 1 which is the key in the
                    // starknet contract for
                    // ERC20_balances(0x04b6c8fa64a0ce8c8eae8e3d421d74fcb77a87ecb771c882ac5bacdccd598012).high (this
                    // address corresponds to the sender address of the invoke tx from json)
                    H256::from_str("0x0654f5a2b807de48bfebf66b84267f110e2577edec66de3fb711a04d491e29bd").unwrap(),
                ),
                U256::from(u128::MAX),
            ),
            (
                (
                    fee_token_address,
                    // pedersen(sn_keccak(b"ERC20_balances"),
                    // 0x02e63de215f650e9d7e2313c6e9ed26b4f920606fb08576b1663c21a7c4a28c5) which is the key in the
                    // starknet contract for
                    // ERC20_balances(0x02e63de215f650e9d7e2313c6e9ed26b4f920606fb08576b1663c21a7c4a28c5).high (this
                    // address corresponds to the sender address of the invoke tx from json)
                    H256::from_str("0x060b6ac06a42730e54bfd5d389ca51256c926bc9317adb44f7c1029711f8bf8e").unwrap(),
                ),
                U256::from(u128::MAX),
            ),
            (
                (
                    fee_token_address,
                    // pedersen(sn_keccak(b"ERC20_balances"),
                    // 0x02e63de215f650e9d7e2313c6e9ed26b4f920606fb08576b1663c21a7c4a28c5) + 1 which is the key in the
                    // starknet contract for
                    // ERC20_balances(0x02e63de215f650e9d7e2313c6e9ed26b4f920606fb08576b1663c21a7c4a28c5).high (this
                    // address corresponds to the sender address of the invoke tx from json)
                    H256::from_str("0x060b6ac06a42730e54bfd5d389ca51256c926bc9317adb44f7c1029711f8bf8f").unwrap(),
                ),
                U256::from(u128::MAX),
            ),
            (
                (
                    fee_token_address,
                    // pedersen(sn_keccak(b"ERC20_balances"),
                    // 0x05ef3fba22df259bf84890945352df711bcc9a4e3b6858cb93e9c90d053cf122) which is the key in the
                    // starknet contract for
                    // ERC20_balances(0x05ef3fba22df259bf84890945352df711bcc9a4e3b6858cb93e9c90d053cf122).high (this
                    // address corresponds to the sender address of the invoke tx from json)
                    H256::from_str("0x078f9a7bb317327b7ad49232784f8e6acfa88269879253bbf780c5bc7a18149a").unwrap(),
                ),
                U256::from(u128::MAX),
            ),
            (
                (
                    fee_token_address,
                    // pedersen(sn_keccak(b"ERC20_balances"),
                    // 0x05ef3fba22df259bf84890945352df711bcc9a4e3b6858cb93e9c90d053cf122) + 1 which is the key in the
                    // starknet contract for
                    // ERC20_balances(0x05ef3fba22df259bf84890945352df711bcc9a4e3b6858cb93e9c90d053cf122).high (this
                    // address corresponds to the sender address of the invoke tx from json)
                    H256::from_str("0x078f9a7bb317327b7ad49232784f8e6acfa88269879253bbf780c5bc7a18149b").unwrap(),
                ),
                U256::from(u128::MAX),
            ),
            (
                (
                    openzeppelin_account_address,
                    // pedersen(sn_keccak(b"Account_public_key")) which is the key in the starknet contract
                    H256::from_str("0x01379ac0624b939ceb9dede92211d7db5ee174fe28be72245b0a1a2abd81c98f").unwrap(),
                ),
                U256::from_str(ACCOUNT_PUBLIC_KEY).unwrap(),
            ),
            (
                (
                    argent_account_address,
                    // pedersen(sn_keccak(b"_signer")) which is the key in the starknet contract
                    H256::from_str("0x01ccc09c8a19948e048de7add6929589945e25f22059c7345aaf7837188d8d05").unwrap(),
                ),
                U256::from_str(ACCOUNT_PUBLIC_KEY).unwrap(),
            ),
            (
                (
                    braavos_account_address,
                    // pedersen(sn_keccak(b"Account_signers"), 0x0) which is the key in the starknet contract
                    H256::from_str("0x01f23302c120008f28b62f70efc67ccd75cfe0b9631d77df231d78b0538dcd8f").unwrap(),
                ),
                U256::from_str("0x03603a2692a2ae60abb343e832ee53b55d6b25f02a3ef1565ec691edc7a209b2").unwrap(),
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

pub enum AccountType {
    Argent,
    ArgentV0,
    Openzeppelin,
    Braavos,
    BraavosProxy,
    NoValidate,
    InnerCall,
}

pub fn account_helper(salt: &str, account_type: AccountType) -> ([u8; 32], [u8; 32], Vec<&str>) {
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
                "0x0244ca3d9fe8b47dd565a6f4270d979ba31a7d6ff2c3bf8776198161505e8b52", // Braavos account class hash
                "0x02dd76e7ad84dbed81c314ffe5e7a7cacfb8f4836f01af4e913f275f89a3de1a", // 'initializer' selector
            ],
        ),
        AccountType::Openzeppelin => (H256::from_str(OPENZEPPELIN_ACCOUNT_CLASS_HASH).unwrap(), vec![]),
        AccountType::NoValidate => (H256::from_str(SIMPLE_ACCOUNT_CLASS_HASH).unwrap(), vec![]),
        AccountType::InnerCall => (H256::from_str(UNAUTHORIZED_INNER_CALL_ACCOUNT_CLASS_HASH).unwrap(), vec![]),
    };
    let account_salt = H256::from_str(salt).unwrap();

    let addr = calculate_contract_address(account_salt, account_class_hash, cd_raw.clone()).unwrap();
    (addr.0.0.0, account_class_hash.to_fixed_bytes(), cd_raw)
}

pub fn get_account_address(account_type: AccountType) -> [u8; 32] {
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

pub fn sign_message_hash(hash: H256) -> BoundedVec<H256, MaxArraySize> {
    let signature = sign(
        &FieldElement::from_str(ACCOUNT_PRIVATE_KEY).unwrap(),
        &FieldElement::from_bytes_be(&hash.0).unwrap(),
        &FieldElement::from_str(K).unwrap(),
    )
    .unwrap();
    bounded_vec!(H256::from(signature.r.to_bytes_be()), H256::from(signature.s.to_bytes_be()))
}
