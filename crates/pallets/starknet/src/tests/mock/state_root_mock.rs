use frame_support::parameter_types;
use frame_support::traits::{ConstU16, ConstU64, GenesisBuild, Hooks};
use mp_starknet::execution::types::Felt252Wrapper;
use mp_starknet::sequencer_address::DEFAULT_SEQUENCER_ADDRESS;
use sp_core::H256;
use sp_runtime::testing::Header;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use starknet_core::types::FieldElement;
use {crate as pallet_starknet, frame_system as system};

use super::helpers::*;
use crate::tests::constants::*;
use crate::tests::utils::get_contract_class;
use crate::{Config, ContractAddressWrapper, SeqAddrUpdate, SequencerAddress};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<MockStateRootRuntime>;
type Block = frame_system::mocking::MockBlock<MockStateRootRuntime>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum MockStateRootRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Starknet: pallet_starknet,
        Timestamp: pallet_timestamp,
    }
);

impl pallet_timestamp::Config for MockStateRootRuntime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<{ 6_000 / 2 }>;
    type WeightInfo = ();
}

impl system::Config for MockStateRootRuntime {
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
    pub const TransactionLongevity: u64 = u64::MAX;
    pub const InvokeTxMaxNSteps: u32 = 1_000_000;
    pub const ValidateMaxNSteps: u32 = 1_000_000;
    pub const EnableStateRoot: bool = true;
    pub const ProtocolVersion: u8 = 0;
}

impl pallet_starknet::Config for MockStateRootRuntime {
    type RuntimeEvent = RuntimeEvent;
    type SystemHash = mp_starknet::crypto::hash::pedersen::PedersenHasher;
    type TimestampProvider = Timestamp;
    type UnsignedPriority = UnsignedPriority;
    type TransactionLongevity = TransactionLongevity;
    type InvokeTxMaxNSteps = InvokeTxMaxNSteps;
    type ValidateMaxNSteps = ValidateMaxNSteps;
    type EnableStateRoot = EnableStateRoot;
    type ProtocolVersion = ProtocolVersion;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext_with_state_root() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<MockStateRootRuntime>().unwrap();

    // ARGENT CLASSES
    let blockifier_account_class = get_contract_class("NoValidateAccount.json");
    let blockifier_account_class_hash = Felt252Wrapper::from_hex_be(BLOCKIFIER_ACCOUNT_CLASS).unwrap();
    let blockifier_account_address = Felt252Wrapper::from_hex_be(BLOCKIFIER_ACCOUNT_ADDRESS).unwrap();

    // TEST CLASSES
    let erc20_class = get_contract_class("ERC20.json");

    // ACCOUNT CONTRACT

    // OPENZEPPELIN ACCOUNT CONTRACT
    let openzeppelin_account_class = get_contract_class("OpenzeppelinAccount.json");
    let openzeppelin_account_class_hash = Felt252Wrapper::from_hex_be(OPENZEPPELIN_ACCOUNT_CLASS_HASH).unwrap();
    let openzeppelin_account_address = get_account_address(AccountType::Openzeppelin);

    // ARGENT ACCOUNT CONTRACT
    let argent_account_class = get_contract_class("ArgentAccount.json");
    let argent_account_class_hash = Felt252Wrapper::from_hex_be(ARGENT_ACCOUNT_CLASS_HASH).unwrap();
    let argent_account_address = get_account_address(AccountType::Argent);

    // BRAAVOS ACCOUNT CONTRACT
    let braavos_account_class = get_contract_class("BraavosAccount.json");
    let braavos_account_class_hash = Felt252Wrapper::from_hex_be(BRAAVOS_ACCOUNT_CLASS_HASH).unwrap();
    let braavos_account_address = get_account_address(AccountType::Braavos);

    let braavos_proxy_class = get_contract_class("Proxy.json");
    let braavos_proxy_class_hash = Felt252Wrapper::from_hex_be(BRAAVOS_PROXY_CLASS_HASH).unwrap();
    let braavos_proxy_address = get_account_address(AccountType::BraavosProxy);

    // UNAUTHORIZED INNER CALL ACCOUNT CONTRACT
    let inner_call_account_class = get_contract_class("UnauthorizedInnerCallAccount.json");
    let inner_call_account_class_hash =
        Felt252Wrapper::from_hex_be(UNAUTHORIZED_INNER_CALL_ACCOUNT_CLASS_HASH).unwrap();
    let inner_call_account_address = get_account_address(AccountType::InnerCall);

    // NO VALIDATE ACCOUNT CONTRACT
    let no_validate_class = get_contract_class("NoValidateAccount.json");
    let no_validate_class_hash = Felt252Wrapper::from_hex_be(NO_VALIDATE_ACCOUNT_CLASS_HASH).unwrap();
    let no_validate_address = get_account_address(AccountType::NoValidate);

    // TEST CONTRACT
    let test_contract_class = get_contract_class("test.json");
    let test_contract_class_hash = Felt252Wrapper::from_hex_be(TEST_CLASS_HASH).unwrap();
    let test_contract_address = Felt252Wrapper::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap();

