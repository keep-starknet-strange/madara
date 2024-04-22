use sp_runtime::BuildStorage;

use crate::genesis_loader::{GenesisData, GenesisLoader};
use crate::{Config, GenesisConfig};

// Configure a mock runtime to test the pallet.
macro_rules! mock_runtime {
    ($mock_runtime:ident, $disable_transaction_fee:expr, $disable_nonce_validation: expr) => {
		pub mod $mock_runtime {
			use frame_support::parameter_types;
			use frame_support::traits::{ConstU16, ConstU64};
			use sp_core::H256;
			use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
			use {crate as pallet_starknet, frame_system as system};
			use crate::{ SeqAddrUpdate, SequencerAddress};
			use frame_support::traits::Hooks;
			use mp_sequencer_address::DEFAULT_SEQUENCER_ADDRESS;
            use mp_felt::Felt252Wrapper;
			use starknet_api::core::{PatriciaKey, ContractAddress};
			use starknet_api::hash::StarkFelt;
			use blockifier::blockifier::block::GasPrices;
			use core::num::NonZeroU128;


			type Block = frame_system::mocking::MockBlock<MockRuntime>;

			frame_support::construct_runtime!(
				pub enum MockRuntime {
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
				type Nonce = u64;
				type Hash = H256;
				type Hashing = BlakeTwo256;
				type AccountId = u64;
				type Lookup = IdentityLookup<Self::AccountId>;
				type Block = Block;
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
				pub const DisableTransactionFee: bool = $disable_transaction_fee;
                pub const DisableNonceValidation: bool = $disable_nonce_validation;
				pub const ProtocolVersion: u8 = 0;
				pub const ProgramHash: Felt252Wrapper = mp_program_hash::SN_OS_PROGRAM_HASH;
				pub const L1GasPrices: GasPrices = GasPrices { eth_l1_gas_price: unsafe { NonZeroU128::new_unchecked(10) }, strk_l1_gas_price: unsafe { NonZeroU128::new_unchecked(10) }, eth_l1_data_gas_price: unsafe { NonZeroU128::new_unchecked(10) }, strk_l1_data_gas_price: unsafe { NonZeroU128::new_unchecked(10) } };
            }

			impl pallet_starknet::Config for MockRuntime {
				type SystemHash = mp_hashers::pedersen::PedersenHasher;
				type TimestampProvider = Timestamp;
				type UnsignedPriority = UnsignedPriority;
				type TransactionLongevity = TransactionLongevity;
				type DisableTransactionFee = DisableTransactionFee;
                type DisableNonceValidation = DisableNonceValidation;
				type ProtocolVersion = ProtocolVersion;
				type ProgramHash = ProgramHash;
				type L1GasPrices = L1GasPrices;
			}

			/// Run to block n.
			/// The function will repeatedly create and run blocks until the block number is equal to `n`.
			/// # Arguments
			/// * `n` - The block number to run to.
			#[allow(unused)]
			pub(crate) fn run_to_block(n: u64) {
				for b in System::block_number()..=n {
					SeqAddrUpdate::<MockRuntime>::put(true);
					System::set_block_number(b);
					Timestamp::set_timestamp(System::block_number() * 6_000);
					Starknet::on_finalize(b);
				}
			}

			/// Setup initial block and sequencer address for unit tests.
			#[allow(unused)]
			pub(crate) fn basic_test_setup(n: u64) {
				SeqAddrUpdate::<MockRuntime>::put(true);
				let default_addr = ContractAddress(PatriciaKey(StarkFelt::new(DEFAULT_SEQUENCER_ADDRESS).unwrap()));
				SequencerAddress::<MockRuntime>::put(default_addr);
				System::set_block_number(0);
				run_to_block(n);
			}
		}
    };
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext<T: Config>() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<T>::default().build_storage().unwrap();

    let genesis_data: GenesisData = serde_json::from_str(std::include_str!("./genesis.json")).unwrap();
    let genesis_loader = GenesisLoader::new(project_root::get_project_root().unwrap(), genesis_data);
    let genesis: GenesisConfig<T> = genesis_loader.into();

    genesis.assimilate_storage(&mut t).unwrap();

    t.into()
}

// Build genesis storage according to the actual runtime.
pub fn test_genesis_ext<T: Config>() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<T>::default().build_storage().unwrap();
    let project_root = project_root::get_project_root().unwrap().join("configs/");

    let genesis_path = project_root.join("genesis-assets/").join("genesis.json");
    let genesis_file_content = std::fs::read_to_string(genesis_path).unwrap();

    let genesis_data: GenesisData = serde_json::from_str(&genesis_file_content).unwrap();
    let genesis_loader = GenesisLoader::new(project_root, genesis_data);
    let genesis: GenesisConfig<T> = genesis_loader.into();

    genesis.assimilate_storage(&mut t).unwrap();

    t.into()
}

mock_runtime!(default_mock, false, false);
mock_runtime!(fees_disabled_mock, true, false);
mock_runtime!(no_nonce_validation_mock, true, true);
