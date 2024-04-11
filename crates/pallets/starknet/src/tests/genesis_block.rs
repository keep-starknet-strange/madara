/// These tests exist to ensure that the genesis block hash remains consistent
/// due to some of the contract classes changing it and breaking the full node setup
use sp_runtime::codec::Decode;

use super::mock::default_mock::*;
use super::mock::*;
#[test]
fn check_genesis_block_hash() {
    let t = test_genesis_ext::<MockRuntime>().execute_with(|| {
        let header = System::finalize();
        header.hash()
    });

    let b = test_genesis_ext::<MockRuntime>().execute_with(|| {
        let header = System::finalize();
        header.hash()
    });

    assert_eq!(b, t)
}

#[test]
fn check_genesis_state_root() {
    let t = test_genesis_ext::<MockRuntime>().execute_with(|| {
        let header = System::finalize();
        header.state_root
    });

    let b = test_genesis_ext::<MockRuntime>().execute_with(|| {
        let header = System::finalize();
        header.state_root
    });

    assert_eq!(b, t)
}

#[test]
fn check_genesis_state_version() {
    let t = test_genesis_ext::<MockRuntime>().execute_with(|| {
        System::finalize();
        frame_system::Pallet::<MockRuntime>::runtime_version().state_version()
    });

    let b = test_genesis_ext::<MockRuntime>().execute_with(|| {
        System::finalize();
        frame_system::Pallet::<MockRuntime>::runtime_version().state_version()
    });
    assert_eq!(b, t)
}

#[test]
fn check_genesis_storage_root() {
    let t = test_genesis_ext::<MockRuntime>().execute_with(|| {
        System::finalize();

        let state_version = frame_system::Pallet::<MockRuntime>::runtime_version().state_version();
        <MockRuntime as frame_system::Config>::Hash::decode(&mut &sp_io::storage::root(state_version)[..]).unwrap()
    });

    let b = test_genesis_ext::<MockRuntime>().execute_with(|| {
        System::finalize();

        let state_version = frame_system::Pallet::<MockRuntime>::runtime_version().state_version();
        <MockRuntime as frame_system::Config>::Hash::decode(&mut &sp_io::storage::root(state_version)[..]).unwrap()
    });

    assert_eq!(b, t)
}

#[test]
fn check_decoder() {
    let t = test_genesis_ext::<MockRuntime>().execute_with(|| {
        System::finalize();
        let mut st: &[u8] = &[0u8; 32];
        <MockRuntime as frame_system::Config>::Hash::decode(&mut st).unwrap()
    });

    let b = test_genesis_ext::<MockRuntime>().execute_with(|| {
        System::finalize();
        let mut st: &[u8] = &[0u8; 32];
        <MockRuntime as frame_system::Config>::Hash::decode(&mut st).unwrap()
    });

    assert_eq!(b, t)
}