    // L1 HANDLER CONTRACT
    let l1_handler_class = get_contract_class("l1_handler.json");
    let l1_handler_contract_address = Felt252Wrapper::from_hex_be(L1_HANDLER_CONTRACT_ADDRESS).unwrap();
    let l1_handler_class_hash = Felt252Wrapper::from_hex_be(L1_HANDLER_CLASS_HASH).unwrap();

    // FEE CONTRACT
    let token_class_hash = Felt252Wrapper::from_hex_be(TOKEN_CONTRACT_CLASS_HASH).unwrap();
    let fee_token_address = Felt252Wrapper::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();

    // SINGLE/MULTIPLE EVENT EMITTING CONTRACT
    let single_event_emitting_class = get_contract_class("emit_single_event.json");
    let single_event_emitting_contract_class_hash = Felt252Wrapper::from_hex_be(EMIT_SINGLE_EVENT_CLASS_HASH).unwrap();
    let single_event_emitting_contract_address =
        Felt252Wrapper::from_hex_be(EMIT_SINGLE_EVENT_CONTRACT_ADDRESS).unwrap();
    let multiple_event_emitting_class = get_contract_class("emit_multiple_events_across_contracts.json");
    let multiple_event_emitting_class_hash = Felt252Wrapper::from_hex_be(MULTIPLE_EVENT_EMITTING_CLASS_HASH).unwrap();
    let multiple_event_emitting_contract_address =
        Felt252Wrapper::from_hex_be(MULTIPLE_EVENT_EMITTING_CONTRACT_ADDRESS).unwrap();

    pallet_starknet::GenesisConfig::<MockStateRootRuntime> {
        contracts: vec![
            (test_contract_address, test_contract_class_hash),
            (l1_handler_contract_address, l1_handler_class_hash),
            (blockifier_account_address, blockifier_account_class_hash),
            (openzeppelin_account_address, openzeppelin_account_class_hash),
            (argent_account_address, argent_account_class_hash),
            (braavos_account_address, braavos_account_class_hash),
            (braavos_proxy_address, braavos_proxy_class_hash),
            (no_validate_address, no_validate_class_hash),
            (inner_call_account_address, inner_call_account_class_hash),
            (fee_token_address, token_class_hash),
            (single_event_emitting_contract_address, single_event_emitting_contract_class_hash),
            (multiple_event_emitting_contract_address, multiple_event_emitting_class_hash),
        ],
        contract_classes: vec![
            (test_contract_class_hash, test_contract_class),
            (l1_handler_class_hash, l1_handler_class),
            (blockifier_account_class_hash, blockifier_account_class),
            (openzeppelin_account_class_hash, openzeppelin_account_class),
            (argent_account_class_hash, argent_account_class),
            (braavos_account_class_hash, braavos_account_class),
            (braavos_proxy_class_hash, braavos_proxy_class),
            (no_validate_class_hash, no_validate_class),
            (inner_call_account_class_hash, inner_call_account_class),
            (token_class_hash, erc20_class),
            (single_event_emitting_contract_class_hash, single_event_emitting_class),
            (multiple_event_emitting_class_hash, multiple_event_emitting_class),
        ],
        fee_token_address,
        storage: vec![
            (
                get_storage_key(&fee_token_address, "ERC20_balances", &[no_validate_address], 0),
                Felt252Wrapper::from(u128::MAX),
            ),
            (
                get_storage_key(&fee_token_address, "ERC20_balances", &[no_validate_address], 1),
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
            (
                get_storage_key(&multiple_event_emitting_contract_address, "external_contract_addr", &[], 0),
                Felt252Wrapper::from_hex_be(EMIT_SINGLE_EVENT_CONTRACT_ADDRESS).unwrap(),
            ),
        ],
        chain_id: Felt252Wrapper(FieldElement::from_byte_slice_be(b"SN_GOERLI").unwrap()),
        seq_addr_updated: true,
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
pub(crate) fn run_to_block_state_root<T: Config>(n: u64) {
    for b in System::block_number()..=n {
        SeqAddrUpdate::<T>::put(true);
        System::set_block_number(b);
        Timestamp::set_timestamp(System::block_number() * 6_000);
        Starknet::on_finalize(b);
    }
}

/// Setup initial block and sequencer address for unit tests.
pub(crate) fn basic_test_setup_state_root<T: Config>(n: u64) {
    SeqAddrUpdate::<T>::put(true);
    let default_addr: ContractAddressWrapper = ContractAddressWrapper::try_from(&DEFAULT_SEQUENCER_ADDRESS).unwrap();
    SequencerAddress::<T>::put(default_addr);
    System::set_block_number(0);
    run_to_block_state_root::<T>(n);
}

/// Returns the chain id used by the mock runtime.
/// # Returns
/// The chain id of the mock runtime.
pub fn _get_chain_id() -> Felt252Wrapper {
    Starknet::chain_id()
}
