use sp_runtime::codec::Decode;
use sp_runtime::traits::Hash;
use sp_runtime::Storage;

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
        let state_version = frame_system::Pallet::<MockRuntime>::runtime_version().state_version();
        state_version
    });

    let b = test_genesis_ext::<MockRuntime>().execute_with(|| {
        System::finalize();
        let state_version = frame_system::Pallet::<MockRuntime>::runtime_version().state_version();
        state_version
    });
    assert_eq!(b, t)
}

#[test]
fn check_genesis_storage_root() {
    let t = test_genesis_ext::<MockRuntime>().execute_with(|| {
        System::finalize();

        let state_version = frame_system::Pallet::<MockRuntime>::runtime_version().state_version();
        let storage_root =
            <MockRuntime as frame_system::Config>::Hash::decode(&mut &sp_io::storage::root(state_version)[..]).unwrap();
        storage_root
    });

    let b = test_genesis_ext::<MockRuntime>().execute_with(|| {
        System::finalize();

        let state_version = frame_system::Pallet::<MockRuntime>::runtime_version().state_version();
        let storage_root =
            <MockRuntime as frame_system::Config>::Hash::decode(&mut &sp_io::storage::root(state_version)[..]).unwrap();
        storage_root
    });

    assert_eq!(b, t)
}

#[test]
fn check_genesis_child_root() {
    let t = test_genesis_ext::<MockRuntime>().execute_with(|| {
        System::finalize();

        let state_version = frame_system::Pallet::<MockRuntime>::runtime_version().state_version();
        let storage_root = &sp_io::storage::root(state_version)[..];
        storage_root.to_vec()
    });

    let b = test_genesis_ext::<MockRuntime>().execute_with(|| {
        System::finalize();

        let state_version = frame_system::Pallet::<MockRuntime>::runtime_version().state_version();

        let storage_root = &sp_io::storage::root(state_version)[..];
        storage_root.to_vec()
    });

    assert_eq!(b, t)
}

#[test]
fn check_decoder() {
    let t = test_genesis_ext::<MockRuntime>().execute_with(|| {
        System::finalize();
        let mut st: &[u8] = &[0u8; 32];
        let storage_root = <MockRuntime as frame_system::Config>::Hash::decode(&mut st).unwrap();
        storage_root
    });

    let b = test_genesis_ext::<MockRuntime>().execute_with(|| {
        System::finalize();
        let mut st: &[u8] = &[0u8; 32];
        let storage_root = <MockRuntime as frame_system::Config>::Hash::decode(&mut st).unwrap();
        storage_root
    });

    assert_eq!(b, t)
}

#[test]
fn check_genesis_storage() {
    let t = test_genesis_ext::<MockRuntime>().execute_with(|| {
        System::finalize();
        let mut st: &[u8] = &[0u8; 32];
        let storage_root = <MockRuntime as frame_system::Config>::Hash::decode(&mut st).unwrap();
        storage_root
    });
}
