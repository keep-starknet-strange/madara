use frame_support::traits::GenesisBuild;

use crate::genesis_loader::{read_file_to_string, GenesisLoader};
use crate::{Config, GenesisConfig};

// Configure a mock runtime to test the pallet.
macro_rules! mock_runtime {
    ($mock_runtime:ident, $enable_state_root:expr, $disable_transaction_fee:expr, $disable_nonce_validation: expr) => {
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
            use mp_starknet::execution::types::Felt252Wrapper;
            use mp_starknet::constants::SN_GOERLI_CHAIN_ID;


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
                pub const DisableNonceValidation: bool = $disable_nonce_validation;
				pub const ProtocolVersion: u8 = 0;
                pub const ChainId: Felt252Wrapper = SN_GOERLI_CHAIN_ID;
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
                type DisableNonceValidation = DisableNonceValidation;
				type ProtocolVersion = ProtocolVersion;
                type ChainId = ChainId;
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

    let genesis: GenesisLoader =
        serde_json::from_str(&read_file_to_string("crates/pallets/starknet/src/tests/mock/genesis.json")).unwrap();
    let genesis: GenesisConfig<T> = genesis.into();

    genesis.assimilate_storage(&mut t).unwrap();

    t.into()
}

mock_runtime!(default_mock, false, false, false);
mock_runtime!(state_root_mock, true, false, false);
mock_runtime!(fees_disabled_mock, false, true, false);
mock_runtime!(no_nonce_validation_mock, false, true, true);
