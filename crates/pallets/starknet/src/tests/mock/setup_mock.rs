use frame_support::traits::GenesisBuild;
use mp_starknet::execution::types::Felt252Wrapper;
use starknet_core::types::FieldElement;

use super::helpers::*;
use crate as pallet_starknet;
use crate::tests::constants::*;
use crate::tests::utils::get_contract_class;
use crate::Config;

// Configure a mock runtime to test the pallet.
macro_rules! mock_runtime {
    ($mock_runtime:ident, $enable_state_root:expr, $disable_transaction_fee:expr) => {
		pub mod $mock_runtime {
			use frame_support::parameter_types;
			use frame_support::traits::{ConstU16, ConstU64};
			use sp_core::H256;
			use sp_runtime::testing::Header;
			use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
			use {crate as pallet_starknet, frame_system as system};
			use crate::{ ContractAddressWrapper, SeqAddrUpdate, SequencerAddress};
			use frame_support::traits::Hooks;
			use mp_starknet::sequencer_address::DEFAULT_SEQUENCER_ADDRESS;


			type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<MockRuntime>;
			type Block = frame_system::mocking::MockBlock<MockRuntime>;

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
				pub const TransactionLongevity: u64 = u64::MAX;
				pub const InvokeTxMaxNSteps: u32 = 1_000_000;
				pub const ValidateMaxNSteps: u32 = 1_000_000;
				pub const EnableStateRoot: bool = $enable_state_root;
				pub const DisableTransactionFee: bool = $disable_transaction_fee;
				pub const ProtocolVersion: u8 = 0;
			}

			impl pallet_starknet::Config for MockRuntime {
				type RuntimeEvent = RuntimeEvent;
				type SystemHash = mp_starknet::crypto::hash::pedersen::PedersenHasher;
				type TimestampProvider = Timestamp;
				type UnsignedPriority = UnsignedPriority;
				type TransactionLongevity = TransactionLongevity;
				type InvokeTxMaxNSteps = InvokeTxMaxNSteps;
				type ValidateMaxNSteps = ValidateMaxNSteps;
				type EnableStateRoot = EnableStateRoot;
				type DisableTransactionFee = DisableTransactionFee;
				type ProtocolVersion = ProtocolVersion;
			}

			/// Run to block n.
			/// The function will repeatedly create and run blocks until the block number is equal to `n`.
			/// # Arguments
			/// * `n` - The block number to run to.
			pub(crate) fn run_to_block(n: u64) {
				for b in System::block_number()..=n {
					SeqAddrUpdate::<MockRuntime>::put(true);
					System::set_block_number(b);
					Timestamp::set_timestamp(System::block_number() * 6_000);
					Starknet::on_finalize(b);
				}
			}

			/// Setup initial block and sequencer address for unit tests.
			pub(crate) fn basic_test_setup(n: u64) {
				SeqAddrUpdate::<MockRuntime>::put(true);
				let default_addr: ContractAddressWrapper = ContractAddressWrapper::try_from(&DEFAULT_SEQUENCER_ADDRESS).unwrap();
				SequencerAddress::<MockRuntime>::put(default_addr);
				System::set_block_number(0);
				run_to_block(n);
			}
		}
    };
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext<T: Config>() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<T>().unwrap();

    // ARGENT CLASSES
    let blockifier_account_class = get_contract_class("NoValidateAccount.json", 0);
    let blockifier_account_class_hash = Felt252Wrapper::from_hex_be(BLOCKIFIER_ACCOUNT_CLASS).unwrap();
    let blockifier_account_address = Felt252Wrapper::from_hex_be(BLOCKIFIER_ACCOUNT_ADDRESS).unwrap();

    // TEST CLASSES
    let erc20_class = get_contract_class("ERC20.json", 0);

    // ACCOUNT CONTRACT

    // OPENZEPPELIN ACCOUNT CONTRACT
    let openzeppelin_account_class = get_contract_class("OpenzeppelinAccount.json", 0);
    let openzeppelin_account_class_hash = Felt252Wrapper::from_hex_be(OPENZEPPELIN_ACCOUNT_CLASS_HASH_CAIRO_0).unwrap();
    let openzeppelin_account_address = get_account_address(AccountType::V0(AccountTypeV0Inner::Openzeppelin));

    // ARGENT ACCOUNT CONTRACT
    let argent_account_class = get_contract_class("ArgentAccount.json", 0);
    let argent_account_class_hash = Felt252Wrapper::from_hex_be(ARGENT_ACCOUNT_CLASS_HASH_CAIRO_0).unwrap();
    let argent_account_address = get_account_address(AccountType::V0(AccountTypeV0Inner::Argent));

    // BRAAVOS ACCOUNT CONTRACT
    let braavos_account_class = get_contract_class("BraavosAccount.json", 0);
    let braavos_account_class_hash = Felt252Wrapper::from_hex_be(BRAAVOS_ACCOUNT_CLASS_HASH_CAIRO_0).unwrap();
    let braavos_account_address = get_account_address(AccountType::V0(AccountTypeV0Inner::Braavos));

    let braavos_proxy_class = get_contract_class("Proxy.json", 0);
    let braavos_proxy_class_hash = Felt252Wrapper::from_hex_be(BRAAVOS_PROXY_CLASS_HASH_CAIRO_0).unwrap();
    let braavos_proxy_address = get_account_address(AccountType::V0(AccountTypeV0Inner::BraavosProxy));

    // UNAUTHORIZED INNER CALL ACCOUNT CONTRACT
    let inner_call_account_class = get_contract_class("UnauthorizedInnerCallAccount.json", 0);
    let inner_call_account_class_hash =
        Felt252Wrapper::from_hex_be(UNAUTHORIZED_INNER_CALL_ACCOUNT_CLASS_HASH_CAIRO_0).unwrap();
    let inner_call_account_address = get_account_address(AccountType::V0(AccountTypeV0Inner::InnerCall));

    // NO VALIDATE ACCOUNT CONTRACT
    let no_validate_class = get_contract_class("NoValidateAccount.json", 0);
    let no_validate_class_hash = Felt252Wrapper::from_hex_be(NO_VALIDATE_ACCOUNT_CLASS_HASH_CAIRO_0).unwrap();
    let no_validate_address = get_account_address(AccountType::V0(AccountTypeV0Inner::NoValidate));

    // CAIRO 1 NO VALIDATE ACCOUNT CONTRACT
    let cairo_1_no_validate_account_class = get_contract_class("NoValidateAccount.casm.json", 1);
    let cairo_1_no_validate_account_class_hash =
        Felt252Wrapper::from_hex_be(NO_VALIDATE_ACCOUNT_CLASS_HASH_CAIRO_1).unwrap();
    let cairo_1_no_validate_account_address = get_account_address(AccountType::V1(AccountTypeV1Inner::NoValidate));

    // TEST CONTRACT
    let test_contract_class = get_contract_class("test.json", 0);
    let test_contract_class_hash = Felt252Wrapper::from_hex_be(TEST_CLASS_HASH).unwrap();
    let test_contract_address = Felt252Wrapper::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap();

    // L1 HANDLER CONTRACT
    let l1_handler_class = get_contract_class("l1_handler.json", 0);
    let l1_handler_contract_address = Felt252Wrapper::from_hex_be(L1_HANDLER_CONTRACT_ADDRESS).unwrap();
    let l1_handler_class_hash = Felt252Wrapper::from_hex_be(L1_HANDLER_CLASS_HASH).unwrap();

    // FEE CONTRACT
    let token_class_hash = Felt252Wrapper::from_hex_be(TOKEN_CONTRACT_CLASS_HASH).unwrap();
    let fee_token_address = Felt252Wrapper::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();

    // SINGLE/MULTIPLE EVENT EMITTING CONTRACT
    let single_event_emitting_class = get_contract_class("emit_single_event.json", 0);
    let single_event_emitting_contract_class_hash = Felt252Wrapper::from_hex_be(EMIT_SINGLE_EVENT_CLASS_HASH).unwrap();
    let single_event_emitting_contract_address =
        Felt252Wrapper::from_hex_be(EMIT_SINGLE_EVENT_CONTRACT_ADDRESS).unwrap();
    let multiple_event_emitting_class = get_contract_class("emit_multiple_events_across_contracts.json", 0);
    let multiple_event_emitting_class_hash = Felt252Wrapper::from_hex_be(MULTIPLE_EVENT_EMITTING_CLASS_HASH).unwrap();
    let multiple_event_emitting_contract_address =
        Felt252Wrapper::from_hex_be(MULTIPLE_EVENT_EMITTING_CONTRACT_ADDRESS).unwrap();

    pallet_starknet::GenesisConfig::<T> {
        contracts: vec![
            (test_contract_address, test_contract_class_hash),
            (l1_handler_contract_address, l1_handler_class_hash),
            (blockifier_account_address, blockifier_account_class_hash),
            (openzeppelin_account_address, openzeppelin_account_class_hash),
            (argent_account_address, argent_account_class_hash),
            (braavos_account_address, braavos_account_class_hash),
            (braavos_proxy_address, braavos_proxy_class_hash),
            (no_validate_address, no_validate_class_hash),
            (cairo_1_no_validate_account_address, cairo_1_no_validate_account_class_hash),
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
            (cairo_1_no_validate_account_class_hash, cairo_1_no_validate_account_class),
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
                get_storage_key(&fee_token_address, "ERC20_balances", &[cairo_1_no_validate_account_address], 0),
                Felt252Wrapper::from(u128::MAX),
            ),
            (
                get_storage_key(&fee_token_address, "ERC20_balances", &[cairo_1_no_validate_account_address], 1),
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

mock_runtime!(default_mock, false, false);
mock_runtime!(state_root_mock, true, false);
mock_runtime!(fees_disabled_mock, false, true);
