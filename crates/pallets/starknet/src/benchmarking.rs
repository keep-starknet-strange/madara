#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::benchmarks;

use crate::*;

benchmarks! {
  // Add individual benchmarks here
  infinite_loop {
     // This benchmark runs an infinite loop till all the gas is used.
     // The idea behind running an infinite loop is to run a function in a way
     // that ensure all gas will be consumed. We can then directly relate the time
     // it took to run the function with the gas consumed. Since one unit of weight is
     // nothing but one picosecond of execution, we will therefore, be able to get a relation
     // between the weight and the gas.

     use frame_system::RawOrigin;
     use mp_starknet::starknet_serde::get_contract_class;

     let casm_json_str = include_str!("../../../../cairo-contracts/build/cairo_1/InfiniteLoop.casm.json");

     // If we don't set the sequencer address we get an error `ERC20: cannot transfer to the zero address`
     // because by default the sequencer address is set to 0x0 and the charge_fee code tries to send the fees
     // to the zero address
     pallet::SequencerAddress::<T>::put(ContractAddressWrapper::from_hex_be("0xdead").unwrap());
     let declare_txn = DeclareTransaction {
        version: 2,
        sender_address: Felt252Wrapper::from_hex_be("0x4").unwrap(),
        class_hash: Felt252Wrapper::from_hex_be("0x03d1fc9d009582f159569678e08b6ac6314eb16eb81fdff5860592b7698522c0").unwrap(),
        compiled_class_hash: Some(Felt252Wrapper::from_hex_be("0x033dc29d3e99ec02901d2d95cde569270ffd9ee1e38262d875c88266989d3924").unwrap()) ,
        contract_class: get_contract_class(&casm_json_str, 1),
        nonce:Felt252Wrapper::ZERO,
        signature:vec![].try_into().unwrap(),
        max_fee: Felt252Wrapper::from_hex_be("0xffffffff").unwrap(),
        is_query:false
     };

     // declare the contract
     assert!(pallet::Pallet::<T>::declare(RawOrigin::None.into(), declare_txn).is_ok());

     let invoke_tx = InvokeTransaction {
        version:1,
        sender_address: Felt252Wrapper::from_hex_be("0x4").unwrap(),
        calldata: vec![
            Felt252Wrapper::from_hex_be("0x1").unwrap(),
            Felt252Wrapper::from_hex_be("0x41a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf").unwrap(), // UDC contract
            Felt252Wrapper::from_hex_be("0x1987cbd17808b9a23693d4de7e246a443cfe37e6e7fbaeabd7d7e6532b07c3d").unwrap(),
            Felt252Wrapper::from_hex_be("0x4").unwrap(),
            Felt252Wrapper::from_hex_be("0x3d1fc9d009582f159569678e08b6ac6314eb16eb81fdff5860592b7698522c0").unwrap(), // class hash
            Felt252Wrapper::from_hex_be("0x1").unwrap(),
            Felt252Wrapper::from_hex_be("0x1").unwrap(),
            Felt252Wrapper::from_hex_be("0x0").unwrap(),
        ].try_into().unwrap(),
        nonce: Felt252Wrapper::ONE,
        max_fee: Felt252Wrapper::from_hex_be("0xffffffff").unwrap(),
        signature: vec![].try_into().unwrap(),
        is_query: false
     };

      // deploy the contract
      assert!(pallet::Pallet::<T>::invoke(RawOrigin::None.into(), invoke_tx).is_ok());
  }: {
     // call the infinite loop function
     let invoke_tx = InvokeTransaction {
        version:1,
        sender_address: Felt252Wrapper::from_hex_be("0x4").unwrap(),
        calldata: vec![
            Felt252Wrapper::from_hex_be("0x1").unwrap(),
            Felt252Wrapper::from_hex_be("0x7c328ec75649bb1e8ddd783b75b7d146c7ae13925a2cc5e2a46598fc551c950").unwrap(), // contract address
            Felt252Wrapper::from_hex_be("0x2e7e6ab3df2d8d293198339ef8cda98658e6ddc2d7ffb24116e343e26d3db8d").unwrap(),
            Felt252Wrapper::from_hex_be("0x0").unwrap()
        ].try_into().unwrap(),
        nonce: Felt252Wrapper::TWO,
        max_fee: Felt252Wrapper::from_dec_str("1000000000").unwrap(),
        signature: vec![].try_into().unwrap(),
        is_query: false
     };

     // once we start charging for failing transactions this won't actually throw an error
     assert!(pallet::Pallet::<T>::invoke(RawOrigin::None.into(), invoke_tx).is_err());
  }
}
